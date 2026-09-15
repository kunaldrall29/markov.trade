export type DepthLevel = { price: number; size: number };

export type MarketState = {
  venue: "pacifica" | "drift" | "phoenix";
  symbol: string;
  mark: number;
  index: number | null;
  funding: number | null;
  nextFunding: number | null;
  openInterest: number | null;
  volume24h: number | null;
  change24h: number | null;
  bids: DepthLevel[];
  asks: DepthLevel[];
  venueTs: number;
  fetchedAt: number;
  slot: number | null;
  takerFeeBps: number | null;
  makerFeeBps: number | null;
  maxLeverage: number | null;
  isolatedOnly: boolean | null;
};

export type AdapterCapabilities = {
  id: "pacifica" | "drift" | "phoenix" | "jupiter";
  env: string;
  execution_model: string;
  order_types: string[];
  margin_modes: string[];
  delegation_model: string;
  onchain_enforceable: boolean;
  executable: boolean;
  freshness_ms: number;
};

export function mid(bids: DepthLevel[], asks: DepthLevel[]): number | null {
  if (!bids[0] || !asks[0]) return null;
  return (bids[0].price + asks[0].price) / 2;
}

export function spreadBps(bids: DepthLevel[], asks: DepthLevel[]): number | null {
  const m = mid(bids, asks);
  if (!m || m === 0 || !bids[0] || !asks[0]) return null;
  return ((asks[0].price - bids[0].price) / m) * 10_000;
}

export function depthNotional(levels: DepthLevel[], midPx: number, bps: number, side: "bid" | "ask"): number {
  const bound = side === "bid" ? midPx * (1 - bps / 10_000) : midPx * (1 + bps / 10_000);
  let n = 0;
  for (const l of levels) {
    if (side === "bid" && l.price < bound) break;
    if (side === "ask" && l.price > bound) break;
    n += l.price * l.size;
  }
  return n;
}

export function slippageBpsAtNotional(levels: DepthLevel[], notionalUsd: number): number | null {
  if (!levels[0] || notionalUsd <= 0) return null;
  const start = levels[0].price;
  let remain = notionalUsd;
  let filled = 0;
  let cost = 0;
  for (const l of levels) {
    const levelN = l.price * l.size;
    const take = Math.min(remain, levelN);
    cost += take;
    filled += take / l.price;
    remain -= take;
    if (remain <= 1e-9) break;
  }
  if (remain > 1e-6 || filled === 0) return null;
  const avg = cost / filled;
  return (Math.abs(avg - start) / start) * 10_000;
}

export async function getJson<T>(url: string, timeoutMs = 8_000): Promise<T> {
  const ctrl = new AbortController();
  const t = setTimeout(() => ctrl.abort(), timeoutMs);
  try {
    const res = await fetch(url, {
      signal: ctrl.signal,
      headers: { accept: "application/json", "user-agent": "markov-adapters/0.1" },
    });
    if (!res.ok) throw new Error(`${res.status} ${url}`);
    return (await res.json()) as T;
  } finally {
    clearTimeout(t);
  }
}
