import { CAPS, PHOENIX_REST } from "@markov/facts";
import { type DepthLevel, type MarketState, type AdapterCapabilities, getJson } from "./types.ts";

type PhxMarket = {
  symbol: string;
  assetId: number;
  marketStatus: string;
  takerFee: number;
  makerFee: number;
  isolatedOnly: boolean;
  leverageTiers?: Array<{ maxLeverage: number }>;
  statsSnapshot?: { markPrice?: number; indexPrice?: number; fundingRate?: number; openInterest?: number };
};

type PhxBook = {
  slot: number;
  symbol: string;
  bids: [number, number][];
  asks: [number, number][];
  mid?: number | null;
};

export function phoenixCapabilities(): AdapterCapabilities {
  return {
    id: "phoenix",
    env: "mainnet",
    execution_model: "onchain-clob+spline",
    order_types: ["limit", "market"],
    margin_modes: ["isolated", "cross"],
    delegation_model: "position-authority",
    onchain_enforceable: true,
    executable: false,
    freshness_ms: CAPS.freshnessPhoenixMs,
  };
}

export async function phoenixMarkets(): Promise<PhxMarket[]> {
  return getJson<PhxMarket[]>(`${PHOENIX_REST}/v1/view/exchange/markets`);
}

export async function phoenixBook(symbol: string): Promise<{ bids: DepthLevel[]; asks: DepthLevel[]; slot: number; mid: number | null }> {
  const body = await getJson<PhxBook>(`${PHOENIX_REST}/v1/view/orderbook/${encodeURIComponent(symbol)}`);
  return {
    slot: body.slot,
    mid: body.mid ?? null,
    bids: (body.bids ?? []).map(([p, s]) => ({ price: p, size: s })),
    asks: (body.asks ?? []).map(([p, s]) => ({ price: p, size: s })),
  };
}

export function phoenixStateFromParts(
  m: PhxMarket,
  book: { bids: DepthLevel[]; asks: DepthLevel[]; slot: number; mid: number | null },
): MarketState {
  const mark = book.mid ?? book.bids[0]?.price ?? m.statsSnapshot?.markPrice ?? 0;
  const fetchedAt = Date.now();
  return {
    venue: "phoenix",
    symbol: m.symbol,
    mark,
    index: m.statsSnapshot?.indexPrice ?? null,
    funding: m.statsSnapshot?.fundingRate ?? null,
    nextFunding: null,
    openInterest: m.statsSnapshot?.openInterest ?? null,
    volume24h: null,
    change24h: null,
    bids: book.bids,
    asks: book.asks,
    venueTs: fetchedAt,
    fetchedAt,
    slot: book.slot,
    takerFeeBps: m.takerFee * 10_000,
    makerFeeBps: m.makerFee * 10_000,
    maxLeverage: m.leverageTiers?.[0]?.maxLeverage ?? null,
    isolatedOnly: m.isolatedOnly,
  };
}

export async function phoenixState(symbol: string): Promise<MarketState> {
  const [markets, book] = await Promise.all([phoenixMarkets(), phoenixBook(symbol)]);
  const m = markets.find((x) => x.symbol === symbol);
  if (!m) throw new Error(`phoenix has no market ${symbol}`);
  return phoenixStateFromParts(m, book);
}
