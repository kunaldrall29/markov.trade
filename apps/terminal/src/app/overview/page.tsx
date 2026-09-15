"use client";

import { useEffect, useState } from "react";
import { api } from "@/lib/api";
import { stageLabel } from "@/lib/env";

type Health = { ok: boolean; env: string; pacifica_markets: number; phoenix_markets: number };
type Markets = { markets: Array<{ canonical_market_id: string; venue: string; mark: number | null; stale: boolean; executable: boolean; funding: number | null; freshness_ms: number }> };
type Receipts = { receipts: Array<{ request_id: string; decision: string; reason: string; market_id: string }> };

export default function Overview() {
  const [health, setHealth] = useState<Health | null>(null);
  const [markets, setMarkets] = useState<Markets["markets"]>([]);
  const [receipts, setReceipts] = useState<Receipts["receipts"]>([]);
  const [err, setErr] = useState<string | null>(null);

  useEffect(() => {
    let alive = true;
    const load = () => {
      Promise.all([api<Health>("/health"), api<Markets>("/markets"), api<Receipts>("/receipts")])
        .then(([h, m, r]) => {
          if (!alive) return;
          setHealth(h);
          setMarkets(m.markets);
          setReceipts(r.receipts);
          setErr(null);
        })
        .catch((e: Error) => alive && setErr(e.message));
    };
    load();
    const t = setInterval(load, 4000);
    return () => {
      alive = false;
      clearInterval(t);
    };
  }, []);

  const sol = markets.find((m) => m.canonical_market_id === "SOL-PERP" && m.executable && !m.stale) ?? markets.find((m) => m.canonical_market_id === "SOL-PERP");

  return (
    <div className="grid gap-4">
      <div>
        <p className="chip" style={{ background: "white" }}>{stageLabel}</p>
        <h1 className="mt-3 text-[40px] leading-[0.95] tracking-[-0.04em]" style={{ fontFamily: "var(--font-display)", fontWeight: 800 }}>
          Capital may only do<br />what you allowed.
        </h1>
          <p className="mt-3 max-w-xl text-[15px] text-[var(--mk-muted)]">
          Live Pacifica and Phoenix marks. Drift executable after RPC subscribe. Phoenix is read-only. Empty panels are empty.
        </p>
      </div>
      {err && <div className="clay p-4 text-[13px] text-[var(--mk-signal)]">API unreachable. {err}</div>}
      <div className="grid sm:grid-cols-2 xl:grid-cols-4 gap-3">
        <Stat label="Equity" value="—" note="No linked venue account" />
        <Stat label="Gross notional" value="—" note="No positions" />
        <Stat label="SOL mark" value={sol?.mark != null ? sol.mark.toFixed(3) : "—"} note={sol ? `${sol.venue} · ${sol.freshness_ms}ms` : "waiting for feed"} />
        <Stat label="Receipts" value={String(receipts.length)} note="this process" />
      </div>
      <div className="grid lg:grid-cols-[1.4fr_0.8fr] gap-3">
        <div className="clay p-5">
          <h2 className="text-[13px] font-semibold">Linked venues</h2>
          <ul className="mt-3 grid gap-2 text-[13px]">
            <li className="flex justify-between"><span>Pacifica {health ? `(${health.pacifica_markets} cached)` : ""}</span><span className="chip" style={{ background: "#e8edff", color: "var(--mk-blue)" }}>executable · {health?.env}</span></li>
            <li className="flex justify-between"><span>Drift</span><span className="chip" style={{ background: "white" }}>stale-closed · no HTTP</span></li>
            <li className="flex justify-between"><span>Phoenix</span><span className="chip" style={{ background: "white" }}>INTEGRATED · read-only</span></li>
          </ul>
        </div>
        <div className="ink p-5">
          <h2 className="text-[13px] font-semibold" style={{ color: "var(--mk-blue-on-ink)" }}>Recent receipts</h2>
          {receipts.length === 0 ? (
            <p className="mt-3 text-[13px] text-white/60">None yet. A reject still writes a receipt.</p>
          ) : (
            <ul className="mt-3 grid gap-2 text-[12px] font-mono">
              {receipts.slice(0, 6).map((r) => (
                <li key={r.request_id}>{r.decision} · {r.reason} · {r.market_id}</li>
              ))}
            </ul>
          )}
        </div>
      </div>
    </div>
  );
}

function Stat({ label, value, note }: { label: string; value: string; note: string }) {
  return (
    <div className="clay p-4">
      <div className="text-[11px] uppercase tracking-[0.1em] text-[var(--mk-muted)] font-mono">{label}</div>
      <div className="num mt-2 text-[28px] tracking-[-0.03em]" style={{ fontFamily: "var(--font-display)", fontWeight: 800 }}>{value}</div>
      <div className="mt-1 text-[12px] text-[var(--mk-muted)]">{note}</div>
    </div>
  );
}
