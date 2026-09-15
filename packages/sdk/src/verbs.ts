/**
 * Owner verbs as ready-to-sign instructions. Every account is derived here
 * from the mandate and the owner; the caller supplies only the signer and
 * the amounts. Nothing here signs or sends — the wallet does that.
 */
import { type Address, type Instruction, type TransactionSigner } from "@solana/kit";
import { findAssociatedTokenPda, getCreateAssociatedTokenIdempotentInstruction, TOKEN_PROGRAM_ADDRESS } from "@solana-program/token";
import {
  getAmendPolicyInstruction,
  getCloseMandateInstruction,
  getCreateMandateInstruction,
  getFundInstruction,
  getOwnerWithdrawInstruction,
  getPauseInstruction,
  getRevokeInstruction,
  getUnpauseInstruction,
  type Policy,
  type PolicyArgs,
} from "./generated/mandate";
import { MANDATE_PROGRAM_ID, PYTH_SOL_USD_PRICE_UPDATE, SOL_USD_FEED_ID_HEX } from "./facts";
import { hexToBytes, idToBytes } from "./format";
import { deriveEventAuthority, deriveMandate, deriveVault } from "./pdas";

export type MandateRef = { mandate: Address; vault: Address; mint: Address };

async function eventAccounts() {
  return { eventAuthority: await deriveEventAuthority(), program: MANDATE_PROGRAM_ID };
}

export async function buildFund(owner: TransactionSigner, ref: MandateRef, amount: bigint): Promise<Instruction[]> {
  const [ata] = await findAssociatedTokenPda({ owner: owner.address, mint: ref.mint, tokenProgram: TOKEN_PROGRAM_ADDRESS });
  return [
    getFundInstruction({ owner, mandate: ref.mandate, mint: ref.mint, ownerAta: ata, vault: ref.vault, amount, ...(await eventAccounts()) }),
  ];
}

/**
 * Withdraw to the owner's own token account, creating it idempotently first
 * so the verb works even if the owner never held the mint. There is no state
 * precondition here by design: the program has none either.
 */
export async function buildWithdraw(owner: TransactionSigner, ref: MandateRef, amount: bigint): Promise<Instruction[]> {
  const [ata] = await findAssociatedTokenPda({ owner: owner.address, mint: ref.mint, tokenProgram: TOKEN_PROGRAM_ADDRESS });
  return [
    getCreateAssociatedTokenIdempotentInstruction({ payer: owner, ata, owner: owner.address, mint: ref.mint }),
    getOwnerWithdrawInstruction({ owner, mandate: ref.mandate, vault: ref.vault, destination: ata, amount, ...(await eventAccounts()) }),
  ];
}

export async function buildPause(caller: TransactionSigner, mandate: Address): Promise<Instruction[]> {
  return [getPauseInstruction({ caller, mandate, ...(await eventAccounts()) })];
}
export async function buildUnpause(owner: TransactionSigner, mandate: Address): Promise<Instruction[]> {
  return [getUnpauseInstruction({ owner, mandate, ...(await eventAccounts()) })];
}
export async function buildRevoke(caller: TransactionSigner, mandate: Address): Promise<Instruction[]> {
  return [getRevokeInstruction({ caller, mandate, ...(await eventAccounts()) })];
}
export async function buildAmend(owner: TransactionSigner, mandate: Address, newPolicy: PolicyArgs): Promise<Instruction[]> {
  return [getAmendPolicyInstruction({ owner, mandate, newPolicy, ...(await eventAccounts()) })];
}
export async function buildClose(owner: TransactionSigner, ref: MandateRef): Promise<Instruction[]> {
  return [getCloseMandateInstruction({ owner, mandate: ref.mandate, vault: ref.vault, ...(await eventAccounts()) })];
}

export type CreateMandateParams = {
  operator: Address;
  emergency: Address;
  strategyId: string;
  nonce: bigint;
  mint: Address;
  policy: PolicyArgs;
  markAccount?: Address;
  feedIdHex?: string;
};

export async function buildCreateMandate(owner: TransactionSigner, p: CreateMandateParams): Promise<{ instructions: Instruction[]; mandate: Address; vault: Address }> {
  const { address: mandate } = await deriveMandate(owner.address, p.strategyId, p.nonce);
  const { address: vault } = await deriveVault(mandate);
  const ix = getCreateMandateInstruction({
    owner,
    mint: p.mint,
    mandate,
    vault,
    ...(await eventAccounts()),
    operator: p.operator,
    emergency: p.emergency,
    strategyId: idToBytes(p.strategyId),
    nonce: p.nonce,
    policy: p.policy,
    markAccount: p.markAccount ?? PYTH_SOL_USD_PRICE_UPDATE,
    feedId: hexToBytes(p.feedIdHex ?? SOL_USD_FEED_ID_HEX),
  });
  return { instructions: [ix], mandate, vault };
}

/** `Policy` in the shape the program validates (`Policy::validate`), from a compact spec. */
export function policyFromSpec(spec: {
  venues: Address[];
  tokens: Address[];
  allowedActions: number;
  perTxCap: bigint;
  dailyCap: bigint;
  spendPerCall: bigint;
  spendDaily: bigint;
  maxSlippageBps: number;
  maxMarkAgeSecs: bigint;
  expiryTs: bigint;
}): PolicyArgs {
  const zero = "11111111111111111111111111111111" as Address;
  const pad = (xs: Address[]) => [0, 1, 2, 3].map((i) => xs[i] ?? zero) as [Address, Address, Address, Address];
  return {
    venues: pad(spec.venues),
    venuesLen: spec.venues.length,
    tokens: pad(spec.tokens),
    tokensLen: spec.tokens.length,
    allowedActions: spec.allowedActions,
    perTxCap: spec.perTxCap,
    dailyCap: spec.dailyCap,
    spendPerCall: spec.spendPerCall,
    spendDaily: spec.spendDaily,
    maxSlippageBps: spec.maxSlippageBps,
    maxMarkAgeSecs: spec.maxMarkAgeSecs,
    expiryTs: spec.expiryTs,
  };
}

export const ACTION_BITS = { open: 1, increase: 2, reduce: 4, close: 8, flatten: 16, all: 31 } as const;

/** The venues and tokens a `Policy` actually lists (the arrays are fixed-width and NUL-padded). */
export function policyLists(policy: Policy): { venues: Address[]; tokens: Address[] } {
  return { venues: policy.venues.slice(0, policy.venuesLen), tokens: policy.tokens.slice(0, policy.tokensLen) };
}

/** `Policy::assert_tightens` mirrored: which fields of `next` widen `prev` (empty = tightens). */
export function wideningFields(prev: Policy, next: PolicyArgs): string[] {
  const out: string[] = [];
  const gt = (a: bigint | number, b: bigint | number) => BigInt(a) > BigInt(b);
  if (gt(next.perTxCap, prev.perTxCap)) out.push("per-trade cap");
  if (gt(next.dailyCap, prev.dailyCap)) out.push("daily cap");
  if (gt(next.spendPerCall, prev.spendPerCall)) out.push("spend per call");
  if (gt(next.spendDaily, prev.spendDaily)) out.push("spend per day");
  if (next.maxSlippageBps > prev.maxSlippageBps) out.push("max slippage");
  if (gt(next.maxMarkAgeSecs, prev.maxMarkAgeSecs)) out.push("mark age");
  if (gt(next.expiryTs, prev.expiryTs)) out.push("expiry");
  if ((next.allowedActions & ~prev.allowedActions) !== 0) out.push("allowed actions");
  const prevLists = policyLists(prev);
  for (const v of next.venues.slice(0, next.venuesLen)) if (!prevLists.venues.includes(v)) out.push("venues");
  for (const t of next.tokens.slice(0, next.tokensLen)) if (!prevLists.tokens.includes(t)) out.push("tokens");
  return [...new Set(out)];
}
