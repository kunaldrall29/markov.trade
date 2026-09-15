import { CANONICAL_MARKETS, REASON_NAME, STAGE, VENUE, VENUE_BIT } from "@markov/facts";
import { loadEnv, type EnvConfig } from "@markov/config";
import {
  depthNotional,
  driftCapabilities,
  jupiterCapabilities,
  mid,
  pacificaBook,
  pacificaCapabilities,
  pacificaMarkets,
  pacificaPrices,
  pacificaStateFromParts,
  phoenixBook,
  phoenixCapabilities,
  phoenixMarkets,
  phoenixStateFromParts,
  slippageBpsAtNotional,
  spreadBps,
  type MarketState,
} from "@markov/adapters";
import { check, type MandateView } from "@markov/policy";
import { compareRoutes } from "@markov/router";
import type { MarketStateDto, Receipt, VenueCapabilities } from "@markov/sdk";

export type Cache = {
  env: EnvConfig;
  pacifica: Map<string, MarketState>;
  phoenix: Map<string, MarketState>;
  lastPacificaOk: number;
  lastPhoenixOk: number;
  receipts: Receipt[];
  sessions: Map<string, { pubkey: string; exp: number }>;
  nonces: Map<string, { pubkey: string; exp: number; message: string }>;
};

const defaultMandate = (env: EnvConfig): MandateView => ({
  version: 1,
  maxLeverageBps: Math.round(env.caps.perPositionLeverageMajors * 10_000),
  maxNotionalUsd: BigInt(env.caps.perAccountGrossNotionalUsd) * 1_000_000n,
  minSafetyBufferBps: 2_000,
  maxDailyLossUsd: 50_000_000n,
  approvedMarkets: CANONICAL_MARKETS.map((m) => m.id),
  allowedVenues: VENUE_BIT.PACIFICA | VENUE_BIT.DRIFT,
});

export function createCache(env = loadEnv()): Cache {
  return {
    env,
    pacifica: new Map(),
    phoenix: new Map(),
    lastPacificaOk: 0,
    lastPhoenixOk: 0,
    receipts: [],
    sessions: new Map(),
    nonces: new Map(),
  };
}

export function dto(canonical: string, state: MarketState, env: EnvConfig, executable: boolean): MarketStateDto {
  const m = mid(state.bids, state.asks);
  const d10 = m ? depthNotional(state.bids, m, 10, "bid") + depthNotional(state.asks, m, 10, "ask") : null;
  const d50 = m ? depthNotional(state.bids, m, 50, "bid") + depthNotional(state.asks, m, 50, "ask") : null;
  const freshness = Date.now() - state.venueTs;
  const limit = state.venue === "phoenix" ? env.caps.freshnessPhoenixMs : env.caps.freshnessPacificaMs;
  return {
    canonical_market_id: canonical,
    venue: state.venue,
    venue_id: state.venue === "pacifica" ? VENUE.PACIFICA : state.venue === "phoenix" ? VENUE.PHOENIX : VENUE.DRIFT,
    symbol: state.symbol,
    mark: state.mark,
    index: state.index,
    funding: state.funding,
    next_funding: state.nextFunding,
    change_24h: state.change24h,
    open_interest: state.openInterest,
    depth_10bps: d10,
    depth_50bps: d50,
    spread_bps: spreadBps(state.bids, state.asks),
    max_leverage: state.maxLeverage,
    isolated_only: state.isolatedOnly,
    taker_fee_bps: state.takerFeeBps,
    maker_fee_bps: state.makerFeeBps,
    health: freshness <= limit * 3 ? "ok" : "degraded",
    freshness_ms: freshness,
    stale: freshness > limit,
    executable,
    env: env.name,
    venue_ts: state.venueTs,
    data_slot: state.slot,
    bids: state.bids.slice(0, 16),
    asks: state.asks.slice(0, 16),
  };
}

export async function refreshMarkets(cache: Cache): Promise<void> {
  const pacEnv = cache.env.name === "devnet" ? "testnet" : "mainnet";
  const [pricesR, infosR, phxR] = await Promise.allSettled([
    pacificaPrices(pacEnv),
    pacificaMarkets(pacEnv),
    phoenixMarkets(),
  ]);
  const prices = pricesR.status === "fulfilled" ? pricesR.value : [];
  const infos = infosR.status === "fulfilled" ? infosR.value : [];
  const phxList = phxR.status === "fulfilled" ? phxR.value : [];
  if (pricesR.status === "fulfilled") cache.lastPacificaOk = Date.now();
  if (phxR.status === "fulfilled") cache.lastPhoenixOk = Date.now();

  await Promise.all(
    CANONICAL_MARKETS.map(async (m) => {
      const p = prices.find((x) => x.symbol === m.pacifica);
      const info = infos.find((x) => x.symbol === m.pacifica);
      if (p) {
        let book = { bids: [] as { price: number; size: number }[], asks: [] as { price: number; size: number }[], ts: p.timestamp };
        try {
          book = await pacificaBook(m.pacifica, pacEnv);
        } catch {
          /* mark still usable; slippage fails closed without a book */
        }
        cache.pacifica.set(m.id, pacificaStateFromParts(m.pacifica, p, info, book));
      }
      const pm = phxList.find((x) => x.symbol === m.phoenix);
      if (pm) {
        try {
          const book = await phoenixBook(m.phoenix);
          cache.phoenix.set(m.id, phoenixStateFromParts(pm, book));
        } catch {
          /* Phoenix row stays empty rather than inventing a book */
        }
      }
    }),
  );
}

export function listMarkets(cache: Cache): MarketStateDto[] {
  const out: MarketStateDto[] = [];
  for (const m of CANONICAL_MARKETS) {
    const p = cache.pacifica.get(m.id);
    if (p) out.push(dto(m.id, p, cache.env, cache.env.pacificaWrites));
    const x = cache.phoenix.get(m.id);
    if (x) out.push(dto(m.id, x, cache.env, false));
  }
  return out;
}

export function capabilities(cache: Cache): VenueCapabilities[] {
  const pac = cache.env.name === "devnet" ? "testnet" : "mainnet";
  return [
    pacificaCapabilities(pac),
    { ...driftCapabilities(cache.env.cluster === "devnet" ? "devnet" : "mainnet"), executable: false },
    phoenixCapabilities(),
    { ...jupiterCapabilities(), executable: cache.env.investLiveSwaps },
  ];
}

export function bestMark(cache: Cache, id: string): MarketStateDto | null {
  const rows = listMarkets(cache).filter((r) => r.canonical_market_id === id && !r.stale && r.mark != null);
  const exec = rows.filter((r) => r.executable);
  const pool = exec.length ? exec : rows;
  if (!pool.length) return null;
  return pool.slice().sort((a, b) => (a.spread_bps ?? 999) - (b.spread_bps ?? 999))[0] ?? null;
}

export function policyPreview(
  cache: Cache,
  input: {
    market: string;
    venue?: string;
    notional_usd: number;
    leverage: number;
    max_slippage_bps?: number;
  },
) {
  const venueName = input.venue && input.venue !== "AUTO" ? input.venue : "pacifica";
  const state = venueName === "phoenix" ? cache.phoenix.get(input.market) : cache.pacifica.get(input.market);
  const freshnessLimit = venueName === "phoenix" ? cache.env.caps.freshnessPhoenixMs : cache.env.caps.freshnessPacificaMs;
  const age = state ? Date.now() - state.venueTs : 99_999;
  const slip = state
    ? slippageBpsAtNotional(state.asks, input.notional_usd) ?? 999
    : 999;
  const result = check({
    action: {
      kind: "TradeOpen",
      actor: "owner",
      isOwner: true,
      venueId: venueName === "phoenix" ? VENUE.PHOENIX : venueName === "drift" ? VENUE.DRIFT : VENUE.PACIFICA,
      marketId: input.market,
      projectedLeverageBps: Math.round(input.leverage * 10_000),
      projectedNotionalUsd: BigInt(Math.round(input.notional_usd * 1_000_000)),
      safetyBufferBps: 5_000,
      dailyLossUsd: 0n,
      slippageBps: Math.round(slip),
      slippageLimitBps: input.max_slippage_bps ?? 40,
      dataAgeMs: age,
      freshnessLimitMs: freshnessLimit,
      slot: BigInt(state?.slot ?? 0),
    },
    account: { owner: "owner", status: "Active", globalPaused: false },
    mandate: defaultMandate(cache.env),
  });
  return {
    decision: result.decision,
    reason_code: result.reason_code,
    reason: REASON_NAME[result.reason_code] ?? String(result.reason_code),
    checks: result.checks.map((c) => ({
      rule: c.rule,
      observed: c.observed.toString(),
      limit: c.limit.toString(),
      pass: c.pass,
    })),
    data_slot: state?.slot ?? 0,
    env: cache.env.name,
    freshness_ms: age,
    stale: !state || age > freshnessLimit,
  };
}

export function routesFor(cache: Cache, market: string, notional: number, horizon: number) {
  const p = cache.pacifica.get(market);
  const x = cache.phoenix.get(market);
  const quotes = [];
  if (p) {
    quotes.push({
      venueId: VENUE.PACIFICA,
      venue: "pacifica",
      executable: cache.env.pacificaWrites,
      linked: true,
      stale: Date.now() - p.venueTs > cache.env.caps.freshnessPacificaMs,
      mark: p.mark,
      slippageBps: slippageBpsAtNotional(p.asks, notional) ?? 50,
      takerFeeBps: p.takerFeeBps ?? 4,
      fundingBpsPerDay: (p.funding ?? 0) * 10_000 * 24,
      exitBps: slippageBpsAtNotional(p.bids, notional) ?? 50,
      riskPremiumBps: 0,
      freshnessMs: Date.now() - p.venueTs,
    });
  }
  if (x) {
    quotes.push({
      venueId: VENUE.PHOENIX,
      venue: "phoenix",
      executable: false,
      linked: false,
      stale: Date.now() - x.venueTs > cache.env.caps.freshnessPhoenixMs,
      mark: x.mark,
      slippageBps: slippageBpsAtNotional(x.asks, notional) ?? 50,
      takerFeeBps: x.takerFeeBps ?? 3.5,
      fundingBpsPerDay: (x.funding ?? 0) * 10_000 * 24,
      exitBps: slippageBpsAtNotional(x.bids, notional) ?? 50,
      riskPremiumBps: 0,
      freshnessMs: Date.now() - x.venueTs,
    });
  }
  return {
    env: cache.env.name,
    market,
    horizon_hours: horizon,
    routes: compareRoutes(quotes, horizon),
    note: "Drift is omitted until RPC+SDK subscribe produces a native mark. Phoenix is integrated, not executable. Pacifica marks are never used as a Drift stand-in.",
  };
}

export function recordReceipt(cache: Cache, partial: Omit<Receipt, "env" | "created_at"> & { env?: Receipt["env"] }): Receipt {
  const rec: Receipt = {
    ...partial,
    env: partial.env ?? cache.env.name,
    created_at: new Date().toISOString(),
  };
  cache.receipts.unshift(rec);
  return rec;
}

export { defaultMandate, STAGE };
