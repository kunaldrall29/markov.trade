"use client";

import Link from "next/link";
import { useEffect, useMemo, useState } from "react";
import { api } from "@/lib/api";

type Row = {
  canonical_market_id: string;
  venue: string;
  mark: number | null;
  change_24h: number | null;
  funding: number | null;
  stale: boolean;
  executable: boolean;
  spread_bps: number | null;
  freshness_ms: number;
};

export default function Markets() {
  const [rows, setRows] = useState<Row[]>([]);
  const [q, setQ] = useState("");
  const [err, setErr] = useState<string | null>(null);
  useEffect(() => {
    const load = () =>
      api<{ markets: Row[] }>("/markets")
        .then((r) => {
          setRows(r.markets);
          setErr(null);
        })
        .catch((e: Error) => setErr(e.message));
    load();
    const t = setInterval(load, 3000);
    return () => clearInterval(t);
  }, []);
  const grouped = useMemo(() => {
    const map = new Map<string, Row[]>();
    for (const r of rows) {
      if (q && !r.canonical_market_id.toLowerCase().includes(q.toLowerCase())) continue;
      const list = map.get(r.canonical_market_id) ?? [];
      list.push(r);
      map.set(r.canonical_market_id, list);
    }
    return [...map.entries()];
  }, [rows, q]);

  return (
    <div className="grid gap-4">
      <h1 className="text-[32px] tracking-[-0.04em]" style={{ fontFamily: "var(--font-display)", fontWeight: 800 }}>Markets</h1>
      <input className="clay px-4 py-3 text-[14px] outline-none w-full" placeholder="Search SOL-PERP" value={q} onChange={(e) => setQ(e.target.value)} />
      {err && <p className="text-[var(--mk-signal)] text-[13px]">{err}</p>}
      <div className="clay overflow-x-auto">
        <table className="w-full text-left text-[13px]">
          <thead className="text-[11px] uppercase tracking-[0.08em] text-[var(--mk-muted)] font-mono">
            <tr>
              <th className="p-3">Market</th>
              <th className="p-3">Mark</th>
              <th className="p-3">24h</th>
              <th className="p-3">Venues</th>
              <th className="p-3">Spread</th>
            </tr>
          </thead>
          <tbody>
            {grouped.length === 0 && (
              <tr><td className="p-4 text-[var(--mk-muted)]" colSpan={5}>No live rows yet — waiting on Pacifica/Phoenix, or the API is down.</td></tr>
            )}
            {grouped.map(([id, vs]) => {
              const best = vs.find((v) => v.executable && !v.stale) ?? vs[0];
              return (
                <tr key={id} className="border-t border-black/5">
                  <td className="p-3"><Link href={`/markets/${id}`} className="font-semibold">{id}</Link></td>
                  <td className="p-3 num">{best?.mark?.toFixed(4) ?? "—"}</td>
                  <td className="p-3 num">{best?.change_24h != null ? `${(best.change_24h * 100).toFixed(2)}%` : "—"}</td>
                  <td className="p-3">
                    {vs.map((v) => (
                      <span key={v.venue} className="chip mr-1" style={{ background: v.executable ? "#e8edff" : "white" }}>
                        {v.venue}{v.stale ? " stale" : ""}{v.executable ? "" : " ro"}
                      </span>
                    ))}
                  </td>
                  <td className="p-3 num">{best?.spread_bps?.toFixed(1) ?? "—"}</td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>
    </div>
  );
}
