//! Cross-language fixtures for `packages/sdk`.
//!
//! Serialises real program accounts and receipts with the program crate's own
//! Anchor codecs — the same code the deployed binary runs — so the TypeScript
//! decoders are tested against bytes the program would actually write, not
//! against a second hand-written encoder. `pnpm --filter @markov/sdk fixtures`
//! regenerates `packages/sdk/test/fixtures/program.json`; the values inside
//! are labelled by what they are, not recorded from a slot, and the SDK's
//! tests say so.
//!
//! Host only. Never deployed, touches no network, reads no key.
use anchor_lang::prelude::*;
use anchor_lang::Discriminator;
use markov_mandate::gates::Intent;
use markov_mandate::receipts::{ActionReceipt, OwnerAction, OwnerActionKind, RefusalReceipt};
use markov_mandate::state::{action_bits, Mandate, MandateState, Policy, Registry};
use markov_types::{ActionKind, BlockReason, MarkSourceKind, Side};

fn pk(n: u8) -> Pubkey {
    Pubkey::new_from_array([n; 32])
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// An account as the runtime stores it: 8-byte discriminator, then borsh.
fn account<T: AnchorSerialize + Discriminator>(v: &T) -> String {
    let mut out = T::DISCRIMINATOR.to_vec();
    v.serialize(&mut out).expect("serialize");
    hex(&out)
}

/// An `emit_cpi!` receipt as it lands in the self-CPI's instruction data:
/// the event instruction tag, the event discriminator, then borsh.
fn event<T: AnchorSerialize + Discriminator>(v: &T) -> String {
    let mut out = anchor_lang::event::EVENT_IX_TAG_LE.to_vec();
    out.extend_from_slice(T::DISCRIMINATOR);
    v.serialize(&mut out).expect("serialize");
    hex(&out)
}

fn instruction_data<T: AnchorSerialize + anchor_lang::InstructionData>(v: &T) -> String {
    hex(&v.data())
}

fn main() {
    let owner = pk(1);
    let operator = pk(2);
    let emergency = pk(3);
    let venue = pk(4);
    let mint = pk(5);
    let vault = pk(6);
    let mark_account = pk(7);
    let mandate_key = pk(8);
    let strategy_id = markov_mandate::BOOK_ONE;
    let market: [u8; 16] = *b"SOL-PERP\0\0\0\0\0\0\0\0";

    let mut policy = Policy {
        venues: [Pubkey::default(); 4],
        venues_len: 1,
        tokens: [Pubkey::default(); 4],
        tokens_len: 1,
        allowed_actions: action_bits::ALL,
        per_tx_cap: 50_000_000,
        daily_cap: 200_000_000,
        spend_per_call: 1_000_000,
        spend_daily: 5_000_000,
        max_slippage_bps: 50,
        max_mark_age_secs: 150,
        expiry_ts: 1_760_000_000,
    };
    policy.venues[0] = venue;
    policy.tokens[0] = mint;

    let mandate = Mandate {
        owner,
        operator,
        emergency,
        strategy_id,
        state: MandateState::Paused,
        policy,
        vault,
        mint,
        mark_account,
        feed_id: [0xef; 32],
        day_epoch: 20_700,
        day_notional_used: 25_000_000,
        day_spend_used: 1_000_000,
        action_seq: 3,
        recent_intents: [[0u8; 32]; markov_mandate::state::RECENT_INTENTS],
        recent_intents_len: 0,
        recent_intents_next: 0,
        created_at: 1_757_000_000,
        nonce: 10,
        bump: 254,
        vault_bump: 253,
        reserve: [0u8; 128],
    };

    let mut registry = Registry {
        admin: pk(9),
        global_halt: false,
        adapters: [Pubkey::default(); markov_mandate::state::MAX_ADAPTERS],
        adapters_len: 1,
        bump: 255,
    };
    registry.adapters[0] = venue;

    let action = ActionReceipt {
        seq: 4,
        intent_id: [0xaa; 32],
        mandate: mandate_key,
        owner,
        operator,
        strategy_id,
        venue,
        market,
        action: ActionKind::Reduce as u8,
        side: Side::Long as u8,
        notional: 25_000_000,
        fill_price: 104_724_049,
        fee: 25_000,
        mark_price: 104_619_430,
        mark_publish_time: 1_756_900_000,
        spend: 1_000_000,
        forced: false,
        ts: 1_756_900_010,
        slot: 492_700_000,
        net_delta_usd_e6: 0,
        gross_usd_e6: 0,
    };

    let refusal = RefusalReceipt {
        seq: 5,
        intent_id: [0xbb; 32],
        mandate: mandate_key,
        operator,
        strategy_id,
        venue,
        action: ActionKind::Increase as u8,
        notional: 51_000_000,
        reason: BlockReason::OverTxCap,
        gate_index: 8,
        forced: true,
        ts: 1_756_900_020,
        slot: 492_700_100,
    };

    let refusal_17 = RefusalReceipt {
        reason: BlockReason::ControlledAccountForwarded,
        gate_index: 15,
        seq: 6,
        ..refusal
    };

    let owner_action = OwnerAction {
        kind: OwnerActionKind::Withdraw,
        mandate: mandate_key,
        owner,
        actor: owner,
        strategy_id,
        mint,
        amount: 40_000_000,
        ts: 1_756_900_030,
        slot: 492_700_200,
    };

    let intent = Intent {
        intent_id: [0xcc; 32],
        action: ActionKind::Open,
        market,
        notional: 10_000_000,
        side: Side::Short,
        limit_price: 100_000_000,
        max_slippage_bps: 25,
        spend: 500_000,
        forced: false,
    };

    let position = demo_perps::Position {
        mandate: mandate_key,
        market_id: market,
        side: Side::Long,
        notional: 15_000_000,
        entry_price: 104_724_049,
        funding_accrued: -120,
        updated_slot: 492_700_000,
        bump: 250,
    };
    let mark = demo_perps::MarkAccount {
        market_id: market,
        price: 10_461_943_000,
        expo: -8,
        publish_time: 1_756_900_000,
        slot: 492_699_990,
        source: MarkSourceKind::Pyth,
        poster: pk(11),
        bump: 249,
    };
    let market_acc = demo_perps::Market {
        authority: pk(9),
        market_id: market,
        base_decimals: 9,
        mark: pk(12),
        fee_bps: 10,
        max_age_slots: 900,
        position_cap: 1_000_000_000,
        paused: false,
        bump: 248,
    };

    let fund_ix = markov_mandate::instruction::Fund { amount: 7_000_000 };
    let withdraw_ix = markov_mandate::instruction::OwnerWithdraw { amount: 8_000_000 };
    let create_ix = markov_mandate::instruction::CreateMandate {
        args: markov_mandate::CreateMandateArgs {
            operator,
            emergency,
            strategy_id,
            nonce: 10,
            policy,
            mark_account,
            feed_id: [0xef; 32],
        },
    };
    let amend_ix = markov_mandate::instruction::AmendPolicy { new_policy: policy };

    let mut intent_bytes = Vec::new();
    intent.serialize(&mut intent_bytes).expect("serialize");
    let intent_hex = hex(&intent_bytes);

    let out = serde_json::json!({
        "note": "Generated by programs/markov-mandate/examples/ts_fixtures.rs with the program crate's own Anchor codecs. Synthetic values, real layouts.",
        "program_id": markov_mandate::ID.to_string(),
        "venue_program_id": demo_perps::ID.to_string(),
        "event_ix_tag_le": hex(anchor_lang::event::EVENT_IX_TAG_LE),
        "discriminators": {
            "Mandate": hex(Mandate::DISCRIMINATOR),
            "Registry": hex(Registry::DISCRIMINATOR),
            "ActionReceipt": hex(ActionReceipt::DISCRIMINATOR),
            "RefusalReceipt": hex(RefusalReceipt::DISCRIMINATOR),
            "OwnerAction": hex(OwnerAction::DISCRIMINATOR),
            "Position": hex(demo_perps::Position::DISCRIMINATOR),
            "MarkAccount": hex(demo_perps::MarkAccount::DISCRIMINATOR),
            "Market": hex(demo_perps::Market::DISCRIMINATOR),
        },
        "accounts": {
            "mandate": { "hex": account(&mandate), "owner": owner.to_string(), "operator": operator.to_string(),
                "state": "Paused", "per_tx_cap": 50_000_000u64, "daily_cap": 200_000_000u64, "action_seq": 3u64,
                "nonce": 10u64, "vault": vault.to_string(), "mint": mint.to_string(), "expiry_ts": 1_760_000_000i64,
                "day_notional_used": 25_000_000u64, "venue": venue.to_string(), "max_slippage_bps": 50u16 },
            "registry": { "hex": account(&registry), "admin": pk(9).to_string(), "global_halt": false, "adapters": [venue.to_string()] },
            "position": { "hex": account(&position), "side": "Long", "notional": 15_000_000u64, "entry_price": 104_724_049u64, "funding_accrued": -120i64 },
            "mark": { "hex": account(&mark), "price": 10_461_943_000i64, "expo": -8, "publish_time": 1_756_900_000i64, "slot": 492_699_990u64, "source": "pyth", "price_e6": 104_619_430u64 },
            "market": { "hex": account(&market_acc), "fee_bps": 10u16, "position_cap": 1_000_000_000u64, "paused": false },
        },
        "events": {
            "action_receipt": { "hex": event(&action), "seq": 4u64, "action": "reduce", "side": "long", "notional": 25_000_000u64, "fill_price": 104_724_049u64, "fee": 25_000u64, "mark_price": 104_619_430u64, "slot": 492_700_000u64, "market": "SOL-PERP" },
            "refusal_receipt": { "hex": event(&refusal), "seq": 5u64, "reason": "OverTxCap", "reason_byte": 0u8, "gate_index": 8u8, "forced": true, "notional": 51_000_000u64 },
            "refusal_receipt_17": { "hex": event(&refusal_17), "seq": 6u64, "reason": "ControlledAccountForwarded", "reason_byte": 17u8, "gate_index": 15u8 },
            "owner_action": { "hex": event(&owner_action), "kind": "Withdraw", "amount": 40_000_000u64, "slot": 492_700_200u64 },
        },
        "instruction_data": {
            "fund": { "hex": instruction_data(&fund_ix), "amount": 7_000_000u64 },
            "owner_withdraw": { "hex": instruction_data(&withdraw_ix), "amount": 8_000_000u64 },
            "create_mandate": { "hex": instruction_data(&create_ix), "nonce": 10u64 },
            "amend_policy": { "hex": instruction_data(&amend_ix) },
            "pause": { "hex": instruction_data(&markov_mandate::instruction::Pause {}) },
            "unpause": { "hex": instruction_data(&markov_mandate::instruction::Unpause {}) },
            "revoke": { "hex": instruction_data(&markov_mandate::instruction::Revoke {}) },
            "close_mandate": { "hex": instruction_data(&markov_mandate::instruction::CloseMandate {}) },
        },
        "intent": { "borsh_hex": intent_hex, "notional": 10_000_000u64 },
        "strategy_id_hex": hex(&strategy_id),
        "market_id_hex": hex(&market),
    });
    println!("{}", serde_json::to_string_pretty(&out).expect("json"));
}
