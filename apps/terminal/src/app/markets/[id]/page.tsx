"use client";

import { useParams } from "next/navigation";
import { useEffect, useState } from "react";
import { useWallet } from "@solana/wallet-adapter-react";
import bs58 from "bs58";
import { api } from "@/lib/api";
import { CandleChart } from "@/components/candles";
import { Depth } from "@/components/depth";

type Level = { price: number; size: number };
type Venue = {
  venue: string;
  mark: number | null;
  funding: number | null;
  spread_bps: number | null;
  depth_10bps: number | null;
  depth_50bps: number | null;
  taker_fee_bps: number | null;
  max_leverage: number | null;
  stale: boolean;
  executable: boolean;
  freshness_ms: number;
  bids?: Level[];
  asks?: Level[];
  tick_size?: number | null;
  lot_size?: number | null;
  min_order_size?: number | null;
};

type Preview = {
  decision: string;
  reason: string;
  reason_code: number;
  stale: boolean;
  checks: Array<{ rule: number; observed: string; limit: string; pass: boolean }>;
};

type Signable = { kind: string; display: string; compact_json: string | null; note?: string; fields?: Record<string, string> };
type Candle = { openTime: number; open: number; high: number; low: number; close: number };

const STATES = ["IDLE", "PREVIEW", "ROUTED", "SIMULATED", "REQUESTED", "AWAITING_SIGNATURE", "SUBMITTED", "FAILED"] as const;

export default function MarketWorkspace() {
  const { id } = useParams<{ id: string }>();
  const { connected, signMessage } = useWallet();
  const [venues, setVenues] = useState<Venue[]>([]);
  const [candles, setCandles] = useState<Candle[]>([]);
  const [side, setSide] = useState<"long" | "short">("long");
  const [notional, setNotional] = useState("50");
  const [leverage, setLeverage] = useState("1.5");
  const [venue, setVenue] = useState("AUTO");
  const [preview, setPreview] = useState<Preview | null>(null);
  const [routes, setRoutes] = useState<Array<{ venue: string; totalBps: number; selected: boolean; executable: boolean; reason: string }>>([]);
  const [msg, setMsg] = useState<string | null>(null);
  const [signable, setSignable] = useState<Signable | null>(null);
  const [requestId, setRequestId] = useState<string | null>(null);
  const [state, setState] = useState<(typeof STATES)[number]>("IDLE");
  const [narrow, setNarrow] = useState(false);
  const [reduceOnly, setReduceOnly] = useState(false);

  useEffect(() => {
    const mq = window.matchMedia("(max-width: 1023px)");
    const apply = () => setNarrow(mq.matches);
    apply();
    mq.addEventListener("change", apply);
    return () => mq.removeEventListener("change", apply);
  }, []);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      const tag = (e.target as HTMLElement | null)?.tagName;
      if (tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT") return;
      if (e.key === "l" || e.key === "L") setSide("long");
      if (e.key === "s" || e.key === "S") setSide("short");
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  useEffect(() => {
    const load = () =>
      api<{ venues: Venue[] }>(`/markets/${id}`)
        .then((r) => setVenues(r.venues))
        .catch(() => undefined);
    load();
    const t = setInterval(load, 3000);
    return () => clearInterval(t);
  }, [id]);

  useEffect(() => {
    api<{ candles: Candle[] }>(`/markets/${id}/candles?venue=pacifica&interval=1h&hours=48`)
      .then((r) => setCandles(r.candles ?? []))
      .catch(() => setCandles([]));
  }, [id]);

  const book = venues.find((v) => v.venue === "pacifica") ?? venues[0];
  const phoenixPinned = venue === "phoenix";

  async function simulate() {
    setMsg(null);
    const body = {
      market: id,
      venue: venue === "AUTO" ? undefined : venue,
      notional_usd: Number(notional),
      leverage: Number(leverage),
    };
    const [p, r] = await Promise.all([
      api<Preview>("/policy/check", { method: "POST", body: JSON.stringify(body) }),
      api<{ routes: typeof routes }>("/routes/compare", {
        method: "POST",
        body: JSON.stringify({ market: id, notional_usd: Number(notional), horizon_hours: 72 }),
      }),
    ]);
    setPreview(p);
    setRoutes(r.routes);
    setState(p.stale ? "PREVIEW" : "SIMULATED");
  }

  async function request() {
    const body = {
      market: id,
      side,
      notional_usd: Number(notional),
      leverage: Number(leverage),
      venue: venue === "AUTO" ? undefined : venue,
    };
    const res = await api<{
      decision: string;
      reason: string;
      request_id: string;
      state?: string;
      approval_required?: boolean;
      signables?: Signable[];
    }>(reduceOnly ? "/trades/reduce" : "/trades/request", { method: "POST", body: JSON.stringify(body) });
    setRequestId(res.request_id);
    setSignable(res.signables?.[0] ?? null);
    setMsg(`${res.decision} · ${res.reason} · ${res.request_id}`);
    setState(res.signables?.[0]?.compact_json ? "AWAITING_SIGNATURE" : res.decision === "REJECT" ? "FAILED" : "REQUESTED");
  }

  async function signPayload() {
    if (!signable?.compact_json || !signMessage || !requestId) return;
    try {
      const sig = await signMessage(new TextEncoder().encode(signable.compact_json));
      const out = await api<{ ok: boolean; note?: string; venue_status?: number }>("/trades/submit", {
        method: "POST",
        body: JSON.stringify({ request_id: requestId, signature: bs58.encode(sig) }),
      });
      setState(out.ok ? "SUBMITTED" : "FAILED");
      setMsg(out.note ?? `venue ${out.venue_status}`);
    } catch (e) {
      setState("FAILED");
      setMsg(e instanceof Error ? e.message : "submit failed");
    }
  }

  async function cancelOrder() {
    const cloid = signable?.fields?.client_order_id ?? requestId;
    if (!cloid) return;
    const res = await api<{
      request_id: string;
      signables?: Signable[];
      reason?: string;
    }>("/orders/cancel", {
      method: "POST",
      body: JSON.stringify({ market: id, client_order_id: cloid }),
    });
    setRequestId(res.request_id);
    setSignable(res.signables?.[0] ?? null);
    setState("AWAITING_SIGNATURE");
    setMsg(res.signables?.[0]?.display ?? "cancel queued");
  }

  return (
    <div className="grid gap-4">
      <div>
        <p className="chip" style={{ background: "white" }}>
          {id}
        </p>
        <h1 className="mt-2 text-[32px] tracking-[-0.04em]" style={{ fontFamily: "var(--font-display)", fontWeight: 800 }}>
          {id}
        </h1>
      </div>
      <div className="grid gap-4 lg:grid-cols-[1.2fr_0.8fr]">
      <div className="grid gap-3 order-2 lg:order-1">
        <div className="clay p-4 overflow-x-auto">
          <CandleChart candles={candles} label="Pacifica testnet · 1h kline · live" />
        </div>
        <div className="grid sm:grid-cols-2 gap-3">
          {venues.map((v) => (
            <div key={v.venue} className={v.executable ? "clay p-4" : "ink p-4"}>
              <div className="flex justify-between text-[12px] font-mono uppercase tracking-[0.08em]">
                <span>{v.venue}</span>
                <span>{v.executable ? "executable" : "INTEGRATED · read-only"}</span>
              </div>
              <div className="num mt-2 text-[28px]" style={{ fontFamily: "var(--font-display)", fontWeight: 800 }}>
                {v.mark?.toFixed(3) ?? "—"}
              </div>
              <dl className="mt-2 grid grid-cols-2 gap-1 text-[12px] text-[var(--mk-muted)]">
                <dt>spread</dt>
                <dd className="num">{v.spread_bps?.toFixed(1) ?? "—"} bps</dd>
                <dt>funding</dt>
                <dd className="num">{v.funding != null ? v.funding.toExponential(2) : "—"}</dd>
                <dt>depth ±10bps</dt>
                <dd className="num">{v.depth_10bps?.toFixed(0) ?? "—"}</dd>
                <dt>taker</dt>
                <dd className="num">{v.taker_fee_bps ?? "unknown"} bps</dd>
                <dt>freshness</dt>
                <dd className="num">
                  {v.freshness_ms}ms {v.stale ? "STALE" : ""}
                </dd>
                <dt>min size</dt>
                <dd className="num">{v.min_order_size ?? "—"}</dd>
              </dl>
            </div>
          ))}
        </div>
        {book && (book.bids?.length || book.asks?.length) ? (
          <div className="clay p-4 overflow-x-auto">
            <Depth bids={book.bids ?? []} asks={book.asks ?? []} venue={book.venue} />
          </div>
        ) : null}
        {routes.length > 0 && (
          <div className="clay p-4">
            <h2 className="text-[13px] font-semibold">Route (72h lifecycle cost)</h2>
            <ul className="mt-2 grid gap-2 text-[12px] font-mono">
              {routes.map((r) => (
                <li key={r.venue}>
                  <div className="h-2 rounded-full bg-black/5 overflow-hidden">
                    <div
                      className="h-full"
                      style={{
                        width: `${Math.min(100, r.totalBps)}%`,
                        background: r.executable ? "var(--mk-blue)" : "#3a3a37",
                      }}
                    />
                  </div>
                  {r.selected ? "→ " : "  "}
                  {r.venue} {r.totalBps.toFixed(1)} bps {r.executable ? "" : "(not executable)"} — {r.reason}
                </li>
              ))}
            </ul>
          </div>
        )}
      </div>
      <aside className="clay p-4 lg:sticky lg:top-20 h-fit order-1 lg:order-2">
        <h2 className="font-semibold">Ticket</h2>
        <p className="text-[12px] text-[var(--mk-muted)] mt-1">
          Owner signs. Keys L / S flip side. Phoenix cannot execute.
        </p>
        {narrow && (
          <p className="mt-2 text-[12px] text-[var(--mk-muted)]">
            On phones the wallet sheet is the authority. Request queues a payload; it does not auto-send.
          </p>
        )}
        <div className="mt-3 grid grid-cols-2 gap-2">
          <button className={`btn ${side === "long" ? "" : "btn-ghost"}`} onClick={() => setSide("long")}>
            Long
          </button>
          <button
            className={`btn ${side === "short" ? "" : "btn-ghost"}`}
            style={side === "short" ? { background: "var(--mk-signal)" } : undefined}
            onClick={() => setSide("short")}
          >
            Short
          </button>
        </div>
        <label className="mt-3 block text-[12px] text-[var(--mk-muted)]">
          Notional USD
          <input className="mt-1 w-full clay px-3 py-2 num" value={notional} onChange={(e) => setNotional(e.target.value)} inputMode="decimal" />
        </label>
        <label className="mt-3 block text-[12px] text-[var(--mk-muted)]">
          Leverage (cap 2.0x)
          <input className="mt-1 w-full clay px-3 py-2 num" value={leverage} onChange={(e) => setLeverage(e.target.value)} inputMode="decimal" disabled={reduceOnly} />
        </label>
        <label className="mt-3 block text-[12px] text-[var(--mk-muted)]">
          Venue
          <select className="mt-1 w-full clay px-3 py-2" value={venue} onChange={(e) => setVenue(e.target.value)}>
            <option>AUTO</option>
            <option value="pacifica">Pacifica</option>
            <option value="drift">Drift</option>
            <option value="phoenix">Phoenix (read-only)</option>
          </select>
        </label>
        <label className="mt-3 flex items-center gap-2 text-[13px]">
          <input type="checkbox" checked={reduceOnly} onChange={(e) => setReduceOnly(e.target.checked)} />
          Reduce only
        </label>
        <div className="mt-4 flex gap-2">
          <button className="btn btn-ghost flex-1" onClick={() => void simulate()}>
            Simulate
          </button>
          <button className="btn flex-1" disabled={!connected || phoenixPinned} onClick={() => void request()}>
            {reduceOnly ? "Reduce" : "Request"}
          </button>
        </div>
        {signable?.compact_json && (
          <button className="btn mt-2 w-full" onClick={() => void signPayload()}>
            Sign and submit to Pacifica
          </button>
        )}
        {connected && requestId && (
          <button className="btn btn-ghost mt-2 w-full" onClick={() => void cancelOrder()}>
            Cancel by client order id
          </button>
        )}
        {!connected && (
          <p className="mt-2 text-[12px] text-[var(--mk-muted)]">Connect a wallet to request. Simulate works disconnected.</p>
        )}
        {phoenixPinned && <p className="mt-2 text-[12px] text-[var(--mk-signal)]">Phoenix is INTEGRATED · read-only.</p>}
        {preview && (
          <div className="mt-4 text-[12px]">
            <div className="chip" style={{ background: preview.decision === "ALLOW" ? "#d9f3e5" : "#fde4dc" }}>
              {preview.decision} · {preview.reason}
              {preview.stale ? " · STALE" : ""}
            </div>
            <table className="mt-2 w-full font-mono">
              <tbody>
                {preview.checks.map((c) => (
                  <tr key={c.rule}>
                    <td>r{c.rule}</td>
                    <td>{c.observed}</td>
                    <td>{c.limit}</td>
                    <td>{c.pass ? "pass" : "fail"}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
        <p className="mt-3 text-[11px] font-mono uppercase tracking-[0.08em] text-[var(--mk-muted)]">{state}</p>
        {msg && <p className="mt-3 text-[12px] font-mono break-all">{msg}</p>}
      </aside>
      </div>
    </div>
  );
}
