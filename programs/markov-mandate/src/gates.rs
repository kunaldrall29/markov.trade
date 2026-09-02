//! The fail-closed ladder, in the order docs/10 §3 fixes.
//!
//! One function per gate, called in sequence. The first failure short-circuits
//! and names both the reason and its 1-based rung, so a receipt says *which*
//! gate refused and a test can assert the order. Gates 1–12 are pure over
//! their inputs and are unit-tested without a validator; 13 (`VenueRejected`)
//! and 14 (`PostCheckFailed`) happen around the CPI and live in the
//! `execute_venue_action` handler.

use anchor_lang::prelude::*;
use markov_types::{ActionKind, BlockReason};

use crate::state::{action_bits, Mandate, MandateState, Policy, Registry};

/// What the operator proposes. One action per tick, no batching.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Intent {
    /// `blake3(mandate, slot_bucket, action, amount, nonce)`, built off-chain.
    pub intent_id: [u8; 32],
    pub action: ActionKind,
    pub market: [u8; 16],
    /// Notional in mint base units.
    pub notional: u64,
    pub side: markov_types::Side,
    /// Limit price, scaled 1e6 per unit, same scale as the mark.
    pub limit_price: u64,
    pub max_slippage_bps: u16,
    /// Data/compute spend charged to this action.
    pub spend: u64,
    /// Redteam marker. It never skips a gate; it is only recorded, so nobody
    /// can later claim a forced refusal was organic.
    pub forced: bool,
}

/// A refusal: the reason and the rung that produced it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Refusal {
    pub reason: BlockReason,
    pub gate_index: u8,
}

const fn refuse(reason: BlockReason, gate_index: u8) -> Option<Refusal> {
    Some(Refusal { reason, gate_index })
}

/// What the mark contributed this tick. `None` means it could not be read or
/// bound — which is a refusal at gate 12, never a default price.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MarkInput {
    pub price_e6: u64,
}

pub fn action_bit(action: ActionKind) -> u16 {
    match action {
        ActionKind::Open => action_bits::OPEN,
        ActionKind::Increase => action_bits::INCREASE,
        ActionKind::Reduce => action_bits::REDUCE,
        ActionKind::Close => action_bits::CLOSE,
        ActionKind::Flatten => action_bits::FLATTEN,
        // `Skip` is the agent's default and is never submitted; no bit maps to
        // it, so it refuses at gate 7 rather than being silently accepted.
        ActionKind::Skip => 0,
    }
}

/// Gate 1: the registry circuit.
pub fn gate_global_halt(registry: &Registry) -> Option<Refusal> {
    if registry.global_halt {
        return refuse(BlockReason::GlobalHalt, 1);
    }
    None
}

/// Gate 2: mandate state, then expiry. Expiry is a gate, not a state bit.
pub fn gate_state(mandate: &Mandate, now: i64) -> Option<Refusal> {
    match mandate.state {
        MandateState::Paused => return refuse(BlockReason::Paused, 2),
        MandateState::Revoked => return refuse(BlockReason::Revoked, 2),
        MandateState::Active => {}
    }
    if now >= mandate.policy.expiry_ts {
        return refuse(BlockReason::Expired, 2);
    }
    None
}

/// Gate 3: the signer is the mandate's operator. Nobody else proposes.
pub fn gate_operator(mandate: &Mandate, signer: &Pubkey) -> Option<Refusal> {
    if !mandate.is_operator(signer) {
        return refuse(BlockReason::Unauthorized, 3);
    }
    None
}

/// Gate 4: replay of an intent id inside the same UTC day.
pub fn gate_duplicate_intent(mandate: &Mandate, intent: &Intent) -> Option<Refusal> {
    if mandate.has_recent_intent(&intent.intent_id) {
        return refuse(BlockReason::DuplicateIntent, 4);
    }
    None
}

/// Gate 5: the venue must be in the mandate's policy **and** in the registry.
/// A policy allow is not enough.
pub fn gate_venue(policy: &Policy, registry: &Registry, venue: &Pubkey) -> Option<Refusal> {
    if !policy.venue_allowed(venue) || !registry.adapter_allowed(venue) {
        return refuse(BlockReason::ProgramNotAllowed, 5);
    }
    None
}

/// Gate 6: the settlement mint must be in the policy.
pub fn gate_token(policy: &Policy, mint: &Pubkey) -> Option<Refusal> {
    if !policy.token_allowed(mint) {
        return refuse(BlockReason::TokenNotAllowed, 6);
    }
    None
}

/// Gate 7: the action kind must be permitted.
pub fn gate_action(policy: &Policy, action: ActionKind) -> Option<Refusal> {
    let bit = action_bit(action);
    if bit == 0 || !policy.action_allowed(bit) {
        return refuse(BlockReason::ActionNotAllowed, 7);
    }
    None
}

/// Gate 8: per-action notional cap.
pub fn gate_per_tx_cap(policy: &Policy, notional: u64) -> Option<Refusal> {
    if notional > policy.per_tx_cap {
        return refuse(BlockReason::OverTxCap, 8);
    }
    None
}

/// Gate 9: rolling UTC-day notional cap. Overflow is a refusal, not a wrap.
pub fn gate_daily_cap(mandate: &Mandate, notional: u64) -> Option<Refusal> {
    match mandate.day_notional_used.checked_add(notional) {
        Some(total) if total <= mandate.policy.daily_cap => None,
        _ => refuse(BlockReason::OverDailyCap, 9),
    }
}

/// Gate 10: spend budgets, per call then per day.
pub fn gate_spend(mandate: &Mandate, spend: u64) -> Option<Refusal> {
    if spend > mandate.policy.spend_per_call {
        return refuse(BlockReason::OverSpendCap, 10);
    }
    match mandate.day_spend_used.checked_add(spend) {
        Some(total) if total <= mandate.policy.spend_daily => None,
        _ => refuse(BlockReason::OverSpendDailyCap, 10),
    }
}

/// Gate 11: the intent's slippage bound must be inside the policy's, and the
/// limit price must sit within that bound of the mark. When the mark could not
/// be bound, only the policy half is checked here and gate 12 refuses.
pub fn gate_slippage(policy: &Policy, intent: &Intent, mark: Option<MarkInput>) -> Option<Refusal> {
    if intent.max_slippage_bps > policy.max_slippage_bps {
        return refuse(BlockReason::SlippageExceeded, 11);
    }
    if let Some(mark) = mark {
        if mark.price_e6 == 0 {
            return refuse(BlockReason::SlippageExceeded, 11);
        }
        let diff = intent.limit_price.abs_diff(mark.price_e6) as u128;
        let bound = (mark.price_e6 as u128)
            .saturating_mul(intent.max_slippage_bps as u128)
            / 10_000u128;
        if diff > bound {
            return refuse(BlockReason::SlippageExceeded, 11);
        }
    }
    None
}

/// Gate 12: the mark must be bound and fresh. `None` means it was not.
pub fn gate_mark(mark: Option<MarkInput>) -> Option<Refusal> {
    if mark.is_none() {
        return refuse(BlockReason::StaleOracle, 12);
    }
    None
}

/// Gates 1–12 in order. The first failure wins.
#[allow(clippy::too_many_arguments)]
pub fn evaluate(
    registry: &Registry,
    mandate: &Mandate,
    signer: &Pubkey,
    venue: &Pubkey,
    mint: &Pubkey,
    intent: &Intent,
    mark: Option<MarkInput>,
    now: i64,
) -> Option<Refusal> {
    gate_global_halt(registry)
        .or_else(|| gate_state(mandate, now))
        .or_else(|| gate_operator(mandate, signer))
        .or_else(|| gate_duplicate_intent(mandate, intent))
        .or_else(|| gate_venue(&mandate.policy, registry, venue))
        .or_else(|| gate_token(&mandate.policy, mint))
        .or_else(|| gate_action(&mandate.policy, intent.action))
        .or_else(|| gate_per_tx_cap(&mandate.policy, intent.notional))
        .or_else(|| gate_daily_cap(mandate, intent.notional))
        .or_else(|| gate_spend(mandate, intent.spend))
        .or_else(|| gate_slippage(&mandate.policy, intent, mark))
        .or_else(|| gate_mark(mark))
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::state::{action_bits, MandateState, RECENT_INTENTS};

    fn pk(n: u8) -> Pubkey {
        Pubkey::new_from_array([n; 32])
    }

    const VENUE: u8 = 1;
    const MINT: u8 = 2;
    const OPERATOR: u8 = 3;
    const NOW: i64 = 1_788_400_000;

    fn policy() -> Policy {
        let mut p = Policy {
            venues: [Pubkey::default(); 4],
            venues_len: 1,
            tokens: [Pubkey::default(); 4],
            tokens_len: 1,
            allowed_actions: action_bits::ALL,
            per_tx_cap: 50,
            daily_cap: 200,
            spend_per_call: 5,
            spend_daily: 20,
            max_slippage_bps: 50,
            max_mark_age_secs: 150,
            expiry_ts: NOW + 86_400,
        };
        p.venues[0] = pk(VENUE);
        p.tokens[0] = pk(MINT);
        p
    }

    fn registry() -> Registry {
        let mut r = Registry {
            admin: pk(9),
            global_halt: false,
            adapters: [Pubkey::default(); crate::state::MAX_ADAPTERS],
            adapters_len: 1,
            bump: 255,
        };
        r.adapters[0] = pk(VENUE);
        r
    }

    fn mandate() -> Mandate {
        Mandate {
            owner: pk(8),
            operator: pk(OPERATOR),
            emergency: pk(7),
            strategy_id: crate::BOOK_ONE,
            state: MandateState::Active,
            policy: policy(),
            vault: pk(10),
            mint: pk(MINT),
            mark_account: pk(11),
            feed_id: [0xef; 32],
            day_epoch: Mandate::utc_day(NOW),
            day_notional_used: 0,
            day_spend_used: 0,
            action_seq: 0,
            recent_intents: [[0u8; 32]; RECENT_INTENTS],
            recent_intents_len: 0,
            recent_intents_next: 0,
            created_at: NOW,
            nonce: 0,
            bump: 254,
            vault_bump: 253,
            reserve: [0u8; 128],
        }
    }

    fn intent() -> Intent {
        Intent {
            intent_id: [1u8; 32],
            action: ActionKind::Open,
            market: *b"SOL-PERP\0\0\0\0\0\0\0\0",
            notional: 10,
            side: markov_types::Side::Long,
            limit_price: 100_000_000,
            max_slippage_bps: 50,
            spend: 1,
            forced: false,
        }
    }

    const MARK: Option<MarkInput> = Some(MarkInput {
        price_e6: 100_000_000,
    });

    fn run(r: &Registry, m: &Mandate, i: &Intent, mark: Option<MarkInput>) -> Option<Refusal> {
        evaluate(r, m, &pk(OPERATOR), &pk(VENUE), &pk(MINT), i, mark, NOW)
    }

    #[test]
    fn a_clean_intent_passes_every_gate() {
        assert_eq!(run(&registry(), &mandate(), &intent(), MARK), None);
    }

    /// The ladder, rung by rung: each row makes exactly one thing wrong and
    /// asserts both the reason and the gate index docs/10 §3 fixes.
    #[test]
    fn gate_order_matches_spec() {
        type Mutate = fn(&mut Registry, &mut Mandate, &mut Intent);
        let cases: [(Mutate, BlockReason, u8); 13] = [
            (|r, _, _| r.global_halt = true, BlockReason::GlobalHalt, 1),
            (|_, m, _| m.state = MandateState::Paused, BlockReason::Paused, 2),
            (|_, m, _| m.state = MandateState::Revoked, BlockReason::Revoked, 2),
            (|_, m, _| m.policy.expiry_ts = NOW, BlockReason::Expired, 2),
            (|_, m, _| m.operator = pk(99), BlockReason::Unauthorized, 3),
            (
                |_, m, i| {
                    m.recent_intents[0] = i.intent_id;
                    m.recent_intents_len = 1;
                },
                BlockReason::DuplicateIntent,
                4,
            ),
            (|_, m, _| m.policy.venues[0] = pk(98), BlockReason::ProgramNotAllowed, 5),
            (|_, m, _| m.policy.tokens[0] = pk(97), BlockReason::TokenNotAllowed, 6),
            (|_, m, _| m.policy.allowed_actions = action_bits::CLOSE, BlockReason::ActionNotAllowed, 7),
            (|_, _, i| i.notional = 51, BlockReason::OverTxCap, 8),
            (|_, m, i| { m.day_notional_used = 195; i.notional = 6; }, BlockReason::OverDailyCap, 9),
            (|_, _, i| i.spend = 6, BlockReason::OverSpendCap, 10),
            (|_, _, i| i.max_slippage_bps = 51, BlockReason::SlippageExceeded, 11),
        ];
        for (i, (mutate, reason, gate)) in cases.iter().enumerate() {
            let (mut r, mut m, mut it) = (registry(), mandate(), intent());
            mutate(&mut r, &mut m, &mut it);
            let got = run(&r, &m, &it, MARK);
            assert_eq!(
                got,
                Some(Refusal { reason: *reason, gate_index: *gate }),
                "case {i}: expected {reason:?} at gate {gate}, got {got:?}"
            );
        }
    }

    #[test]
    fn an_earlier_gate_wins_over_a_later_one() {
        // Revoked (gate 2) and over cap (gate 8) at once: the ladder reports 2.
        let (r, mut m, mut i) = (registry(), mandate(), intent());
        m.state = MandateState::Revoked;
        i.notional = 10_000;
        assert_eq!(
            run(&r, &m, &i, MARK),
            Some(Refusal { reason: BlockReason::Revoked, gate_index: 2 })
        );
    }

    #[test]
    fn a_venue_in_the_policy_but_not_the_registry_is_refused() {
        let (mut r, m, i) = (registry(), mandate(), intent());
        r.adapters_len = 0;
        assert_eq!(
            run(&r, &m, &i, MARK),
            Some(Refusal { reason: BlockReason::ProgramNotAllowed, gate_index: 5 })
        );
    }

    #[test]
    fn an_unreadable_mark_is_stale_oracle_at_gate_12() {
        assert_eq!(
            run(&registry(), &mandate(), &intent(), None),
            Some(Refusal { reason: BlockReason::StaleOracle, gate_index: 12 })
        );
    }

    #[test]
    fn a_limit_price_outside_the_bound_is_slippage() {
        let (r, m, mut i) = (registry(), mandate(), intent());
        // 50 bps of 100.000000 is 0.5; 100.6 is outside.
        i.limit_price = 100_600_000;
        assert_eq!(
            run(&r, &m, &i, MARK),
            Some(Refusal { reason: BlockReason::SlippageExceeded, gate_index: 11 })
        );
        i.limit_price = 100_400_000;
        assert_eq!(run(&r, &m, &i, MARK), None);
    }

    #[test]
    fn skip_is_never_a_venue_action() {
        let (r, m, mut i) = (registry(), mandate(), intent());
        i.action = ActionKind::Skip;
        assert_eq!(
            run(&r, &m, &i, MARK),
            Some(Refusal { reason: BlockReason::ActionNotAllowed, gate_index: 7 })
        );
    }

    #[test]
    fn counters_that_would_overflow_refuse_rather_than_wrap() {
        let (r, mut m, mut i) = (registry(), mandate(), intent());
        m.day_notional_used = u64::MAX;
        m.policy.daily_cap = u64::MAX;
        i.notional = 1;
        assert_eq!(
            run(&r, &m, &i, MARK),
            Some(Refusal { reason: BlockReason::OverDailyCap, gate_index: 9 })
        );
        let (r, mut m, mut i) = (registry(), mandate(), intent());
        m.day_spend_used = u64::MAX;
        m.policy.spend_daily = u64::MAX;
        m.policy.spend_per_call = u64::MAX;
        i.spend = 1;
        assert_eq!(
            run(&r, &m, &i, MARK),
            Some(Refusal { reason: BlockReason::OverSpendDailyCap, gate_index: 10 })
        );
    }

    #[test]
    fn daily_counter_rolls() {
        let mut m = mandate();
        m.day_notional_used = 200;
        m.day_spend_used = 20;
        // A different id from the intent below, so this test exercises the
        // daily counter and not the replay ring.
        m.recent_intents[0] = [42u8; 32];
        m.recent_intents_len = 1;
        // Same day: nothing moves, and the cap still bites.
        m.rollover(NOW + 60);
        assert_eq!(m.day_notional_used, 200);
        assert!(m.has_recent_intent(&[42u8; 32]));
        let mut i = intent();
        i.notional = 1;
        assert_eq!(
            run(&registry(), &m, &i, MARK),
            Some(Refusal { reason: BlockReason::OverDailyCap, gate_index: 9 })
        );
        // Next UTC day: counters and the replay ring reset.
        m.rollover(NOW + 86_400);
        assert_eq!(m.day_notional_used, 0);
        assert_eq!(m.day_spend_used, 0);
        assert!(!m.has_recent_intent(&[42u8; 32]));
        assert_eq!(run(&registry(), &m, &i, MARK), None);
    }

    #[test]
    fn the_replay_ring_remembers_the_last_eight_and_forgets_older_ones() {
        let mut m = mandate();
        for n in 0..RECENT_INTENTS as u8 {
            m.remember_intent([n; 32]);
        }
        for n in 0..RECENT_INTENTS as u8 {
            assert!(m.has_recent_intent(&[n; 32]), "forgot {n}");
        }
        m.remember_intent([99; 32]);
        assert!(m.has_recent_intent(&[99; 32]));
        assert!(!m.has_recent_intent(&[0; 32]), "ring did not evict the oldest");
    }
}
