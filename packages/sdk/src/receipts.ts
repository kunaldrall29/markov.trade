/**
 * Receipts as they land on chain.
 *
 * `emit_cpi!` writes each receipt as a self-CPI whose instruction data is the
 * event instruction tag, the event discriminator, then borsh — it is *inner
 * instruction data*, not a log line, so it survives log truncation and is
 * decoded by IDL here, never by string matching. The same bytes the agent's
 * submitter reads (`crates/markov-chain`) are what this module reads.
 */
import { getBase58Encoder, type Address, type Signature } from "@solana/kit";
import {
  ACTION_RECEIPT_EVENT_DISCRIMINATOR,
  BlockReason,
  getActionReceiptEventDecoder,
  getOwnerActionEventDecoder,
  getRefusalReceiptEventDecoder,
  OWNER_ACTION_EVENT_DISCRIMINATOR,
  OwnerActionKind,
  REFUSAL_RECEIPT_EVENT_DISCRIMINATOR,
  type ActionReceiptEvent,
  type OwnerActionEvent,
  type RefusalReceiptEvent,
} from "./generated/mandate";
import { CLUSTER } from "./facts";
import { bytesToId } from "./format";

/** `anchor_lang::event::EVENT_IX_TAG_LE` — `0x1d9acb512ea545e4` little-endian. */
export const EVENT_IX_TAG_LE = new Uint8Array([0xe4, 0x45, 0xa5, 0x2e, 0x51, 0xcb, 0x9a, 0x1d]);

export const ACTION_NAMES = ["skip", "open", "increase", "reduce", "close", "flatten"] as const;
export const SIDE_NAMES = ["long", "short"] as const;

/** Exactly `BlockReason::name()` in `crates/markov-types`, verbatim, append-only. */
export const BLOCK_REASON_NAMES = [
  "OverTxCap",
  "OverDailyCap",
  "OverSpendCap",
  "OverSpendDailyCap",
  "ProgramNotAllowed",
  "TokenNotAllowed",
  "SlippageExceeded",
  "Expired",
  "Paused",
  "Revoked",
  "Unauthorized",
  "StaleOracle",
  "ActionNotAllowed",
  "DuplicateIntent",
  "GlobalHalt",
  "VenueRejected",
  "PostCheckFailed",
  "ControlledAccountForwarded",
] as const;

export const OWNER_ACTION_NAMES = ["Create", "Fund", "Amend", "Pause", "Unpause", "Revoke", "Withdraw", "Close"] as const;

export function blockReasonName(reason: BlockReason | number): string {
  return BLOCK_REASON_NAMES[reason] ?? `Unknown(${reason})`;
}
export function actionName(action: number): string {
  return ACTION_NAMES[action] ?? `unknown(${action})`;
}
export function sideName(side: number): string {
  return SIDE_NAMES[side] ?? `unknown(${side})`;
}
export function ownerActionName(kind: OwnerActionKind | number): string {
  return OWNER_ACTION_NAMES[kind] ?? `Unknown(${kind})`;
}

export type ReceiptBase = {
  signature: Signature;
  /** Position of the event among this transaction's inner instructions. */
  eventIndex: number;
  slot: bigint;
  blockTime: number | null;
  /** The transaction landed but the runtime reported an error; a receipt inside it did not commit. */
  txError: boolean;
};

export type Receipt = ReceiptBase & DecodedReceipt;

function startsWith(data: Uint8Array, prefix: Uint8Array, offset = 0): boolean {
  if (data.length < offset + prefix.length) return false;
  for (let i = 0; i < prefix.length; i += 1) if (data[offset + i] !== prefix[i]) return false;
  return true;
}

export type DecodedReceipt =
  | { kind: "action"; event: ActionReceiptEvent }
  | { kind: "refusal"; event: RefusalReceiptEvent }
  | { kind: "owner"; event: OwnerActionEvent };

/** Decode one inner-instruction payload; `null` if it is not a Markov receipt. */
export function decodeReceiptData(data: Uint8Array): DecodedReceipt | null {
  if (!startsWith(data, EVENT_IX_TAG_LE)) return null;
  const body = data.subarray(EVENT_IX_TAG_LE.length);
  if (startsWith(body, ACTION_RECEIPT_EVENT_DISCRIMINATOR as Uint8Array)) {
    return { kind: "action", event: getActionReceiptEventDecoder().decode(body) };
  }
  if (startsWith(body, REFUSAL_RECEIPT_EVENT_DISCRIMINATOR as Uint8Array)) {
    return { kind: "refusal", event: getRefusalReceiptEventDecoder().decode(body) };
  }
  if (startsWith(body, OWNER_ACTION_EVENT_DISCRIMINATOR as Uint8Array)) {
    return { kind: "owner", event: getOwnerActionEventDecoder().decode(body) };
  }
  return null;
}

/** The subset of an RPC `getTransaction` (json encoding) result this decoder needs. */
export type TransactionLike = {
  slot: bigint | number;
  blockTime?: bigint | number | null;
  meta: {
    err: unknown;
    innerInstructions?: readonly { index: number; instructions: readonly { programIdIndex: number; data: string }[] }[] | null;
  } | null;
  transaction: { message: { accountKeys: readonly (string | { pubkey: string })[] } };
};

const base58 = getBase58Encoder();

/** Every receipt in one landed transaction, in emission order. */
export function receiptsFromTransaction(signature: Signature, tx: TransactionLike, programId: Address): Receipt[] {
  const out: Receipt[] = [];
  if (!tx.meta) return out;
  const keys = tx.transaction.message.accountKeys.map((k) => (typeof k === "string" ? k : k.pubkey));
  let eventIndex = 0;
  for (const group of tx.meta.innerInstructions ?? []) {
    for (const ix of group.instructions) {
      if (keys[ix.programIdIndex] !== programId) continue;
      const data = base58.encode(ix.data) as Uint8Array;
      const decoded = decodeReceiptData(data);
      if (!decoded) continue;
      out.push({
        ...decoded,
        signature,
        eventIndex: eventIndex++,
        slot: BigInt(tx.slot),
        blockTime: tx.blockTime == null ? null : Number(tx.blockTime),
        txError: tx.meta.err != null,
      } as Receipt);
    }
  }
  return out;
}

/** Plain-words summary for a row: what happened and what the program said. */
export function receiptSummary(r: Receipt): { label: string; verdict: "allowed" | "refused" | "owner"; detail: string } {
  if (r.kind === "action") {
    const e = r.event;
    return {
      label: `${actionName(e.action)} ${bytesToId(e.market)} ${sideName(e.side)}`,
      verdict: "allowed",
      detail: `fill ${e.fillPrice} · fee ${e.fee} · mark ${e.markPrice}`,
    };
  }
  if (r.kind === "refusal") {
    const e = r.event;
    return { label: `${actionName(e.action)}`, verdict: "refused", detail: `${blockReasonName(e.reason)} · gate ${e.gateIndex}${e.forced ? " · forced" : ""}` };
  }
  const e = r.event;
  return { label: ownerActionName(e.kind), verdict: "owner", detail: e.amount > 0n ? `amount ${e.amount}` : "" };
}

export function explorerTx(signature: string, cluster: string = CLUSTER): string {
  return `https://explorer.solana.com/tx/${signature}?cluster=${cluster}`;
}
export function explorerAccount(address: string, cluster: string = CLUSTER): string {
  return `https://explorer.solana.com/address/${address}?cluster=${cluster}`;
}
