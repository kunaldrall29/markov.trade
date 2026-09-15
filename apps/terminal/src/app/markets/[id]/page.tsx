"use client";

import { useParams } from "next/navigation";
import { useEffect, useState } from "react";
import { useWallet } from "@solana/wallet-adapter-react";
import bs58 from "bs58";
import { api } from "@/lib/api";

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
};

type Preview = {
  decision: string;
  reason: string;
  reason_code: number;
  stale: boolean;
  checks: Array<{ rule: number; observed: string; limit: string; pass: boolean }>;
};

type Signable = { kind: string; display: string; compact_json: string | null; note?: string };

export default function MarketWorkspace() {
  const { id } = useParams<{ id: string }>();
  const { connected, signMessage } = useWallet();
  const [venues, setVenues] = useState<Venue[]>([]);
  const [side, setSide] = useState<"long" | "short">("long");
  const [notional, setNotional] = useState("50");
  const [leverage, setLeverage] = useState("1.5");
  const [venue, setVenue] = useState("AUTO");
  const [preview, setPreview] = useState<Preview | null>(null);
  const [routes, setRoutes] = useState<Array<{ venue: string; totalBps: number; selected: boolean; executable: boolean; reason: string }>>([]);
  const [msg, setMsg] = useState<string | null>(null);
  const [signable, setSignable] = useState<Signable | null>(null);
  const [requestId, setRequestId] = useState<string | null>(null);

  useEffect(() => {
    const load = () =>
      api<{ venues: Venue[] }>(`/markets/${id}`)
        .then((r) => setVenues(r.venues))
        .catch(() => undefined);
    load();
    const t = setInterval(load, 3000);
    return () => clearInterval(t);
  }, [id]);

  const book = venues.find((v) => v.venue === "pacifica") ?? venues[0];

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
      approval_required?: boolean;
      signables?: Signable[];
    }>("/trades/request", { method: "POST", body: JSON.stringify(body) });
    setRequestId(res.request_id);
    setSignable(res.signables?.[0] ?? null);
    setMsg(`${res.decision} · ${res.reason} · ${res.request_id}`);
  }

  async function signPayload() {
    if (!signable?.compact_json || !signMessage || !requestId) return;
    const sig = await signMessage(new TextEncoder().encode(signable.compact_json));
    await api("/trades/confirm-signature", {
      method: "POST",
      body: JSON.stringify({ request_id: requestId, signature: bs58.encode(sig) }),
    });
    setMsg(`signature stored on receipt ${requestId}. not a fill.`);
  }

  return (
    <div className="grid gap-4 lg:grid-cols-[1.2fr_0.8fr]">
      <div className="grid gap-3">
        <div>
          <p className="chip" style={{ background: "white" }}>
            {id}
          </p>
          <h1 className="mt-2 text-[32px] tracking-[-0.04em]" style={{ fontFamily: "var(--font-display)", fontWeight: 800 }}>
            {id}
          </h1>
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
              </dl>
            </div>
          ))}
        </div>
        {book && (book.bids?.length || book.asks?.length) ? (
          <div className="clay p-4 overflow-x-auto">
            <h2 className="text-[13px] font-semibold">Book · {book.venue}</h2>
            <div className="mt-2 grid grid-cols-2 gap-3 text-[12px] font-mono">
              <div>
                <div className="text-[var(--mk-muted)] uppercase tracking-[0.08em] text-[11px]">Bids</div>
                {(book.bids ?? []).slice(0, 8).map((l, i) => (
                  <div key={`b${i}`} className="flex justify-between">
                    <span>{l.price}</span>
                    <span>{l.size}</span>
                  </div>
                ))}
              </div>
              <div>
                <div className="text-[var(--mk-muted)] uppercase tracking-[0.08em] text-[11px]">Asks</div>
                {(book.asks ?? []).slice(0, 8).map((l, i) => (
                  <div key={`a${i}`} className="flex justify-between">
                    <span>{l.price}</span>
                    <span>{l.size}</span>
                  </div>
                ))}
              </div>
            </div>
          </div>
        ) : null}
        {routes.length > 0 && (
          <div className="clay p-4">
            <h2 className="text-[13px] font-semibold">Route (72h lifecycle cost)</h2>
            <ul className="mt-2 grid gap-1 text-[12px] font-mono">
              {routes.map((r) => (
                <li key={r.venue}>
                  {r.selected ? "→ " : "  "}
                  {r.venue} {r.totalBps.toFixed(1)} bps {r.executable ? "" : "(not executable)"} — {r.reason}
                </li>
              ))}
            </ul>
          </div>
        )}
      </div>
      <aside className="clay p-4 lg:sticky lg:top-20 h-fit">
        <h2 className="font-semibold">Ticket</h2>
        <p className="text-[12px] text-[var(--mk-muted)] mt-1">
          Owner signs. Phones can submit a request; the wallet prompt is the authority.
        </p>
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
          <input className="mt-1 w-full clay px-3 py-2 num" value={leverage} onChange={(e) => setLeverage(e.target.value)} inputMode="decimal" />
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
        <div className="mt-4 flex gap-2">
          <button className="btn btn-ghost flex-1" onClick={() => void simulate()}>
            Simulate
          </button>
          <button className="btn flex-1" disabled={!connected} onClick={() => void request()}>
            Request
          </button>
        </div>
        {signable?.compact_json && (
          <button className="btn mt-2 w-full" onClick={() => void signPayload()}>
            Sign Pacifica payload
          </button>
        )}
        {!connected && (
          <p className="mt-2 text-[12px] text-[var(--mk-muted)]">Connect a wallet to request. Simulate works disconnected.</p>
        )}
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
        {msg && <p className="mt-3 text-[12px] font-mono break-all">{msg}</p>}
      </aside>
    </div>
  );
}
