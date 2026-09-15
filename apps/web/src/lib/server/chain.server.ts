/**
 * Server-side chain reads for the same-origin read API.
 *
 * This is a cache in front of the chain, not a source of truth (ADR-010).
 * Every response carries the slot it was read at and `source: "chain"`; a
 * read that fails returns an error, never the previous value dressed as new.
 */
import {
  BOOK_ONE_STRATEGY_ID,
  DEMO_PERPS_PROGRAM_ID,
  DEVNET_RPC_FALLBACK,
  DEVNET_RPC_PRIMARY,
  GATE_B_MANDATE,
  GATE_B_POLICY,
  MANDATE_PROGRAM_ID,
  MandateState,
  SOL_PERP_MARKET_ID,
  bytesToId,
  createRpc,
  fetchMandateView,
  fetchPythPrice,
  fetchRegistry,
  fetchTokenBalance,
  fetchVenueView,
  priceE6,
  walkReceipts,
  type MandateView,
  type MarkovRpc,
} from "@markov/sdk";
import type { Address, Signature } from "@solana/kit";
import type { BookStats, Circuit, Health, MandateSummary, PolicyView, ReceiptsResponse } from "@/lib/api-types";

const primaryUrl = process.env.RPC_URL || DEVNET_RPC_PRIMARY;
const fallbackUrl = process.env.RPC_HTTP_FALLBACK || DEVNET_RPC_FALLBACK;

let primary: MarkovRpc | undefined;
let fallback: MarkovRpc | undefined;

async function readEither<T>(fn: (rpc: MarkovRpc) => Promise<T>): Promise<T> {
  try {
    return await fn((primary ??= createRpc(primaryUrl)));
  } catch (a) {
    if (fallbackUrl === primaryUrl) throw a;
    try {
      return await fn((fallback ??= createRpc(fallbackUrl)));
    } catch (b) {
      throw new Error(`both RPC endpoints failed: ${host(primaryUrl)}: ${msg(a)}; ${host(fallbackUrl)}: ${msg(b)}`);
    }
  }
}

function msg(e: unknown): string {
  return e instanceof Error ? e.message : String(e);
}
function host(u: string): string {
  try {
    return new URL(u).host;
  } catch {
    return u;
  }
}

// ---- cache -----------------------------------------------------------------

type Entry<T> = { at: number; value: Promise<T> };
const cache = new Map<string, Entry<unknown>>();

async function cached<T>(key: string, ttlMs: number, fn: () => Promise<T>): Promise<T> {
  const now = Date.now();
  const hit = cache.get(key) as Entry<T> | undefined;
  if (hit && now - hit.at < ttlMs) return hit.value;
  const value = fn();
  cache.set(key, { at: now, value });
  value.catch(() => cache.delete(key));
  return value;
}

// ---- serialisation -----------------------------------------------------------

import { money, serializeReceipt } from "@/lib/receipt-rows";

function policyView(m: MandateView): PolicyView {
  const p = m.data.policy;
  const names = ["open", "increase", "reduce", "close", "flatten"];
  return {
    venues: p.venues.slice(0, p.venuesLen),
    tokens: p.tokens.slice(0, p.tokensLen),
    allowed_actions: names.filter((_, i) => (p.allowedActions & (1 << i)) !== 0),
    per_tx_cap: money(p.perTxCap, m.data.mint),
    daily_cap: money(p.dailyCap, m.data.mint),
    spend_per_call: money(p.spendPerCall, m.data.mint),
    spend_daily: money(p.spendDaily, m.data.mint),
    max_slippage_bps: p.maxSlippageBps,
    max_mark_age_secs: p.maxMarkAgeSecs.toString(),
    expiry_ts: p.expiryTs.toString(),
  };
}

const STATE_NAMES = ["Active", "Paused", "Revoked"] as const;

export async function mandateSummary(rpc: MarkovRpc, m: MandateView): Promise<MandateSummary> {
  const vault = await fetchTokenBalance(rpc, m.data.vault, m.data.mint);
  return {
    address: m.address,
    owner: m.data.owner,
    operator: m.data.operator,
    emergency: m.data.emergency,
    strategy: bytesToId(m.data.strategyId),
    state: STATE_NAMES[m.data.state] ?? "Active",
    nonce: m.data.nonce.toString(),
    created_at: m.data.createdAt.toString(),
    action_seq: m.data.actionSeq.toString(),
    mint: m.data.mint,
    mark_account: m.data.markAccount,
    vault: { ...money(vault.amount, m.data.mint), address: m.data.vault, exists: vault.exists },
    policy: policyView(m),
    day: { epoch: m.data.dayEpoch.toString(), notional_used: money(m.data.dayNotionalUsed, m.data.mint), spend_used: money(m.data.daySpendUsed, m.data.mint) },
    withdraw_enabled: true,
  };
}

// ---- endpoints ----------------------------------------------------------------

export async function receiptsPage(opts: { address?: string; limit: number; before?: string }): Promise<ReceiptsResponse> {
  const address = (opts.address ?? MANDATE_PROGRAM_ID) as Address;
  const key = `receipts:${address}:${opts.limit}:${opts.before ?? ""}`;
  return cached(key, 8_000, async () => {
    const page = await readEither((rpc) =>
      walkReceipts(rpc, { address, limit: opts.limit, before: opts.before as Signature | undefined, concurrency: 4 }),
    );
    return {
      env: "devnet",
      source: "chain",
      data_slot: page.slot.toString(),
      fetched_at: Date.now(),
      address,
      receipts: page.receipts.map((r) => serializeReceipt(r)),
      before: page.before,
      signatures: page.signatures,
    };
  });
}

export async function mandateView(address: string): Promise<MandateSummary | null> {
  return cached(`mandate:${address}`, 5_000, async () =>
    readEither(async (rpc) => {
      const m = await fetchMandateView(rpc, address as Address);
      return m ? mandateSummary(rpc, m) : null;
    }),
  );
}

export async function bookStats(): Promise<BookStats> {
  return cached("book-stats", 5_000, async () =>
    readEither(async (rpc) => {
      const m = await fetchMandateView(rpc, GATE_B_MANDATE);
      if (!m) throw new Error(`Gate B mandate ${GATE_B_MANDATE} not found on chain`);
      const [summary, venue, pyth, registry, recent] = await Promise.all([
        mandateSummary(rpc, m),
        fetchVenueView(rpc, GATE_B_MANDATE, SOL_PERP_MARKET_ID, DEMO_PERPS_PROGRAM_ID),
        fetchPythPrice(rpc, m.data.markAccount),
        fetchRegistry(rpc),
        walkReceipts(rpc, { address: GATE_B_MANDATE, limit: 100, concurrency: 4 }),
      ]);
      const now = Math.floor(Date.now() / 1000);
      const cutoff = now - 24 * 3600;
      const inWindow = recent.receipts.filter((r) => (r.blockTime ?? 0) >= cutoff && !r.txError);
      const oldestWalked = recent.receipts.length ? Math.min(...recent.receipts.map((r) => r.blockTime ?? now)) : now;
      const truncated = recent.signatures >= 100 && oldestWalked >= cutoff;

      const pythAge = pyth ? Math.max(0, now - Number(pyth.price.priceMessage.publishTime)) : null;
      const maxAge = Number(m.data.policy.maxMarkAgeSecs);
      let circuit: Circuit = "live";
      if (registry?.globalHalt) circuit = "global_halt";
      else if (m.data.state === MandateState.Paused) circuit = "paused";
      else if (m.data.state === MandateState.Revoked) circuit = "revoked";
      else if (now >= Number(m.data.policy.expiryTs)) circuit = "expired";
      else if (pythAge == null || pythAge > maxAge) circuit = "stale_mark";

      return {
        env: "devnet",
        source: "chain",
        data_slot: venue.slot.toString(),
        fetched_at: Date.now(),
        mandate: summary,
        position: venue.position
          ? {
              address: venue.addresses.position,
              side: venue.position.side === 1 ? "short" : "long",
              notional: money(venue.position.notional, m.data.mint),
              entry_price_e6: venue.position.entryPrice.toString(),
              funding_accrued: venue.position.fundingAccrued.toString(),
              updated_slot: venue.position.updatedSlot.toString(),
            }
          : null,
        mark: {
          venue: venue.mark
            ? {
                price_e6: priceE6(venue.mark.price, venue.mark.expo)?.toString() ?? null,
                source: venue.mark.source === 1 ? "house" : "pyth",
                slot: venue.mark.slot.toString(),
                publish_time: venue.mark.publishTime.toString(),
              }
            : null,
          pyth: pyth
            ? {
                price_e6: priceE6(pyth.price.priceMessage.price, pyth.price.priceMessage.exponent)?.toString() ?? null,
                publish_time: pyth.price.priceMessage.publishTime.toString(),
                posted_slot: pyth.price.postedSlot.toString(),
                verification: pyth.price.verificationLevel.__kind,
                age_secs: pythAge ?? 0,
              }
            : null,
        },
        registry: registry ? { global_halt: registry.globalHalt, adapters: registry.adapters.slice(0, registry.adaptersLen) } : null,
        circuit,
        window: {
          hours: 24,
          refusals: inWindow.filter((r) => r.kind === "refusal").length,
          actions: inWindow.filter((r) => r.kind === "action").length,
          owner_actions: inWindow.filter((r) => r.kind === "owner").length,
          truncated,
          signatures_walked: recent.signatures,
        },
        enforcement: { delta: "offchain", gross: "offchain", daily_loss: "offchain" },
        offchain_limits: {
          delta_band: money(GATE_B_POLICY.offchain.deltaBand, m.data.mint),
          max_gross: money(GATE_B_POLICY.offchain.maxGross, m.data.mint),
          daily_loss_bps: GATE_B_POLICY.offchain.dailyLossBps,
        },
      } satisfies BookStats;
    }),
  );
}

export async function health(): Promise<Health> {
  const started = Date.now();
  const failing: string[] = [];
  let slot: bigint | null = null;
  let error: string | null = null;
  try {
    slot = await readEither((rpc) => rpc.getSlot({ commitment: "confirmed" }).send());
  } catch (e) {
    error = msg(e);
    failing.push("rpc");
  }
  const latency = Date.now() - started;
  if (slot != null && latency > 10_000) failing.push("latency");
  return {
    ok: failing.length === 0,
    chainReady: failing.length === 0,
    failing,
    env: "devnet",
    program: MANDATE_PROGRAM_ID,
    rpc: { host: host(primaryUrl), slot: slot?.toString() ?? null, latency_ms: slot == null ? null : latency, error },
    checked_at: Date.now(),
  };
}

export const constants = { BOOK_ONE_STRATEGY_ID, GATE_B_MANDATE, SOL_PERP_MARKET_ID };
