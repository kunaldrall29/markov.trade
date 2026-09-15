"use client";

import { useEffect, useState } from "react";
import { api } from "@/lib/api";
import { Page } from "@/components/page";

type Rec = { request_id: string; decision: string; reason: string; market_id: string; created_at: string; tx_signature?: string | null };

export default function Receipts() {
  const [rows, setRows] = useState<Rec[]>([]);
  useEffect(() => {
    const load = () =>
      api<{ receipts: Rec[] }>("/receipts")
        .then((r) => setRows(r.receipts))
        .catch(() => undefined);
    load();
    const t = setInterval(load, 4000);
    return () => clearInterval(t);
  }, []);
  return (
    <Page title="Receipts" note="Every allow, reject, skip and approval is a receipt. Copy JSON from the row.">
      <div className="clay overflow-x-auto">
        <table className="w-full text-[13px]">
          <thead className="font-mono text-[11px] uppercase tracking-[0.08em] text-[var(--mk-muted)]">
            <tr><th className="p-3 text-left">When</th><th className="p-3 text-left">Decision</th><th className="p-3 text-left">Market</th><th className="p-3 text-left">Id</th></tr>
          </thead>
          <tbody>
            {rows.length === 0 && <tr><td className="p-4 text-[var(--mk-muted)]" colSpan={4}>No receipts in this process yet.</td></tr>}
            {rows.map((r) => (
              <tr key={r.request_id} className="border-t border-black/5">
                <td className="p-3 num">{r.created_at}</td>
                <td className="p-3">{r.decision} · {r.reason}</td>
                <td className="p-3">{r.market_id}</td>
                <td className="p-3 num text-[11px]"><a href={`/receipts/${r.request_id}`}>{r.request_id.slice(0, 8)}…</a></td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </Page>
  );
}
