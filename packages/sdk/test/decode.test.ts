/**
 * Decoders against bytes the program crate itself serialised
 * (`programs/markov-mandate/examples/ts_fixtures.rs`). Synthetic values,
 * real layouts: if a field moves in Rust, these fail here.
 */
import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { getMandateDecoder, getRegistryDecoder, MandateState, MANDATE_DISCRIMINATOR } from "../src/generated/mandate";
import { getMarkAccountDecoder, getMarketDecoder, getPositionDecoder, MarkSourceKind, Side } from "../src/generated/demoPerps";
import { getFundInstructionDataEncoder, getOwnerWithdrawInstructionDataEncoder, getPauseInstructionDataEncoder, getRevokeInstructionDataEncoder, getUnpauseInstructionDataEncoder, getCloseMandateInstructionDataEncoder } from "../src/generated/mandate";
import { blockReasonName, decodeReceiptData, EVENT_IX_TAG_LE, receiptsFromTransaction, BLOCK_REASON_NAMES } from "../src/receipts";
import { bytesToHex, bytesToId, hexToBytes } from "../src/format";
import { priceE6 } from "../src/reader";
import { getBase58Decoder } from "@solana/kit";

const fx = JSON.parse(readFileSync(join(__dirname, "fixtures", "program.json"), "utf8"));

describe("program fixtures", () => {
  it("event instruction tag matches anchor_lang::event::EVENT_IX_TAG_LE", () => {
    expect(bytesToHex(EVENT_IX_TAG_LE)).toBe(fx.event_ix_tag_le);
  });

  it("discriminators match the crate", () => {
    expect(bytesToHex(MANDATE_DISCRIMINATOR)).toBe(fx.discriminators.Mandate);
  });

  it("decodes a Mandate account", () => {
    const bytes = hexToBytes(fx.accounts.mandate.hex);
    expect(bytesToHex(bytes.subarray(0, 8))).toBe(fx.discriminators.Mandate);
    const m = getMandateDecoder().decode(bytes);
    expect(m.owner).toBe(fx.accounts.mandate.owner);
    expect(m.operator).toBe(fx.accounts.mandate.operator);
    expect(m.state).toBe(MandateState.Paused);
    expect(m.policy.perTxCap).toBe(BigInt(fx.accounts.mandate.per_tx_cap));
    expect(m.policy.dailyCap).toBe(BigInt(fx.accounts.mandate.daily_cap));
    expect(m.policy.maxSlippageBps).toBe(fx.accounts.mandate.max_slippage_bps);
    expect(m.policy.expiryTs).toBe(BigInt(fx.accounts.mandate.expiry_ts));
    expect(m.policy.venues[0]).toBe(fx.accounts.mandate.venue);
    expect(m.policy.venuesLen).toBe(1);
    expect(m.actionSeq).toBe(BigInt(fx.accounts.mandate.action_seq));
    expect(m.dayNotionalUsed).toBe(BigInt(fx.accounts.mandate.day_notional_used));
    expect(m.nonce).toBe(BigInt(fx.accounts.mandate.nonce));
    expect(m.vault).toBe(fx.accounts.mandate.vault);
    expect(m.mint).toBe(fx.accounts.mandate.mint);
    expect(bytesToId(m.strategyId)).toBe("BOOK_ONE");
  });

  it("decodes a Registry account", () => {
    const r = getRegistryDecoder().decode(hexToBytes(fx.accounts.registry.hex));
    expect(r.admin).toBe(fx.accounts.registry.admin);
    expect(r.globalHalt).toBe(false);
    expect(r.adaptersLen).toBe(1);
    expect(r.adapters[0]).toBe(fx.accounts.registry.adapters[0]);
  });

  it("decodes demo_perps Position, MarkAccount and Market", () => {
    const p = getPositionDecoder().decode(hexToBytes(fx.accounts.position.hex));
    expect(p.side).toBe(Side.Long);
    expect(p.notional).toBe(BigInt(fx.accounts.position.notional));
    expect(p.entryPrice).toBe(BigInt(fx.accounts.position.entry_price));
    expect(p.fundingAccrued).toBe(BigInt(fx.accounts.position.funding_accrued));
    const k = getMarkAccountDecoder().decode(hexToBytes(fx.accounts.mark.hex));
    expect(k.price).toBe(BigInt(fx.accounts.mark.price));
    expect(k.expo).toBe(fx.accounts.mark.expo);
    expect(k.source).toBe(MarkSourceKind.Pyth);
    expect(priceE6(k.price, k.expo)).toBe(BigInt(fx.accounts.mark.price_e6));
    const m = getMarketDecoder().decode(hexToBytes(fx.accounts.market.hex));
    expect(m.feeBps).toBe(fx.accounts.market.fee_bps);
    expect(m.positionCap).toBe(BigInt(fx.accounts.market.position_cap));
    expect(m.paused).toBe(false);
  });

  it("decodes an ActionReceipt from emit_cpi instruction data", () => {
    const r = decodeReceiptData(hexToBytes(fx.events.action_receipt.hex));
    expect(r?.kind).toBe("action");
    if (r?.kind !== "action") throw new Error("unreachable");
    expect(r.event.seq).toBe(BigInt(fx.events.action_receipt.seq));
    expect(r.event.fillPrice).toBe(BigInt(fx.events.action_receipt.fill_price));
    expect(r.event.fee).toBe(BigInt(fx.events.action_receipt.fee));
    expect(r.event.markPrice).toBe(BigInt(fx.events.action_receipt.mark_price));
    expect(r.event.slot).toBe(BigInt(fx.events.action_receipt.slot));
    expect(bytesToId(r.event.market)).toBe("SOL-PERP");
  });

  it("decodes RefusalReceipts, including reason 17 which the older IDL snapshot lacked", () => {
    const r = decodeReceiptData(hexToBytes(fx.events.refusal_receipt.hex));
    if (r?.kind !== "refusal") throw new Error("expected refusal");
    expect(r.event.reason).toBe(fx.events.refusal_receipt.reason_byte);
    expect(blockReasonName(r.event.reason)).toBe("OverTxCap");
    expect(r.event.gateIndex).toBe(8);
    expect(r.event.forced).toBe(true);
    const r17 = decodeReceiptData(hexToBytes(fx.events.refusal_receipt_17.hex));
    if (r17?.kind !== "refusal") throw new Error("expected refusal");
    expect(r17.event.reason).toBe(17);
    expect(blockReasonName(r17.event.reason)).toBe("ControlledAccountForwarded");
    expect(r17.event.gateIndex).toBe(15);
  });

  it("decodes an OwnerAction", () => {
    const r = decodeReceiptData(hexToBytes(fx.events.owner_action.hex));
    if (r?.kind !== "owner") throw new Error("expected owner");
    expect(r.event.amount).toBe(BigInt(fx.events.owner_action.amount));
    expect(r.event.kind).toBe(6); // Withdraw
  });

  it("returns null for non-receipt data", () => {
    expect(decodeReceiptData(new Uint8Array([1, 2, 3]))).toBeNull();
    expect(decodeReceiptData(hexToBytes(fx.instruction_data.fund.hex))).toBeNull();
  });

  it("encodes owner verb instruction data exactly as the crate does", () => {
    expect(bytesToHex(getFundInstructionDataEncoder().encode({ amount: BigInt(fx.instruction_data.fund.amount) }))).toBe(fx.instruction_data.fund.hex);
    expect(bytesToHex(getOwnerWithdrawInstructionDataEncoder().encode({ amount: BigInt(fx.instruction_data.owner_withdraw.amount) }))).toBe(fx.instruction_data.owner_withdraw.hex);
    expect(bytesToHex(getPauseInstructionDataEncoder().encode({}))).toBe(fx.instruction_data.pause.hex);
    expect(bytesToHex(getUnpauseInstructionDataEncoder().encode({}))).toBe(fx.instruction_data.unpause.hex);
    expect(bytesToHex(getRevokeInstructionDataEncoder().encode({}))).toBe(fx.instruction_data.revoke.hex);
    expect(bytesToHex(getCloseMandateInstructionDataEncoder().encode({}))).toBe(fx.instruction_data.close_mandate.hex);
  });

  it("BlockReason names are append-only and dense", () => {
    expect(BLOCK_REASON_NAMES.length).toBe(18);
    expect(BLOCK_REASON_NAMES[0]).toBe("OverTxCap");
    expect(BLOCK_REASON_NAMES[10]).toBe("Unauthorized");
    expect(BLOCK_REASON_NAMES[17]).toBe("ControlledAccountForwarded");
  });

  it("extracts receipts from a json-encoded transaction shape", () => {
    const b58 = getBase58Decoder();
    const data = b58.decode(hexToBytes(fx.events.refusal_receipt.hex));
    const tx = {
      slot: 492700100,
      blockTime: 1756900020,
      meta: { err: null, innerInstructions: [{ index: 0, instructions: [{ programIdIndex: 3, data }] }] },
      transaction: { message: { accountKeys: ["a", "b", "c", fx.program_id] } },
    };
    const rs = receiptsFromTransaction("sig" as never, tx, fx.program_id);
    expect(rs).toHaveLength(1);
    expect(rs[0]?.kind).toBe("refusal");
    expect(rs[0]?.slot).toBe(492700100n);
    // The same payload under a different program id is not a Markov receipt.
    expect(receiptsFromTransaction("sig" as never, tx, "b" as never)).toHaveLength(0);
  });
});
