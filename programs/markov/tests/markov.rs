//! Invariant tests (`01_Program_Build_Prompt` §4 and §6) against the built
//! program in LiteSVM. Adjacency is exercised with real transactions: the
//! instructions sysvar the program reads is the one the SVM builds from the
//! transaction under test.
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod harness;

use anchor_lang::{InstructionData, ToAccountMetas};
use harness::*;
use markov::state::*;
use solana_instruction::Instruction;
use solana_signer::Signer;

fn allow_args(id: u128) -> markov::DecisionArgs {
    decision_args(id, Decision::Allow, ok_checks())
}

fn send_decision(
    e: &mut Env,
    signer_is_owner: bool,
    args: &markov::DecisionArgs,
    trailing: Vec<Instruction>,
) -> Result<(), String> {
    let signer = if signer_is_owner {
        e.owner.insecure_clone()
    } else {
        e.actor.insecure_clone()
    };
    let mut ixs = vec![e.record_decision_ix(&signer.pubkey(), args)];
    ixs.extend(trailing);
    e.send(ixs, &signer, &[&signer])
        .map(|_| ())
        .map_err(|m| meta_err(&m))
}

// ---- owner verbs and versions --------------------------------------------------

#[test]
fn account_and_mandate_versions() {
    let mut e = Env::new();
    e.init_config();
    e.create_account();
    // v2 before v1 is refused; v1 then v2 is fine; v2 again is refused.
    assert!(err_contains(
        &e.set_mandate(2, mandate_fields()),
        "BadMandateVersion"
    ));
    e.set_mandate(1, mandate_fields()).unwrap();
    e.set_mandate(2, mandate_fields()).unwrap();
    assert_eq!(e.active_mandate_version(), 2);
    let r = e.set_mandate(2, mandate_fields());
    assert!(r.is_err(), "a version can be set once");
    // Invalid fields are refused.
    let mut bad = mandate_fields();
    bad.max_leverage_bps = 0;
    assert!(err_contains(&e.set_mandate(3, bad), "InvalidMandate"));
}

#[test]
fn owner_only_verbs_need_the_owner_even_with_an_all_scope_permission() {
    let mut e = Env::ready();
    let actor = e.actor.pubkey();
    e.set_permission(actor, scopes::ALL, 0, 0, 0).unwrap();
    // The actor tries to set a mandate: the account PDA is derived from the
    // owner, so an actor-signed set_mandate cannot even address it.
    let ix = e.ix(
        markov::accounts::SetMandate {
            owner: actor,
            account: e.account,
            mandate: e.mandate_pda(2),
            system_program: SYSTEM_PROGRAM,
            event_authority: e.event_authority,
            program: e.program,
        },
        markov::instruction::SetMandate {
            version: 2,
            fields: mandate_fields(),
        },
    );
    let a = e.actor.insecure_clone();
    let r = e.send(vec![ix], &a, &[&a]);
    assert!(r.is_err(), "invariant 7: an actor cannot set a mandate");
    // Same for pause.
    let ix = e.ix(
        markov::accounts::OwnerOnly {
            owner: actor,
            account: e.account,
        },
        markov::instruction::Pause {},
    );
    assert!(e.send(vec![ix], &a, &[&a]).is_err());
    assert_eq!(e.active_mandate_version(), 1);
}

// ---- invariant 1 & 2: allow needs a valid verdict and an adjacent venue leg -------

#[test]
fn allow_with_venue_leg_writes_the_receipt_and_the_ledger() {
    let mut e = Env::ready();
    let owner = e.owner.pubkey();
    let args = allow_args(1);
    send_decision(&mut e, true, &args, vec![venue_leg(&owner)]).unwrap();
    let r = e.receipt(1).expect("receipt");
    assert_eq!(r.decision, Decision::Allow);
    assert_eq!(r.reason_code, reason::NONE);
    assert_eq!(r.mandate_version, 1);
    assert!(!r.finalized);
    // Limits on the enforced checks come from the chain, not the caller.
    let lev = r.checks.iter().find(|c| c.rule == rule::LEVERAGE).unwrap();
    assert_eq!(lev.limit, 20_000);
    assert!(lev.pass);
    let led = e.ledger(args.day_epoch).expect("ledger");
    assert_eq!(led.actions_count, 1);
    assert_eq!(led.gross_notional_usd, 500 * USD);
}

#[test]
fn allow_without_a_venue_leg_is_refused() {
    let mut e = Env::ready();
    let r = send_decision(&mut e, true, &allow_args(2), vec![]);
    assert!(err_contains(&r, "MissingAdjacentInstruction"), "{r:?}");
    assert!(e.receipt(2).is_none());
}

#[test]
fn allow_followed_by_the_wrong_program_is_refused() {
    let mut e = Env::ready();
    let r = send_decision(&mut e, true, &allow_args(3), vec![wrong_leg()]);
    assert!(err_contains(&r, "AdjacentProgramNotAllowed"), "{r:?}");
}

#[test]
fn allow_followed_by_this_program_is_refused() {
    let mut e = Env::ready();
    let owner = e.owner.pubkey();
    // A second record_decision right after the first: the adjacent program is
    // the Markov program itself.
    let second = e.record_decision_ix(&owner, &allow_args(5));
    let r = send_decision(&mut e, true, &allow_args(4), vec![second]);
    assert!(err_contains(&r, "AdjacentIsSelf"), "{r:?}");
}

#[test]
fn allow_whose_observed_values_violate_the_mandate_is_refused() {
    let mut e = Env::ready();
    let owner = e.owner.pubkey();
    let mut checks = ok_checks();
    checks
        .iter_mut()
        .find(|c| c.rule == rule::LEVERAGE)
        .unwrap()
        .observed = 25_000; // > 2.0x
    let args = decision_args(6, Decision::Allow, checks);
    let r = send_decision(&mut e, true, &args, vec![venue_leg(&owner)]);
    assert!(err_contains(&r, "AllowViolatesMandate"), "{r:?}");
    assert!(e.receipt(6).is_none());
}

#[test]
fn allow_on_a_market_outside_the_mandate_is_refused() {
    let mut e = Env::ready();
    let owner = e.owner.pubkey();
    let mut args = allow_args(7);
    args.market_id = *b"DOGE-PERP\0\0\0\0\0\0\0";
    let r = send_decision(&mut e, true, &args, vec![venue_leg(&owner)]);
    assert!(err_contains(&r, "AllowViolatesMandate"), "{r:?}");
}

#[test]
fn stale_data_cannot_be_allowed() {
    let mut e = Env::ready();
    let owner = e.owner.pubkey();
    let mut args = allow_args(8);
    args.data_slot = SLOT - 10; // freshness_slots = 2
    let r = send_decision(&mut e, true, &args, vec![venue_leg(&owner)]);
    assert!(err_contains(&r, "AllowViolatesMandate"), "{r:?}");
}

#[test]
fn a_reject_receipt_lands_with_the_programs_reason_and_no_venue_leg() {
    let mut e = Env::ready();
    let mut checks = ok_checks();
    checks
        .iter_mut()
        .find(|c| c.rule == rule::NOTIONAL)
        .unwrap()
        .observed = 5_000 * USD as i64;
    let args = decision_args(9, Decision::Reject, checks);
    send_decision(&mut e, true, &args, vec![]).unwrap();
    let r = e.receipt(9).unwrap();
    assert_eq!(r.decision, Decision::Reject);
    assert_eq!(r.reason_code, reason::MAX_NOTIONAL_EXCEEDED);
    let n = r.checks.iter().find(|c| c.rule == rule::NOTIONAL).unwrap();
    assert!(!n.pass);
    assert_eq!(n.limit, 1_000 * USD as i64);
    // Nothing but the receipt changed.
    assert_eq!(e.ledger(args.day_epoch).unwrap().actions_count, 0);
}

#[test]
fn a_reject_may_not_ride_with_a_venue_leg() {
    let mut e = Env::ready();
    let owner = e.owner.pubkey();
    let args = decision_args(10, Decision::Reject, ok_checks());
    let r = send_decision(&mut e, true, &args, vec![venue_leg(&owner)]);
    assert!(err_contains(&r, "RejectFollowedByVenue"), "{r:?}");
}

#[test]
fn an_off_chain_rule_can_reject_even_when_every_on_chain_rule_passes() {
    let mut e = Env::ready();
    let mut args = decision_args(11, Decision::Reject, ok_checks());
    args.reason_code = reason::SLIPPAGE_LIMIT;
    send_decision(&mut e, true, &args, vec![]).unwrap();
    let r = e.receipt(11).unwrap();
    assert_eq!(r.reason_code, reason::SLIPPAGE_LIMIT);
    assert!(r.checks.iter().any(|c| c.rule == rule::SLIPPAGE));
}

// ---- invariant 3: pauses ---------------------------------------------------------

#[test]
fn a_paused_account_rejects_risk_increasing_decisions_with_11() {
    let mut e = Env::ready();
    let owner = e.owner.pubkey();
    e.owner_verb(markov::instruction::Pause {}).unwrap();
    // The caller says Allow and brings a venue leg: the program turns it into
    // a Reject, and a Reject cannot ride with a venue leg, so nothing lands.
    let r = send_decision(&mut e, true, &allow_args(12), vec![venue_leg(&owner)]);
    assert!(err_contains(&r, "RejectFollowedByVenue"), "{r:?}");
    // The honest form: a Reject with no venue leg lands with reason 11.
    let args = decision_args(13, Decision::Reject, ok_checks());
    send_decision(&mut e, true, &args, vec![]).unwrap();
    assert_eq!(e.receipt(13).unwrap().reason_code, reason::ACCOUNT_PAUSED);
    // Reduce is not risk-increasing and still goes through.
    let mut args = allow_args(14);
    args.kind = ReceiptKind::TradeReduce;
    send_decision(&mut e, true, &args, vec![venue_leg(&owner)]).unwrap();
    assert_eq!(e.receipt(14).unwrap().decision, Decision::Allow);
    e.owner_verb(markov::instruction::Unpause {}).unwrap();
    send_decision(&mut e, true, &allow_args(15), vec![venue_leg(&owner)]).unwrap();
}

#[test]
fn global_pause_stops_actor_flows_but_not_the_owner() {
    let mut e = Env::ready();
    let owner = e.owner.pubkey();
    let actor = e.actor.pubkey();
    e.set_permission(
        actor,
        scopes::TRADE_REQUEST | scopes::TRADE_REDUCE,
        1_000 * USD,
        5_000 * USD,
        0,
    )
    .unwrap();
    let ix = e.admin_ix(markov::instruction::SetGlobalPause { paused: true });
    let admin = e.admin.insecure_clone();
    e.send(vec![ix], &admin, &[&admin]).unwrap();
    // Owner-signed keeps working.
    send_decision(&mut e, true, &allow_args(16), vec![venue_leg(&owner)]).unwrap();
    // Actor gets reason 12 on the honest form.
    let args = decision_args(17, Decision::Reject, ok_checks());
    send_decision(&mut e, false, &args, vec![]).unwrap();
    assert_eq!(e.receipt(17).unwrap().reason_code, reason::GLOBAL_PAUSED);
}

// ---- invariant 4: permissions before mandate ------------------------------------

#[test]
fn actor_needs_scope_and_caps_and_daily_caps_roll() {
    let mut e = Env::ready();
    let owner = e.owner.pubkey();
    let actor = e.actor.pubkey();
    // No permission at all: the instruction cannot even resolve.
    let r = send_decision(
        &mut e,
        false,
        &decision_args(18, Decision::Reject, ok_checks()),
        vec![],
    );
    assert!(r.is_err());
    // Read-only scope: a trade decision is refused with 8, as a receipt.
    e.set_permission(actor, scopes::ACCOUNT_READ, 0, 0, 0)
        .unwrap();
    send_decision(
        &mut e,
        false,
        &decision_args(19, Decision::Reject, ok_checks()),
        vec![],
    )
    .unwrap();
    assert_eq!(
        e.receipt(19).unwrap().reason_code,
        reason::ACTOR_SCOPE_DENIED
    );
    // Trade scope with a $600 daily cap: first $500 passes, the second does not.
    e.set_permission(actor, scopes::TRADE_REQUEST, 600 * USD, 600 * USD, 0)
        .unwrap();
    send_decision(&mut e, false, &allow_args(20), vec![venue_leg(&owner)]).unwrap();
    assert_eq!(e.permission(&actor).spent_today_usd, 500 * USD);
    let r = send_decision(&mut e, false, &allow_args(21), vec![venue_leg(&owner)]);
    assert!(err_contains(&r, "RejectFollowedByVenue"), "{r:?}");
    send_decision(
        &mut e,
        false,
        &decision_args(22, Decision::Reject, ok_checks()),
        vec![],
    )
    .unwrap();
    assert_eq!(
        e.receipt(22).unwrap().reason_code,
        reason::ACTOR_CAP_EXCEEDED
    );
    // Next UTC day: the cap rolls.
    set_clock(&mut e.svm, NOW + 86_400, SLOT + 10);
    let mut args = allow_args(23);
    args.day_epoch = (NOW + 86_400).div_euclid(86_400);
    args.data_slot = SLOT + 10;
    send_decision(&mut e, false, &args, vec![venue_leg(&owner)]).unwrap();
    assert_eq!(e.permission(&actor).spent_today_usd, 500 * USD);
    // Revoked: hard error.
    let ix = e.ix(
        markov::accounts::RevokePermission {
            owner,
            account: e.account,
            permission: e.permission_pda(&actor),
        },
        markov::instruction::RevokePermission {},
    );
    let o = e.owner.insecure_clone();
    e.send(vec![ix], &o, &[&o]).unwrap();
    let mut args = decision_args(24, Decision::Reject, ok_checks());
    args.day_epoch = (NOW + 86_400).div_euclid(86_400);
    args.data_slot = SLOT + 10;
    let r = send_decision(&mut e, false, &args, vec![]);
    assert!(err_contains(&r, "PermissionRevoked"), "{r:?}");
    // And the wrong day epoch is its own, named refusal.
    args.request_id = 26;
    args.day_epoch -= 1;
    let r = send_decision(&mut e, false, &args, vec![]);
    assert!(err_contains(&r, "WrongDayEpoch"), "{r:?}");
}

#[test]
fn an_expired_permission_is_refused() {
    let mut e = Env::ready();
    let actor = e.actor.pubkey();
    e.set_permission(actor, scopes::TRADE_REQUEST, 0, 0, SLOT + 5)
        .unwrap();
    set_clock(&mut e.svm, NOW, SLOT + 5);
    let mut args = decision_args(25, Decision::Reject, ok_checks());
    args.data_slot = SLOT + 5;
    let r = send_decision(&mut e, false, &args, vec![]);
    assert!(err_contains(&r, "PermissionExpired"), "{r:?}");
}

// ---- invariant 6 & 8: replay and append-only receipts ---------------------------

#[test]
fn a_request_id_can_be_decided_once() {
    let mut e = Env::ready();
    let owner = e.owner.pubkey();
    send_decision(&mut e, true, &allow_args(30), vec![venue_leg(&owner)]).unwrap();
    let r = send_decision(&mut e, true, &allow_args(30), vec![venue_leg(&owner)]);
    assert!(
        r.is_err(),
        "invariant 6: second decision with the same id fails"
    );
}

#[test]
fn finalize_adds_two_fields_once_after_the_venue_leg() {
    let mut e = Env::ready();
    let owner = e.owner.pubkey();
    let args = allow_args(31);
    let fin = e.ix(
        markov::accounts::FinalizeDecision {
            signer: owner,
            account: e.account,
            receipt: e.receipt_pda(31),
            instructions: INSTRUCTIONS_SYSVAR,
            event_authority: e.event_authority,
            program: e.program,
        },
        markov::instruction::FinalizeDecision {
            post_state_hash: [9u8; 32],
            tx_signature_hint: [8u8; 32],
        },
    );
    send_decision(&mut e, true, &args, vec![venue_leg(&owner), fin.clone()]).unwrap();
    let r = e.receipt(31).unwrap();
    assert!(r.finalized);
    assert_eq!(r.post_state_hash, [9u8; 32]);
    assert_eq!(r.tx_signature_hint, [8u8; 32]);
    assert_eq!(r.pre_state_hash, [2u8; 32]);
    // A second finalize is refused; the receipt is unchanged.
    let o = e.owner.insecure_clone();
    let r2 = e.send(vec![venue_leg(&owner), fin], &o, &[&o]);
    assert!(r2.is_err());
    assert_eq!(e.receipt(31).unwrap().post_state_hash, [9u8; 32]);
    // A stranger cannot finalize.
    let s = e.stranger.insecure_clone();
    let fin_stranger = e.ix(
        markov::accounts::FinalizeDecision {
            signer: s.pubkey(),
            account: e.account,
            receipt: e.receipt_pda(31),
            instructions: INSTRUCTIONS_SYSVAR,
            event_authority: e.event_authority,
            program: e.program,
        },
        markov::instruction::FinalizeDecision {
            post_state_hash: [1u8; 32],
            tx_signature_hint: [1u8; 32],
        },
    );
    assert!(e
        .send(vec![venue_leg(&s.pubkey()), fin_stranger], &s, &[&s])
        .is_err());
}

#[test]
fn close_receipt_only_after_thirty_days() {
    let mut e = Env::ready();
    let owner = e.owner.pubkey();
    send_decision(&mut e, true, &allow_args(32), vec![venue_leg(&owner)]).unwrap();
    let close = e.ix(
        markov::accounts::CloseReceipt {
            owner,
            account: e.account,
            receipt: e.receipt_pda(32),
        },
        markov::instruction::CloseReceipt {},
    );
    let o = e.owner.insecure_clone();
    let r = e.send(vec![close.clone()], &o, &[&o]);
    assert!(r.is_err(), "too young");
    set_clock(&mut e.svm, NOW + 31 * 86_400, SLOT + 100);
    e.send(vec![close], &o, &[&o]).unwrap();
    assert!(e.receipt(32).is_none());
}

// ---- invariant 5: invest spend ---------------------------------------------------

fn invest_args(id: u128, mint: solana_pubkey::Pubkey, amount: u64) -> markov::InvestArgs {
    markov::InvestArgs {
        request_id: id,
        mint,
        usdc_amount: amount,
        quote_cost_bps: 20,
        official_price: 0,
        quote_price: 0,
        market_status: MarketStatus::Open,
        checks: vec![Check {
            rule: rule::INVEST_SINGLE_ASSET,
            observed: 1_000,
            limit: 5_000,
            pass: true,
        }],
        data_slot: SLOT,
        route_snapshot_hash: [3u8; 32],
        pre_state_hash: [4u8; 32],
    }
}

fn invest_ix(e: &Env, signer: &solana_pubkey::Pubkey, args: &markov::InvestArgs) -> Instruction {
    let is_owner = *signer == e.owner.pubkey();
    let metas = markov::accounts::InvestExecute {
        signer: *signer,
        config: e.config,
        account: e.account,
        invest_mandate: e.invest_pda(1),
        permission: if is_owner {
            None
        } else {
            Some(e.permission_pda(signer))
        },
        reserve_token_account: e.owner_usdc,
        receipt: e.receipt_pda(args.request_id),
        instructions: INSTRUCTIONS_SYSVAR,
        system_program: SYSTEM_PROGRAM,
        event_authority: e.event_authority,
        program: e.program,
    }
    .to_account_metas(None);
    Instruction {
        program_id: e.program,
        accounts: metas,
        data: markov::instruction::InvestExecute { args: args.clone() }.data(),
    }
}

fn send_invest(
    e: &mut Env,
    args: &markov::InvestArgs,
    trailing: Vec<Instruction>,
) -> Result<(), String> {
    let actor = e.actor.insecure_clone();
    let mut ixs = vec![invest_ix(e, &actor.pubkey(), args)];
    ixs.extend(trailing);
    e.send(ixs, &actor, &[&actor])
        .map(|_| ())
        .map_err(|m| meta_err(&m))
}

#[test]
fn invest_execute_charges_the_month_and_needs_collect_then_swap() {
    let mut e = Env::ready();
    let actor = e.actor.pubkey();
    let mint = e.usdc_mint;
    e.set_permission(actor, scopes::INVEST_EXECUTE, 100 * USD, 1_000 * USD, 0)
        .unwrap();
    let legs = vec![collect_leg(), swap_leg()];
    send_invest(&mut e, &invest_args(40, mint, 25 * USD), legs.clone()).unwrap();
    assert_eq!(e.invest_mandate(1).spent_this_month_usd, 25 * USD);
    let r = e.receipt(40).unwrap();
    assert_eq!(r.kind, ReceiptKind::InvestExecute);
    assert_eq!(r.decision, Decision::Allow);
    assert!(r
        .checks
        .iter()
        .any(|c| c.rule == rule::INVEST_RESERVE && c.pass && c.observed == 175 * USD as i64));
    // Missing the swap leg: refused.
    let r = send_invest(
        &mut e,
        &invest_args(41, mint, 25 * USD),
        vec![collect_leg()],
    );
    assert!(err_contains(&r, "MissingAdjacentInstruction"), "{r:?}");
    // Legs in the wrong order: refused.
    let r = send_invest(
        &mut e,
        &invest_args(42, mint, 25 * USD),
        vec![swap_leg(), collect_leg()],
    );
    assert!(err_contains(&r, "AdjacentProgramNotAllowed"), "{r:?}");
    // Over the per-period budget: refused (the keeper must record_skip instead).
    let r = send_invest(&mut e, &invest_args(43, mint, 30 * USD), legs.clone());
    assert!(err_contains(&r, "AllowViolatesMandate"), "{r:?}");
    // Spend up to the ceiling: 25 + 25 + 25 + 25 = 100 fits; the fifth does not.
    for id in 44..47u128 {
        send_invest(&mut e, &invest_args(id, mint, 25 * USD), legs.clone()).unwrap();
    }
    assert_eq!(e.invest_mandate(1).spent_this_month_usd, 100 * USD);
    let r = send_invest(&mut e, &invest_args(47, mint, 25 * USD), legs.clone());
    assert!(err_contains(&r, "AllowViolatesMandate"), "{r:?}");
    // Next month: the counter resets.
    set_clock(&mut e.svm, NOW + 20 * 86_400, SLOT + 1_000);
    let mut a = invest_args(48, mint, 25 * USD);
    a.data_slot = SLOT + 1_000;
    send_invest(&mut e, &a, legs).unwrap();
    assert_eq!(e.invest_mandate(1).spent_this_month_usd, 25 * USD);
}

#[test]
fn invest_reserve_floor_reads_the_token_account_not_a_claim() {
    let mut e = Env::ready();
    let actor = e.actor.pubkey();
    let mint = e.usdc_mint;
    e.set_permission(actor, scopes::INVEST_EXECUTE, 0, 0, 0)
        .unwrap();
    // Owner holds 200; floor is 50; a 25 buy leaves 175: fine. Make the floor 190.
    let mut f = invest_fields(mint);
    f.reserve_floor_usd = 190 * USD;
    e.set_invest_mandate(2, f).unwrap();
    let mut args = invest_args(50, mint, 25 * USD);
    args.checks = vec![];
    let mut ix = invest_ix(&e, &actor, &args);
    // Point at version 2.
    for m in ix.accounts.iter_mut() {
        if m.pubkey == e.invest_pda(1) {
            m.pubkey = e.invest_pda(2);
        }
    }
    let a = e.actor.insecure_clone();
    let r = e
        .send(vec![ix, collect_leg(), swap_leg()], &a, &[&a])
        .map(|_| ())
        .map_err(|m| meta_err(&m));
    assert!(err_contains(&r, "AllowViolatesMandate"), "{r:?}");
}

#[test]
fn invest_execute_without_the_scope_is_refused_and_a_skip_lands() {
    let mut e = Env::ready();
    let actor = e.actor.pubkey();
    let mint = e.usdc_mint;
    e.set_permission(actor, scopes::INVEST_PROPOSE, 0, 0, 0)
        .unwrap();
    let r = send_invest(
        &mut e,
        &invest_args(60, mint, 25 * USD),
        vec![collect_leg(), swap_leg()],
    );
    assert!(err_contains(&r, "AllowViolatesMandate"), "{r:?}");
    // A skip from an INVEST_EXECUTE actor lands with its reason.
    e.set_permission(actor, scopes::INVEST_EXECUTE, 0, 0, 0)
        .unwrap();
    let skip = markov::SkipArgs {
        request_id: 61,
        reason_code: reason::DELEGATION_INSUFFICIENT,
        checks: vec![],
        data_slot: SLOT,
    };
    let metas = markov::accounts::RecordSkip {
        signer: actor,
        config: e.config,
        account: e.account,
        permission: Some(e.permission_pda(&actor)),
        receipt: e.receipt_pda(61),
        instructions: INSTRUCTIONS_SYSVAR,
        system_program: SYSTEM_PROGRAM,
        event_authority: e.event_authority,
        program: e.program,
    }
    .to_account_metas(None);
    let ix = Instruction {
        program_id: e.program,
        accounts: metas,
        data: markov::instruction::RecordSkip { args: skip }.data(),
    };
    let a = e.actor.insecure_clone();
    e.send(vec![ix.clone()], &a, &[&a]).unwrap();
    let r = e.receipt(61).unwrap();
    assert_eq!(r.decision, Decision::Skip);
    assert_eq!(r.reason_code, reason::DELEGATION_INSUFFICIENT);
    // A skip cannot ride with the collect leg.
    let skip2 = markov::SkipArgs {
        request_id: 62,
        reason_code: reason::BUDGET_EXHAUSTED,
        checks: vec![],
        data_slot: SLOT,
    };
    let mut ix2 = ix;
    ix2.data = markov::instruction::RecordSkip { args: skip2 }.data();
    for m in ix2.accounts.iter_mut() {
        if m.pubkey == e.receipt_pda(61) {
            m.pubkey = e.receipt_pda(62);
        }
    }
    let r = e
        .send(vec![ix2, collect_leg()], &a, &[&a])
        .map(|_| ())
        .map_err(|m| meta_err(&m));
    assert!(err_contains(&r, "RejectFollowedByVenue"), "{r:?}");
}
