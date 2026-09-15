"use client";

import { useEffect, useMemo, useState } from "react";
import { api } from "@/lib/api";
import { Page } from "@/components/page";

type Rec = {
  request_id: string;
  decision: string;
  reason: string;
  market_id: string;
  created_at: string;
  kind: string;
  tx_signature?: string | null;
};

const DECISIONS = ["ALL", "ALLOW", "REJECT", "REQUIRE_APPROVAL", "SKIP"] as const;

export default function Receipts() {
  const [rows, setRows] = useState<Rec[]>([]);
  const [decision, setDecision] = useState<(typeof DECISIONS)[number]>("ALL");
  const [q, setQ] = useState("");

  useEffect(() => {
    const load = () => {
      const qs = decision === "ALL" ? "" : `?decision=${decision}`;
      api<{ receipts: Rec[] }>(`/receipts${qs}`)
        .then((r) => setRows(r.receipts))
        .catch(() => undefined);
    };
    load();
    const t = setInterval(load, 4000);
    return () => clearInterval(t);
  }, [decision]);

  const shown = useMemo(() => {
    const needle = q.toLowerCase();
    if (!needle) return rows;
    return rows.filter(
      (r) =>
        r.kind.toLowerCase().includes(needle) ||
        r.market_id.toLowerCase().includes(needle) ||
        r.request_id.toLowerCase().includes(needle) ||
        r.reason.toLowerCase().includes(needle),
    );
  }, [rows, q]);

  return (
    <Page title="Receipts" note="Every allow, reject, skip and approval is a receipt. Open a row for JSON.">
      <div className="flex flex-wrap gap-2">
        {DECISIONS.map((d) => (
          <button key={d} className={`btn ${decision === d ? "" : "btn-ghost"}`} onClick={() => setDecision(d)}>
            {d}
          </button>
        ))}
      </div>
      <input
        className="clay px-4 py-3 text-[14px] outline-none w-full"
        placeholder="Filter kind, market, id"
        value={q}
        onChange={(e) => setQ(e.target.value)}
      />
      <div className="clay overflow-x-auto">
        <table className="w-full text-[13px]">
          <thead className="font-mono text-[11px] uppercase tracking-[0.08em] text-[var(--mk-muted)]">
            <tr>
              <th className="p-3 text-left">When</th>
              <th className="p-3 text-left">Kind</th>
              <th className="p-3 text-left">Decision</th>
              <th className="p-3 text-left">Market</th>
              <th className="p-3 text-left">Id</th>
            </tr>
          </thead>
          <tbody>
            {shown.length === 0 && (
              <tr>
                <td className="p-4 text-[var(--mk-muted)]" colSpan={5}>
                  No receipts in this process yet.
                </td>
              </tr>
            )}
            {shown.map((r) => (
              <tr key={r.request_id} className="border-t border-black/5">
                <td className="p-3 num whitespace-nowrap">{r.created_at.replace("T", " ").slice(0, 19)}</td>
                <td className="p-3">{r.kind}</td>
                <td className="p-3">
                  {r.decision} · {r.reason}
                </td>
                <td className="p-3">{r.market_id}</td>
                <td className="p-3 num text-[11px]">
                  <a href={`/receipts/${r.request_id}`}>{r.request_id.slice(0, 8)}…</a>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </Page>
  );
}
