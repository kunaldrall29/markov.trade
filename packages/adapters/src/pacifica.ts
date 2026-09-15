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
    tickSize: info ? Number(info.tick_size) : null,
    lotSize: info ? Number(info.lot_size) : null,
    minOrderSize: info ? Number(info.min_order_size) : null,
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

export type PacificaCandle = {
  openTime: number;
  closeTime: number;
  symbol: string;
  interval: string;
  open: number;
  high: number;
  low: number;
  close: number;
  volume: number;
  trades: number;
};

export async function pacificaKlines(
  symbol: string,
  env: "mainnet" | "testnet" = "mainnet",
  interval = "1h",
  hours = 48,
): Promise<PacificaCandle[]> {
  const end = Date.now();
  const start = end - hours * 3_600_000;
  const body = await getJson<{
    success: boolean;
    data: Array<{ t: number; T: number; s: string; i: string; o: string; c: string; h: string; l: string; v: string; n: number }>;
  }>(
    `${pacificaBase(env)}/kline?symbol=${encodeURIComponent(symbol)}&interval=${encodeURIComponent(interval)}&start_time=${start}&end_time=${end}`,
  );
  if (!body.success) throw new Error("pacifica /kline unsuccessful");
  return (body.data ?? []).map((k) => ({
    openTime: k.t,
    closeTime: k.T,
    symbol: k.s,
    interval: k.i,
    open: Number(k.o),
    high: Number(k.h),
    low: Number(k.l),
    close: Number(k.c),
    volume: Number(k.v),
    trades: k.n,
  }));
}

export async function pacificaAccount(
  account: string,
  env: "mainnet" | "testnet" = "mainnet",
): Promise<{ found: boolean; status: number; raw: unknown }> {
  const url = `${pacificaBase(env)}/account?account=${encodeURIComponent(account)}`;
  const ctrl = new AbortController();
  const t = setTimeout(() => ctrl.abort(), 8_000);
  try {
    const res = await fetch(url, {
      signal: ctrl.signal,
      headers: { accept: "application/json", "user-agent": "markov-adapters/0.1" },
    });
    const raw = await res.json();
    const found = res.ok && (raw as { success?: boolean }).success !== false;
    return { found, status: res.status, raw };
  } finally {
    clearTimeout(t);
  }
}

export async function pacificaPositions(account: string, env: "mainnet" | "testnet" = "mainnet"): Promise<unknown[]> {
  const body = await getJson<{ success: boolean; data: unknown[] }>(
    `${pacificaBase(env)}/positions?account=${encodeURIComponent(account)}`,
  );
  return body.success ? (body.data ?? []) : [];
}

export type PacificaOrderFields = {
  symbol: string;
  price: string;
  amount: string;
  side: "bid" | "ask";
  tif: "IOC" | "GTC";
  reduce_only: boolean;
  client_order_id: string;
};

export function roundToStep(value: number, step: number, mode: "floor" | "ceil"): number {
  if (!(step > 0) || !Number.isFinite(value)) return value;
  const n = mode === "floor" ? Math.floor(value / step + 1e-12) * step : Math.ceil(value / step - 1e-12) * step;
  const decimals = (step.toString().split(".")[1] ?? "").length;
  return Number(n.toFixed(decimals));
}

export function formatStep(value: number, step: number): string {
  const decimals = (step.toString().split(".")[1] ?? "").length;
  return value.toFixed(decimals);
}

export function buildPacificaCreateOrder(input: {
  account: string;
  symbol: string;
  side: "long" | "short";
  notionalUsd: number;
  mark: number;
  tickSize: number;
  lotSize: number;
  bestBid: number | null;
  bestAsk: number | null;
  tif?: "IOC" | "GTC";
  reduceOnly?: boolean;
  clientOrderId: string;
  timestamp?: number;
  expiryWindow?: number;
}): {
  compactJson: string;
  display: string;
  fields: PacificaOrderFields;
  timestamp: number;
  expiry_window: number;
} {
  const tif = input.tif ?? "IOC";
  const side = input.side === "long" ? "bid" : "ask";
  const pxRaw = input.side === "long" ? (input.bestAsk ?? input.mark) : (input.bestBid ?? input.mark);
  const tick = input.tickSize || 0.01;
  const lot = input.lotSize || 0.01;
  const priceN = roundToStep(pxRaw, tick, input.side === "long" ? "ceil" : "floor");
  const amountN = roundToStep(input.notionalUsd / input.mark, lot, "floor");
  if (!(amountN > 0) || !(priceN > 0) || !(input.mark > 0)) {
    throw new Error("order size or price rounds to zero");
  }
  const fields: PacificaOrderFields = {
    symbol: input.symbol,
    price: formatStep(priceN, tick),
    amount: formatStep(amountN, lot),
    side,
    tif,
    reduce_only: Boolean(input.reduceOnly),
    client_order_id: input.clientOrderId,
  };
  const timestamp = input.timestamp ?? Date.now();
  const expiry_window = input.expiryWindow ?? 30_000;
  const signed = buildPacificaSignable({
    type: "create_order",
    account: input.account,
    timestamp,
    expiry_window,
    data: fields,
  });
  return { ...signed, fields, timestamp, expiry_window };
}

export async function submitPacificaOrder(
  env: "mainnet" | "testnet",
  input: {
    account: string;
    signature: string;
    timestamp: number;
    expiry_window: number;
    fields: PacificaOrderFields;
  },
): Promise<{ status: number; body: unknown }> {
  const url = `${pacificaBase(env)}/orders/create`;
  const ctrl = new AbortController();
  const t = setTimeout(() => ctrl.abort(), 12_000);
  try {
    const res = await fetch(url, {
      method: "POST",
      signal: ctrl.signal,
      headers: {
        "content-type": "application/json",
        accept: "application/json",
        "user-agent": "markov-adapters/0.1",
      },
      body: JSON.stringify({
        account: input.account,
        signature: input.signature,
        timestamp: input.timestamp,
        expiry_window: input.expiry_window,
        ...input.fields,
      }),
    });
    const text = await res.text();
    let body: unknown = text;
    try {
      body = JSON.parse(text);
    } catch {
      /* venue sometimes returns plain text */
    }
    return { status: res.status, body };
  } finally {
    clearTimeout(t);
  }
}
