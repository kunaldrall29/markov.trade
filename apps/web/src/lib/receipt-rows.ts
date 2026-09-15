/** SDK receipts → wire rows. Shared by the read API and the owner's page. */
import { USDC_D_DECIMALS, actionName, blockReasonName, bytesToId, explorerTx, ownerActionName, sideName, type Receipt } from "@markov/sdk";
import type { Money, ReceiptRow } from "./api-types";

export function money(raw: bigint, mint: string, decimals = USDC_D_DECIMALS): Money {
  return { raw: raw.toString(), decimals, mint };
}

export function serializeReceipt(r: Receipt, settlementMint = "settlement"): ReceiptRow {
  const base = {
    signature: r.signature,
    eventIndex: r.eventIndex,
    slot: r.slot.toString(),
    blockTime: r.blockTime,
    txError: r.txError,
    explorer: explorerTx(r.signature),
    seq: null,
    action: null,
    side: null,
    market: null,
    notional: null,
    fillPrice: null,
    fee: null,
    markPrice: null,
    markPublishTime: null,
    reason: null,
    reasonByte: null,
    gateIndex: null,
    forced: null,
    ownerKind: null,
    amount: null,
  };
  if (r.kind === "action") {
    const e = r.event;
    return {
      ...base,
      kind: "action",
      mandate: e.mandate,
      actor: e.operator,
      seq: e.seq.toString(),
      action: actionName(e.action),
      side: sideName(e.side),
      market: bytesToId(e.market),
      notional: money(e.notional, settlementMint),
      fillPrice: e.fillPrice.toString(),
      fee: e.fee.toString(),
      markPrice: e.markPrice.toString(),
      markPublishTime: Number(e.markPublishTime),
      forced: e.forced,
    };
  }
  if (r.kind === "refusal") {
    const e = r.event;
    return {
      ...base,
      kind: "refusal",
      mandate: e.mandate,
      actor: e.operator,
      seq: e.seq.toString(),
      action: actionName(e.action),
      notional: money(e.notional, settlementMint),
      reason: blockReasonName(e.reason),
      reasonByte: e.reason,
      gateIndex: e.gateIndex,
      forced: e.forced,
    };
  }
  const e = r.event;
  return { ...base, kind: "owner", mandate: e.mandate, actor: e.actor, ownerKind: ownerActionName(e.kind), amount: money(e.amount, e.mint) };
}
