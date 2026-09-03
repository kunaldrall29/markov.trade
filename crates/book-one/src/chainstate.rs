//! Reading the book off the chain.
//!
//! The guard is handed everything and reads nothing; this is where "handed"
//! comes from in devnet mode. Every value below is read from the mandate
//! account itself rather than configured, because a policy in an environment
//! variable is a policy that can disagree with the one being enforced — and
//! the whole claim is that the chain is the authority.
//!
//! A read that fails is not a default. `docs/11` §2: any failure is a `Skip`
//! with a logged reason, never an assumed state.

use anchor_lang::AccountDeserialize;
use markov_chain::{Chain, ChainError};
use markov_guard::{MandateState, PolicyView};
use markov_mandate::state::{Mandate, MandateState as OnChainState};
use solana_pubkey::Pubkey;

/// The mandate as it stands on chain, in the shapes the guard and the core
/// take.
#[derive(Clone, Debug)]
pub struct MandateSnapshot {
    pub vault: Pubkey,
    pub mint: Pubkey,
    pub mark_account: Pubkey,
    pub feed_id: [u8; 32],
    pub state: MandateState,
    pub policy: PolicyView,
    pub day_notional_used: u64,
    pub day_spend_used: u64,
    pub action_seq: u64,
    pub expiry_ts: i64,
    pub venues: Vec<Pubkey>,
}

impl MandateSnapshot {
    /// The state the guard should see, given the time.
    ///
    /// The program has no `Expired` state — expiry is a gate, checked against
    /// the clock (`FACTS` `ACCOUNT_LAYOUT`). The guard *does* have one, so the
    /// translation happens here, once, rather than in three call sites.
    pub fn state_at(&self, now_unix: i64) -> MandateState {
        if self.state == MandateState::Active && self.expiry_ts != 0 && now_unix >= self.expiry_ts {
            MandateState::Expired
        } else {
            self.state
        }
    }
}

/// Read the mandate. Nothing is defaulted: a failure here means the tick
/// skips.
pub fn read_mandate(
    chain: &Chain,
    mandate: &Pubkey,
    delta_band: u128,
    max_gross: u128,
    daily_loss_bps: u16,
) -> Result<MandateSnapshot, ChainError> {
    let account = chain.account_or_missing(mandate)?;
    let m = Mandate::try_deserialize(&mut account.data.as_slice()).map_err(|e| {
        ChainError::NotConfirmed {
            attempts: 1,
            last: format!("mandate {mandate} did not decode: {e}"),
        }
    })?;
    Ok(MandateSnapshot {
        vault: m.vault,
        mint: m.mint,
        mark_account: m.mark_account,
        feed_id: m.feed_id,
        state: match m.state {
            OnChainState::Active => MandateState::Active,
            OnChainState::Paused => MandateState::Paused,
            OnChainState::Revoked => MandateState::Revoked,
        },
        policy: PolicyView {
            max_mark_age_secs: i64::try_from(m.policy.max_mark_age_secs).unwrap_or(i64::MAX),
            // The program's mask is 16 bits and the guard's is 8; every
            // `ActionKind` fits in the low six, so the narrowing is safe and
            // the assertion below keeps it that way.
            allowed_actions: (m.policy.allowed_actions & 0xFF) as u8,
            per_tx_cap: m.policy.per_tx_cap,
            daily_cap: m.policy.daily_cap,
            spend_cap: m.policy.spend_per_call,
            spend_daily_cap: m.policy.spend_daily,
            max_slippage_bps: m.policy.max_slippage_bps,
            // These three have no on-chain counterpart in v0 (ADR-005), so
            // they are the operator's configuration and are labelled
            // `OffChainV0` everywhere they appear.
            delta_band,
            max_gross,
            daily_loss_bps,
        },
        day_notional_used: m.day_notional_used,
        day_spend_used: m.day_spend_used,
        action_seq: m.action_seq,
        expiry_ts: m.policy.expiry_ts,
        venues: m
            .policy
            .venues
            .iter()
            .take(usize::from(m.policy.venues_len))
            .copied()
            .collect(),
    })
}

/// Does this mandate's policy match the Gate B template? P07's pre-flight #2.
///
/// Returned as a list of differences rather than a bool, because "it does not
/// match" is not an answer anyone can act on.
pub fn gate_b_policy_differences(p: &PolicyView) -> Vec<String> {
    use crate::config::{GATE_B_DAILY_CAP, GATE_B_MAX_SLIPPAGE_BPS, GATE_B_PER_TX_CAP};
    let mut out = Vec::new();
    let mut check = |name: &str, got: u64, want: u64| {
        if got != want {
            out.push(format!("{name}: on chain {got}, Gate B template {want}"));
        }
    };
    check("per_tx_cap", p.per_tx_cap, GATE_B_PER_TX_CAP);
    check("daily_cap", p.daily_cap, GATE_B_DAILY_CAP);
    check(
        "max_slippage_bps",
        u64::from(p.max_slippage_bps),
        u64::from(GATE_B_MAX_SLIPPAGE_BPS),
    );
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{GATE_B_DAILY_CAP, GATE_B_MAX_SLIPPAGE_BPS, GATE_B_PER_TX_CAP};

    fn snapshot(state: MandateState, expiry_ts: i64) -> MandateSnapshot {
        MandateSnapshot {
            vault: Pubkey::new_unique(),
            mint: Pubkey::new_unique(),
            mark_account: Pubkey::new_unique(),
            feed_id: [0; 32],
            state,
            policy: gate_b_policy(),
            day_notional_used: 0,
            day_spend_used: 0,
            action_seq: 0,
            expiry_ts,
            venues: vec![],
        }
    }

    fn gate_b_policy() -> PolicyView {
        PolicyView {
            max_mark_age_secs: 150,
            allowed_actions: 62,
            per_tx_cap: GATE_B_PER_TX_CAP,
            daily_cap: GATE_B_DAILY_CAP,
            spend_cap: GATE_B_PER_TX_CAP,
            spend_daily_cap: GATE_B_DAILY_CAP,
            max_slippage_bps: GATE_B_MAX_SLIPPAGE_BPS,
            delta_band: 20_000_000,
            max_gross: 100_000_000,
            daily_loss_bps: 500,
        }
    }

    /// The program has no `Expired` state; the guard does. The translation
    /// happens once, here.
    #[test]
    fn expiry_becomes_a_state_for_the_guard() {
        let s = snapshot(MandateState::Active, 2_000);
        assert_eq!(s.state_at(1_999), MandateState::Active);
        assert_eq!(s.state_at(2_000), MandateState::Expired, "at the boundary");
        assert_eq!(s.state_at(2_001), MandateState::Expired);

        // No expiry set is not "expired at the epoch".
        let never = snapshot(MandateState::Active, 0);
        assert_eq!(never.state_at(i64::MAX), MandateState::Active);

        // A paused or revoked mandate does not become "expired": the stronger
        // fact is the one to report.
        for state in [MandateState::Paused, MandateState::Revoked] {
            assert_eq!(snapshot(state, 1).state_at(9_999), state);
        }
    }

    #[test]
    fn the_gate_b_template_is_checkable() {
        assert!(gate_b_policy_differences(&gate_b_policy()).is_empty());
        let loose = PolicyView {
            per_tx_cap: GATE_B_PER_TX_CAP * 2,
            max_slippage_bps: 500,
            ..gate_b_policy()
        };
        let diffs = gate_b_policy_differences(&loose);
        assert_eq!(diffs.len(), 2, "{diffs:?}");
        assert!(diffs.iter().any(|d| d.contains("per_tx_cap")));
        assert!(diffs.iter().any(|d| d.contains("max_slippage_bps")));
    }
}
