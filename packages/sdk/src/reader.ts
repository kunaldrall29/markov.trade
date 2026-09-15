/**
 * Chain readers. Every function reads the chain through `@solana/kit` and
 * returns either decoded state or `null` for "does not exist"; an RPC failure
 * throws. Nothing here defaults a value — a read that fails is a read that
 * failed, and the surface says so (conventions §1 "stale fails closed").
 */
import {
  createSolanaRpc,
  getBase58Decoder,
  getBase64Encoder,
  type Address,
  type Rpc,
  type Signature,
  type SolanaRpcApi,
} from "@solana/kit";
import { decodeToken, findAssociatedTokenPda, TOKEN_PROGRAM_ADDRESS } from "@solana-program/token";
import { getMandateDecoder, getRegistryDecoder, MANDATE_DISCRIMINATOR, type Mandate, type Registry } from "./generated/mandate";
import { getPriceUpdateV2Decoder, type PriceUpdateV2 } from "./generated/mandate/types";
import { getMarkAccountDecoder, getMarketDecoder, getPositionDecoder, type MarkAccount, type Market, type Position } from "./generated/demoPerps";
import { DEMO_PERPS_PROGRAM_ID, DEVNET_RPC_PRIMARY, MANDATE_PROGRAM_ID, PYTH_RECEIVER_PROGRAM_ID, SOL_PERP_MARKET_ID } from "./facts";
import { deriveRegistry, deriveVenueMark, deriveVenueMarket, deriveVenuePosition } from "./pdas";
import { receiptsFromTransaction, type Receipt, type TransactionLike } from "./receipts";

export type MarkovRpc = Rpc<SolanaRpcApi>;

export function createRpc(url: string = DEVNET_RPC_PRIMARY): MarkovRpc {
  return createSolanaRpc(url);
}

const b64 = getBase64Encoder();

async function fetchRaw(rpc: MarkovRpc, address: Address): Promise<{ data: Uint8Array; owner: Address; slot: bigint } | null> {
  const res = await rpc.getAccountInfo(address, { encoding: "base64", commitment: "confirmed" }).send();
  if (!res.value) return null;
  const [encoded] = res.value.data;
  return { data: b64.encode(encoded) as Uint8Array, owner: res.value.owner, slot: res.context.slot };
}

export type MandateView = { address: Address; data: Mandate; slot: bigint };

export async function fetchMandateView(rpc: MarkovRpc, mandate: Address): Promise<MandateView | null> {
  const raw = await fetchRaw(rpc, mandate);
  if (!raw) return null;
  if (raw.owner !== MANDATE_PROGRAM_ID) throw new Error(`${mandate} is not owned by the mandate program`);
  return { address: mandate, data: getMandateDecoder().decode(raw.data), slot: raw.slot };
}

/**
 * Every mandate whose `owner` is `owner`: `getProgramAccounts` filtered on the
 * account discriminator (offset 0) and the owner pubkey (offset 8).
 */
export async function fetchMandatesByOwner(rpc: MarkovRpc, owner: Address): Promise<MandateView[]> {
  const base58 = getBase58Decoder();
  const res = await rpc
    .getProgramAccounts(MANDATE_PROGRAM_ID, {
      encoding: "base64",
      commitment: "confirmed",
      filters: [
        { memcmp: { offset: 0n, bytes: base58.decode(MANDATE_DISCRIMINATOR) as never, encoding: "base58" } },
        { memcmp: { offset: 8n, bytes: owner as never, encoding: "base58" } },
      ],
    })
    .send();
  const decoder = getMandateDecoder();
  const slot = await rpc.getSlot({ commitment: "confirmed" }).send();
  return res
    .map((a) => ({ address: a.pubkey, data: decoder.decode(b64.encode(a.account.data[0]) as Uint8Array), slot }))
    .sort((x, y) => Number(y.data.createdAt - x.data.createdAt));
}

export async function fetchRegistry(rpc: MarkovRpc): Promise<Registry | null> {
  const raw = await fetchRaw(rpc, await deriveRegistry());
  return raw ? getRegistryDecoder().decode(raw.data) : null;
}

export type TokenBalance = { address: Address; amount: bigint; mint: Address; exists: true } | { address: Address; exists: false; amount: 0n; mint: Address };

/** A token account's balance; a missing account is `exists: false`, not zero. */
export async function fetchTokenBalance(rpc: MarkovRpc, tokenAccount: Address, expectedMint: Address): Promise<TokenBalance> {
  const res = await rpc.getAccountInfo(tokenAccount, { encoding: "base64", commitment: "confirmed" }).send();
  if (!res.value) return { address: tokenAccount, exists: false, amount: 0n, mint: expectedMint };
  const decoded = decodeToken({
    address: tokenAccount,
    data: b64.encode(res.value.data[0]) as Uint8Array,
    executable: res.value.executable,
    lamports: res.value.lamports,
    programAddress: res.value.owner,
    space: res.value.space,
    exists: true,
  } as never);
  return { address: tokenAccount, exists: true, amount: decoded.data.amount, mint: decoded.data.mint };
}

export async function ownerAta(owner: Address, mint: Address): Promise<Address> {
  const [ata] = await findAssociatedTokenPda({ owner, mint, tokenProgram: TOKEN_PROGRAM_ADDRESS });
  return ata;
}

export type VenueView = {
  market: Market | null;
  mark: MarkAccount | null;
  position: Position | null;
  addresses: { market: Address; mark: Address; position: Address };
  slot: bigint;
};

/** The mock venue's view of one mandate on one market. `null` parts mean the account does not exist. */
export async function fetchVenueView(rpc: MarkovRpc, mandate: Address, marketId: string = SOL_PERP_MARKET_ID, venue: Address = DEMO_PERPS_PROGRAM_ID): Promise<VenueView> {
  const [marketAddr, markAddr, positionAddr] = await Promise.all([
    deriveVenueMarket(marketId, venue),
    deriveVenueMark(marketId, venue),
    deriveVenuePosition(mandate, marketId, venue),
  ]);
  const res = await rpc.getMultipleAccounts([marketAddr, markAddr, positionAddr], { encoding: "base64", commitment: "confirmed" }).send();
  const [m, k, p] = res.value;
  const dec = (v: typeof m) => (v ? (b64.encode(v.data[0]) as Uint8Array) : null);
  const md = dec(m);
  const kd = dec(k);
  const pd = dec(p);
  return {
    market: md ? getMarketDecoder().decode(md) : null,
    mark: kd ? getMarkAccountDecoder().decode(kd) : null,
    position: pd ? getPositionDecoder().decode(pd) : null,
    addresses: { market: marketAddr, mark: markAddr, position: positionAddr },
    slot: res.context.slot,
  };
}

/** `PriceUpdateV2` account discriminator (Pyth receiver, Anchor `#[account]`). */
export const PRICE_UPDATE_V2_DISCRIMINATOR = new Uint8Array([34, 241, 35, 99, 157, 126, 244, 205]);

export type PythView = { price: PriceUpdateV2; slot: bigint; address: Address };

/** A Pyth `PriceUpdateV2`, refused (thrown) unless the receiver owns it and the discriminator matches. */
export async function fetchPythPrice(rpc: MarkovRpc, account: Address): Promise<PythView | null> {
  const raw = await fetchRaw(rpc, account);
  if (!raw) return null;
  if (raw.owner !== PYTH_RECEIVER_PROGRAM_ID) throw new Error(`${account} is not owned by the Pyth receiver`);
  for (let i = 0; i < 8; i += 1) if (raw.data[i] !== PRICE_UPDATE_V2_DISCRIMINATOR[i]) throw new Error(`${account} is not a PriceUpdateV2`);
  return { price: getPriceUpdateV2Decoder().decode(raw.data.subarray(8)), slot: raw.slot, address: account };
}

/** Rescale a Pyth `(price, expo)` to the program's 1e6 fixed point; `null` if it cannot be stated. */
export function priceE6(price: bigint, expo: number): bigint | null {
  if (price <= 0n) return null;
  const shift = expo + 6;
  if (shift >= 0) return price * 10n ** BigInt(shift);
  return price / 10n ** BigInt(-shift);
}

export type ReceiptPage = { receipts: Receipt[]; signatures: number; before: Signature | null; slot: bigint };

/**
 * Walk the program's signature history newest-first and decode every receipt.
 * `getTransaction` calls run with bounded concurrency because keyless devnet
 * endpoints throttle bursts (FACTS `RPC endpoints`).
 */
export async function walkReceipts(
  rpc: MarkovRpc,
  opts: { limit?: number; before?: Signature; until?: Signature; concurrency?: number; programId?: Address; address?: Address } = {},
): Promise<ReceiptPage> {
  const programId = opts.programId ?? MANDATE_PROGRAM_ID;
  const target = opts.address ?? programId;
  const limit = Math.min(Math.max(opts.limit ?? 50, 1), 1000);
  const sigs = await rpc
    .getSignaturesForAddress(target, { limit, before: opts.before, until: opts.until, commitment: "confirmed" })
    .send();
  const slot = await rpc.getSlot({ commitment: "confirmed" }).send();
  const concurrency = Math.max(1, opts.concurrency ?? 4);
  const results: Receipt[][] = new Array(sigs.length);
  let next = 0;
  async function worker() {
    while (next < sigs.length) {
      const i = next++;
      const s = sigs[i]!;
      const tx = await rpc
        .getTransaction(s.signature, { encoding: "json", maxSupportedTransactionVersion: 0, commitment: "confirmed" })
        .send();
      results[i] = tx ? receiptsFromTransaction(s.signature, tx as unknown as TransactionLike, programId) : [];
    }
  }
  await Promise.all(Array.from({ length: Math.min(concurrency, sigs.length) }, worker));
  return {
    receipts: results.flat(),
    signatures: sigs.length,
    before: sigs.length ? sigs[sigs.length - 1]!.signature : null,
    slot,
  };
}
