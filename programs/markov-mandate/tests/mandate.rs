//! P02 acceptance. Names are the ones `prompts/P02-program-core.md` fixes.
//!
//! The gate ladder itself is unit-tested in `src/gates.rs`
//! (`gate_order_matches_spec`, `daily_counter_rolls`, …) where every rung can
//! be isolated. These tests run the real program in LiteSVM and check the
//! things that are only true on chain: that withdraw is never gated, that the
//! wrong signer cannot act, that a refusal **commits** with a receipt.
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod harness;

use anchor_lang::InstructionData;
use harness::*;
use markov_mandate::state::MandateState;
use markov_types::{ActionKind, BlockReason};
use solana_signer::Signer;

/// The invariant the whole product rests on: no state disables the exit.
#[test]
fn withdraw_succeeds_in_every_state() {
    for (label, drive) in [
        ("active", (|_: &mut Env| {}) as fn(&mut Env)),
        ("paused", |e: &mut Env| {
            let owner = e.owner.insecure_clone();
            e.pause(&owner).expect("pause");
        }),
        ("revoked", |e: &mut Env| {
            let owner = e.owner.insecure_clone();
            e.revoke(&owner).expect("revoke");
        }),
        ("expired", |e: &mut Env| {
            // Walk the clock past expiry. Withdraw must not care.
            set_clock(&mut e.svm, NOW + 86_400 + 1);
        }),
        ("revoked_and_expired", |e: &mut Env| {
            let owner = e.owner.insecure_clone();
            e.revoke(&owner).expect("revoke");
            set_clock(&mut e.svm, NOW + 86_400 + 1);
        }),
        ("halted", |e: &mut Env| e.set_global_halt(true)),
    ] {
        for amount in [1u64, 7, 100, 1_000] {
            let mut e = Env::new(1_000);
            drive(&mut e);
            let before = e.token_amount(&e.owner_ata);
            let owner = e.owner.insecure_clone();
            e.withdraw(amount, &owner)
                .unwrap_or_else(|err| panic!("withdraw of {amount} failed in {label}: {err:?}"));
            assert_eq!(e.vault_amount(), 1_000 - amount, "{label}");
            assert_eq!(e.token_amount(&e.owner_ata), before + amount, "{label}");
        }
    }
}

#[test]
fn withdraw_rejects_non_owner() {
    let mut e = Env::new(1_000);
    let stranger = e.stranger.insecure_clone();
    assert!(e.withdraw(1, &stranger).is_err());
    assert_eq!(e.vault_amount(), 1_000);
}

#[test]
fn operator_cannot_withdraw() {
    let mut e = Env::new(1_000);
    let operator = e.operator.insecure_clone();
    // Both shapes: as the signer, and trying to send the coins elsewhere.
    assert!(e.withdraw(1, &operator).is_err());
    let elsewhere = e.owner_ata;
    assert!(e.withdraw_to(1, &operator, elsewhere).is_err());
    assert_eq!(e.vault_amount(), 1_000);
}

#[test]
fn operator_cannot_unpause() {
    let mut e = Env::new(0);
    let owner = e.owner.insecure_clone();
    e.pause(&owner).expect("pause");
    let operator = e.operator.insecure_clone();
    assert!(e.unpause(&operator).is_err());
    assert_eq!(e.mandate_state().state, MandateState::Paused);
    // The owner still can.
    e.unpause(&owner).expect("owner unpause");
    assert_eq!(e.mandate_state().state, MandateState::Active);
}

#[test]
fn emergency_cannot_unpause_or_withdraw() {
    let mut e = Env::new(1_000);
    let emergency = e.emergency.insecure_clone();
    // It may pause…
    e.pause(&emergency).expect("emergency pause");
    assert_eq!(e.mandate_state().state, MandateState::Paused);
    // …and it may not unpause.
    assert!(e.unpause(&emergency).is_err());
    assert_eq!(e.mandate_state().state, MandateState::Paused);
    // …and it may not withdraw.
    assert!(e.withdraw(1, &emergency).is_err());
    assert_eq!(e.vault_amount(), 1_000);
    // It may revoke, which is protective.
    e.revoke(&emergency).expect("emergency revoke");
    assert_eq!(e.mandate_state().state, MandateState::Revoked);
    // And the owner still leaves.
    let owner = e.owner.insecure_clone();
    e.withdraw(1_000, &owner)
        .expect("owner withdraw after emergency revoke");
    assert_eq!(e.vault_amount(), 0);
}

#[test]
fn amend_tighten_ok() {
    let mut e = Env::new(0);
    let owner = e.owner.insecure_clone();
    let mut p = policy(e.venue, e.mint, NOW + 86_400);
    p.per_tx_cap = 10;
    p.daily_cap = 20;
    p.max_slippage_bps = 10;
    p.max_mark_age_secs = 60;
    p.expiry_ts = NOW + 3_600;
    p.allowed_actions = markov_mandate::state::action_bits::CLOSE;
    e.amend(p, &owner).expect("tighten");
    let m = e.mandate_state();
    assert_eq!(m.policy.per_tx_cap, 10);
    assert_eq!(m.policy.expiry_ts, NOW + 3_600);
    assert_eq!(
        m.policy.allowed_actions,
        markov_mandate::state::action_bits::CLOSE
    );
}

#[test]
fn amend_widen_rejected() {
    let owner_widenings: [(&str, fn(&mut markov_mandate::state::Policy)); 7] = [
        ("per_tx_cap", |p| p.per_tx_cap += 1),
        ("daily_cap", |p| p.daily_cap += 1),
        ("spend_per_call", |p| p.spend_per_call += 1),
        ("spend_daily", |p| p.spend_daily += 1),
        ("max_slippage_bps", |p| p.max_slippage_bps += 1),
        ("max_mark_age_secs", |p| p.max_mark_age_secs += 1),
        ("expiry_ts", |p| p.expiry_ts += 1),
    ];
    for (label, widen) in owner_widenings {
        let mut e = Env::new(0);
        let owner = e.owner.insecure_clone();
        let mut p = policy(e.venue, e.mint, NOW + 86_400);
        widen(&mut p);
        assert!(e.amend(p, &owner).is_err(), "widening {label} was accepted");
        assert_eq!(
            e.mandate_state().policy,
            policy(e.venue, e.mint, NOW + 86_400),
            "policy moved after a rejected amend of {label}"
        );
    }
    // A venue or token the old policy did not contain is also a widening.
    let mut e = Env::new(0);
    let owner = e.owner.insecure_clone();
    let mut p = policy(e.venue, e.mint, NOW + 86_400);
    p.venues[0] = solana_pubkey::Pubkey::new_unique();
    assert!(e.amend(p, &owner).is_err(), "a new venue was accepted");
    let mut p = policy(e.venue, e.mint, NOW + 86_400);
    p.allowed_actions = markov_mandate::state::action_bits::ALL | 1 << 15;
    assert!(
        e.amend(p, &owner).is_err(),
        "an unknown action bit was accepted"
    );
}

#[test]
fn amend_rejects_non_owner_and_revoked() {
    let mut e = Env::new(0);
    let mut p = policy(e.venue, e.mint, NOW + 86_400);
    p.per_tx_cap = 1;
    let operator = e.operator.insecure_clone();
    let emergency = e.emergency.insecure_clone();
    assert!(e.amend(p, &operator).is_err());
    assert!(e.amend(p, &emergency).is_err());
    let owner = e.owner.insecure_clone();
    e.revoke(&owner).expect("revoke");
    assert!(
        e.amend(p, &owner).is_err(),
        "amend accepted on a revoked mandate"
    );
}

/// The most important implementation detail in the program: a refusal is a
/// committed log, not a rolled-back error.
#[test]
fn refusal_emits_receipt_and_commits() {
    let mut e = Env::new(1_000);
    let operator = e.operator.insecure_clone();
    let over_cap = intent(ActionKind::Open, 51, 1); // policy per_tx_cap is 50
    let meta = e
        .execute(over_cap, &operator)
        .expect("a refusal must succeed as a transaction");

    let receipts = refusals(&meta);
    assert_eq!(receipts.len(), 1, "expected exactly one RefusalReceipt");
    let r = &receipts[0];
    assert_eq!(r.reason, BlockReason::OverTxCap);
    assert_eq!(r.gate_index, 8);
    assert_eq!(r.notional, 51);
    assert_eq!(r.operator, operator.pubkey());
    assert_eq!(r.strategy_id, markov_mandate::BOOK_ONE);
    assert!(!r.forced);
    assert!(
        actions(&meta).is_empty(),
        "a refusal must not also emit an ActionReceipt"
    );

    // Committed: the sequence advanced and the counters did not.
    let m = e.mandate_state();
    assert_eq!(m.action_seq, 1, "the receipt's seq was not persisted");
    assert_eq!(
        m.day_notional_used, 0,
        "a refused action must not spend the daily cap"
    );
    assert_eq!(
        e.vault_amount(),
        1_000,
        "a refused action must not move tokens"
    );
}

#[test]
fn every_refusal_reason_reaches_the_chain_with_its_gate_index() {
    let cases: [(
        &str,
        fn(&mut Env) -> markov_mandate::gates::Intent,
        BlockReason,
        u8,
    ); 6] = [
        (
            "global halt",
            |e| {
                e.set_global_halt(true);
                intent(ActionKind::Open, 10, 1)
            },
            BlockReason::GlobalHalt,
            1,
        ),
        (
            "revoked",
            |e| {
                let owner = e.owner.insecure_clone();
                e.revoke(&owner).expect("revoke");
                intent(ActionKind::Open, 10, 2)
            },
            BlockReason::Revoked,
            2,
        ),
        (
            "paused",
            |e| {
                let owner = e.owner.insecure_clone();
                e.pause(&owner).expect("pause");
                intent(ActionKind::Open, 10, 3)
            },
            BlockReason::Paused,
            2,
        ),
        (
            "over tx cap",
            |_| intent(ActionKind::Open, 51, 4),
            BlockReason::OverTxCap,
            8,
        ),
        (
            "over spend cap",
            |_| {
                let mut i = intent(ActionKind::Open, 10, 5);
                i.spend = 6; // policy spend_per_call is 5
                i
            },
            BlockReason::OverSpendCap,
            10,
        ),
        (
            "slippage",
            |_| {
                let mut i = intent(ActionKind::Open, 10, 6);
                i.max_slippage_bps = 51; // policy allows 50
                i
            },
            BlockReason::SlippageExceeded,
            11,
        ),
    ];
    for (label, drive, reason, gate) in cases {
        let mut e = Env::new(1_000);
        let it = drive(&mut e);
        let operator = e.operator.insecure_clone();
        let meta = e
            .execute(it, &operator)
            .unwrap_or_else(|err| panic!("{label}: {err:?}"));
        let rs = refusals(&meta);
        assert_eq!(rs.len(), 1, "{label}: expected one receipt");
        assert_eq!(rs[0].reason, reason, "{label}");
        assert_eq!(rs[0].gate_index, gate, "{label}");
        assert_eq!(
            e.vault_amount(),
            1_000,
            "{label}: tokens moved on a refusal"
        );
    }
}

/// A mark that is too old is `StaleOracle` — and a mark for another feed, or
/// one that is not the mandate's account, is the same refusal, because none of
/// them is a price this mandate may act on.
#[test]
fn stale_or_unbound_mark_is_refused() {
    // Too old: publish_time older than policy.max_mark_age_secs (150 s).
    let mut e = Env::new(1_000);
    let price = e.price_update;
    let mut acc = e.svm.get_account(&price).unwrap();
    acc.data = price_update_data(FEED_ID, MARK_PRICE_E6 as i64, NOW - 1_000, 1);
    e.svm.set_account(price, acc).unwrap();
    let operator = e.operator.insecure_clone();
    let meta = e
        .execute(intent(ActionKind::Open, 10, 7), &operator)
        .expect("commits");
    let rs = refusals(&meta);
    assert_eq!(rs[0].reason, BlockReason::StaleOracle);
    assert_eq!(rs[0].gate_index, 12);

    // Right shape, wrong feed.
    let mut e = Env::new(1_000);
    let price = e.price_update;
    let mut acc = e.svm.get_account(&price).unwrap();
    acc.data = price_update_data([0x11; 32], MARK_PRICE_E6 as i64, NOW - 10, 1);
    e.svm.set_account(price, acc).unwrap();
    let operator = e.operator.insecure_clone();
    let meta = e
        .execute(intent(ActionKind::Open, 10, 8), &operator)
        .expect("commits");
    assert_eq!(refusals(&meta)[0].reason, BlockReason::StaleOracle);
}

#[test]
fn duplicate_intent_refused() {
    let mut e = Env::new(1_000);
    let operator = e.operator.insecure_clone();
    let it = intent(ActionKind::Open, 10, 9);

    // First one is allowed: the ladder passes and the venue accepts.
    let meta = e.execute(it, &operator).expect("first execute");
    let allowed = actions(&meta);
    assert_eq!(
        allowed.len(),
        1,
        "expected an ActionReceipt: {:?}",
        meta.logs
    );
    assert_eq!(allowed[0].notional, 10);
    assert_eq!(allowed[0].mark_price, MARK_PRICE_E6);
    let m = e.mandate_state();
    assert_eq!(m.day_notional_used, 10, "the daily counter did not advance");
    assert_eq!(m.day_spend_used, 1);

    // The same id again in the same UTC day is a replay.
    let meta = e.execute(it, &operator).expect("second execute commits");
    let rs = refusals(&meta);
    assert_eq!(rs.len(), 1);
    assert_eq!(rs[0].reason, BlockReason::DuplicateIntent);
    assert_eq!(rs[0].gate_index, 4);
    assert_eq!(
        e.mandate_state().day_notional_used,
        10,
        "a replay must not spend the cap twice"
    );
}

#[test]
fn only_the_operator_may_propose() {
    let mut e = Env::new(1_000);
    for signer in [
        e.owner.insecure_clone(),
        e.emergency.insecure_clone(),
        e.stranger.insecure_clone(),
    ] {
        let meta = e
            .execute(intent(ActionKind::Open, 10, 20), &signer)
            .expect("commits as a refusal");
        let rs = refusals(&meta);
        assert_eq!(rs[0].reason, BlockReason::Unauthorized);
        assert_eq!(rs[0].gate_index, 3);
    }
}

#[test]
fn owner_verbs_emit_receipts() {
    let mut e = Env::new(0);
    let owner = e.owner.insecure_clone();
    let meta = e.fund(100).expect("fund");
    let evs = owner_actions(&meta);
    assert_eq!(evs.len(), 1);
    assert_eq!(evs[0].amount, 100);
    assert_eq!(evs[0].owner, owner.pubkey());

    let meta = e.pause(&owner).expect("pause");
    assert_eq!(owner_actions(&meta).len(), 1);
    let meta = e.unpause(&owner).expect("unpause");
    assert_eq!(owner_actions(&meta).len(), 1);
    let meta = e.withdraw(50, &owner).expect("withdraw");
    let evs = owner_actions(&meta);
    assert_eq!(evs[0].amount, 50);
    assert_eq!(evs[0].actor, owner.pubkey());
}

/// B6's proof is a pair: revoke, then the next attempt refused. Both halves
/// have to be indexable from the chain.
#[test]
fn revoke_then_next_attempt_refused() {
    let mut e = Env::new(1_000);
    let owner = e.owner.insecure_clone();
    let revoke_meta = e.revoke(&owner).expect("revoke");
    assert_eq!(
        owner_actions(&revoke_meta).len(),
        1,
        "revoke emitted no receipt"
    );

    let operator = e.operator.insecure_clone();
    let refuse_meta = e
        .execute(intent(ActionKind::Open, 10, 30), &operator)
        .expect("the refusal commits");
    let rs = refusals(&refuse_meta);
    assert_eq!(rs[0].reason, BlockReason::Revoked);

    // …and the owner still leaves, in the revoked state.
    e.withdraw(1_000, &owner).expect("withdraw while revoked");
    assert_eq!(e.vault_amount(), 0);
}

#[test]
fn fund_is_refused_once_revoked() {
    let mut e = Env::new(0);
    let owner = e.owner.insecure_clone();
    e.revoke(&owner).expect("revoke");
    assert!(e.fund(10).is_err());
}

#[test]
fn a_venue_outside_the_registry_is_refused_even_if_the_policy_allows_it() {
    let mut e = Env::new(1_000);
    e.set_adapters(vec![]); // policy still lists it; the registry no longer does
    let operator = e.operator.insecure_clone();
    let meta = e
        .execute(intent(ActionKind::Open, 10, 40), &operator)
        .expect("commits");
    let rs = refusals(&meta);
    assert_eq!(rs[0].reason, BlockReason::ProgramNotAllowed);
    assert_eq!(rs[0].gate_index, 5);
}

#[test]
fn close_mandate_needs_an_empty_vault_and_returns_rent() {
    let mut e = Env::new(100);
    let owner = e.owner.insecure_clone();
    let ix_data = markov_mandate::instruction::CloseMandate {}.data();
    let close = |e: &mut Env, owner: &solana_keypair::Keypair| {
        let metas = markov_mandate::accounts::CloseMandate {
            owner: owner.pubkey(),
            mandate: e.mandate,
            vault: e.vault,
            token_program: anchor_spl::token::ID,
            event_authority: e.event_authority,
            program: e.program,
        };
        let ix = solana_instruction::Instruction {
            program_id: e.program,
            accounts: anchor_lang::ToAccountMetas::to_account_metas(&metas, None)
                .into_iter()
                .map(|m| solana_instruction::AccountMeta {
                    pubkey: m.pubkey,
                    is_signer: m.is_signer,
                    is_writable: m.is_writable,
                })
                .collect(),
            data: ix_data.clone(),
        };
        e.send(ix, &[owner])
    };
    assert!(close(&mut e, &owner).is_err(), "closed with a funded vault");
    e.withdraw(100, &owner).expect("withdraw");
    close(&mut e, &owner).expect("close");
    assert!(e
        .svm
        .get_account(&e.mandate)
        .is_none_or(|a| a.data.is_empty()));
}

// ─────────────── P04: the venue is real now, so gates 13 and 14 fire ───────

/// The receipt must carry the price the venue actually filled at. Before P04
/// this field was `intent.limit_price` — a fabricated fill on a signed
/// receipt, which is exactly what ADR-007 exists to prevent.
#[test]
fn the_action_receipt_carries_the_venues_real_fill_not_the_limit() {
    let mut e = Env::new(1_000);
    let operator = e.operator.insecure_clone();
    let mut it = intent(ActionKind::Open, 10, 50);
    // Ask with a limit above the mark, so limit and fill cannot coincide.
    // +30 bps: inside the policy's 50 bps bound, and deliberately not equal
    // to the fill (mark + the venue's 10 bps fee), so the two cannot coincide
    // and pass this test by accident.
    it.limit_price = MARK_PRICE_E6 + MARK_PRICE_E6 * 30 / 10_000;
    let meta = e.execute(it, &operator).expect("execute");
    let a = actions(&meta);
    assert_eq!(a.len(), 1, "no ActionReceipt: {:?}", meta.logs);
    let r = &a[0];

    // demo_perps' market charges 10 bps and a taker never gets a better
    // price than the mark, so a long open fills at mark + 10 bps.
    let expected = MARK_PRICE_E6 + MARK_PRICE_E6 * 10 / 10_000;
    assert_eq!(r.fill_price, expected, "fill is not mark + fee");
    assert_ne!(
        r.fill_price, it.limit_price,
        "fill_price is the limit price again"
    );
    assert_eq!(r.mark_price, MARK_PRICE_E6);
    assert_eq!(r.fee, 10 * 10 / 10_000, "fee is not the venue's");
    assert_eq!(r.notional, 10);

    // …and the venue really moved: the position exists at that price.
    let p = e.venue_position_state();
    assert_eq!(p.notional, 10);
    assert_eq!(p.entry_price, expected);
}

/// Gate 13: when the venue refuses, the mandate records a refusal and commits
/// — it does not fail the transaction and it does not invent a fill.
#[test]
fn a_venue_refusal_is_a_committed_receipt_at_gate_13() {
    for (label, drive) in [
        (
            "stale venue mark",
            (|e: &mut Env| {
                // The venue's own mark, not the Pyth account the mandate reads:
                // demo_perps refuses on its own freshness rule (max_age 300
                // slots). The Pyth mark stays fresh, so this isolates gate 13
                // from gate 12: the mandate's own freshness gate passes and the
                // venue is the one that refuses.
                e.venue_replay_old_mark(1);
                set_clock_slot(&mut e.svm, 100_000);
            }) as fn(&mut Env),
        ),
        ("paused venue", |e: &mut Env| e.venue_pause(true)),
    ] {
        let mut e = Env::new(1_000);
        drive(&mut e);
        let operator = e.operator.insecure_clone();
        let meta = e
            .execute(intent(ActionKind::Open, 10, 60), &operator)
            .unwrap_or_else(|err| panic!("{label}: a venue refusal must commit, got {err:?}"));
        let rs = refusals(&meta);
        assert_eq!(
            rs.len(),
            1,
            "{label}: expected one receipt, logs {:?}",
            meta.logs
        );
        assert_eq!(rs[0].reason, BlockReason::VenueRejected, "{label}");
        assert_eq!(rs[0].gate_index, 13, "{label}");
        assert!(
            actions(&meta).is_empty(),
            "{label}: a refusal emitted an ActionReceipt"
        );
        assert_eq!(
            e.venue_position_state().notional,
            0,
            "{label}: the position moved anyway"
        );
        assert_eq!(
            e.mandate_state().day_notional_used,
            0,
            "{label}: the cap was spent"
        );
    }
}

/// A fill can be smaller than the intent but never larger, and the daily
/// counter spends what was filled.
#[test]
fn the_counter_spends_the_filled_notional() {
    let mut e = Env::new(1_000);
    let operator = e.operator.insecure_clone();
    let meta = e
        .execute(intent(ActionKind::Open, 40, 70), &operator)
        .expect("open");
    assert_eq!(actions(&meta)[0].notional, 40);
    assert_eq!(e.mandate_state().day_notional_used, 40);

    // Closing reports the whole held size, which is what the venue filled.
    let mut close = intent(ActionKind::Close, 40, 71);
    close.action = ActionKind::Close;
    let meta = e.execute(close, &operator).expect("close");
    let a = actions(&meta);
    assert_eq!(a.len(), 1, "{:?}", meta.logs);
    assert_eq!(a[0].notional, 40, "close did not report the held size");
    assert_eq!(e.venue_position_state().notional, 0);
}
