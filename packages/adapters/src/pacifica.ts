import { CAPS, PACIFICA_REST, PACIFICA_TESTNET } from "@markov/facts";
import { type DepthLevel, type MarketState, type AdapterCapabilities, getJson } from "./types.ts";

type InfoRow = {
  symbol: string;
  max_leverage: number;
  isolated_only: boolean;
  tick_size: string;
  lot_size: string;
  min_order_size: string;
  funding_rate?: string;
  next_funding_rate?: string;
  instrument_type: string;
  base_asset: string;
};

type PriceRow = {
  symbol: string;
  mark: string;
  oracle: string;
  funding: string;
  next_funding: string;
  open_interest: string;
  volume_24h: string;
  yesterday_price: string;
  timestamp: number;
};

type Book = {
  success: boolean;
  data: { s: string; t: number; l: [Array<{ p: string; a: string; n: number }>, Array<{ p: string; a: string; n: number }>] };
};

export function pacificaBase(env: "mainnet" | "testnet"): string {
  return env === "testnet" ? PACIFICA_TESTNET : PACIFICA_REST;
}

export function pacificaCapabilities(env: "mainnet" | "testnet"): AdapterCapabilities {
  return {
    id: "pacifica",
    env,
    execution_model: "hybrid-offchain-clob",
    order_types: ["limit", "market"],
    margin_modes: ["cross", "isolated"],
    delegation_model: "agent-key-unverified-withdrawal-scope",
    onchain_enforceable: false,
    executable: true,
    freshness_ms: CAPS.freshnessPacificaMs,
  };
}

export async function pacificaMarkets(env: "mainnet" | "testnet" = "mainnet"): Promise<InfoRow[]> {
  const body = await getJson<{ success: boolean; data: InfoRow[] }>(`${pacificaBase(env)}/info`);
  if (!body.success) throw new Error("pacifica /info unsuccessful");
  return body.data.filter((m) => m.instrument_type === "perpetual");
}

export async function pacificaPrices(env: "mainnet" | "testnet" = "mainnet"): Promise<PriceRow[]> {
  const body = await getJson<{ success: boolean; data: PriceRow[] }>(`${pacificaBase(env)}/info/prices`);
  if (!body.success) throw new Error("pacifica /info/prices unsuccessful");
  return body.data;
}

export async function pacificaBook(symbol: string, env: "mainnet" | "testnet" = "mainnet"): Promise<{ bids: DepthLevel[]; asks: DepthLevel[]; ts: number }> {
  const body = await getJson<Book>(`${pacificaBase(env)}/book?symbol=${encodeURIComponent(symbol)}&agg_level=1`);
  if (!body.success) throw new Error("pacifica /book unsuccessful");
  const [bids, asks] = body.data.l;
  return {
    ts: body.data.t,
    bids: (bids ?? []).map((l) => ({ price: Number(l.p), size: Number(l.a) })),
    asks: (asks ?? []).map((l) => ({ price: Number(l.p), size: Number(l.a) })),
  };
}

export function pacificaStateFromParts(
  symbol: string,
  p: PriceRow,
  info: InfoRow | undefined,
  book: { bids: DepthLevel[]; asks: DepthLevel[]; ts: number },
): MarketState {
  const mark = Number(p.mark);
  const y = Number(p.yesterday_price);
  const fetchedAt = Date.now();
  return {
    venue: "pacifica",
    symbol,
    mark,
    index: Number(p.oracle),
    funding: Number(p.funding),
    nextFunding: Number(p.next_funding),
    openInterest: Number(p.open_interest),
    volume24h: Number(p.volume_24h),
    change24h: y ? (mark - y) / y : null,
    bids: book.bids,
    asks: book.asks,
    venueTs: p.timestamp,
    fetchedAt,
    slot: null,
    takerFeeBps: 4.0,
    makerFeeBps: 1.5,
    maxLeverage: info?.max_leverage ?? null,
    isolatedOnly: info?.isolated_only ?? null,
  };
}

export async function pacificaState(symbol: string, env: "mainnet" | "testnet" = "mainnet"): Promise<MarketState> {
  const [prices, book, infos] = await Promise.all([
    pacificaPrices(env),
    pacificaBook(symbol, env),
    pacificaMarkets(env),
  ]);
  const p = prices.find((x) => x.symbol === symbol);
  const info = infos.find((x) => x.symbol === symbol);
  if (!p) throw new Error(`pacifica has no price for ${symbol}`);
  return pacificaStateFromParts(symbol, p, info, book);
}

/** Bytes the owner must `signMessage`. Never a server-held key. */
export function buildPacificaSignable(input: {
  type: string;
  account: string;
  timestamp: number;
  expiry_window: number;
  data: Record<string, unknown>;
}): { compactJson: string; display: string } {
  const payload = {
    data: input.data,
    expiry_window: input.expiry_window,
    timestamp: input.timestamp,
    type: input.type,
  };
  const sorted = sortKeys(payload);
  const compactJson = JSON.stringify(sorted);
  const display = `${input.type} ${JSON.stringify(input.data)} for ${input.account}`;
  return { compactJson, display };
}

function sortKeys(value: unknown): unknown {
  if (Array.isArray(value)) return value.map(sortKeys);
  if (value && typeof value === "object") {
    const out: Record<string, unknown> = {};
    for (const k of Object.keys(value as object).sort()) {
      out[k] = sortKeys((value as Record<string, unknown>)[k]);
    }
    return out;
  }
  return value;
}
