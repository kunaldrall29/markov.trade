"use client";

import { useEffect, useState } from "react";
import { api } from "@/lib/api";
import { Page } from "@/components/page";

type Portfolio = {
  equity: number | null;
  gross_notional: number | null;
  net_delta: number | null;
  effective_leverage: number | null;
  cap_notional_usd: number;
  cap_leverage: number;
  pubkey: string | null;
  note: string;
};

export default function Portfolio() {
  const [p, setP] = useState<Portfolio | null>(null);
  const [err, setErr] = useState<string | null>(null);

  useEffect(() => {
    api<Portfolio>("/portfolio")
      .then(setP)
      .catch((e: Error) => setErr(e.message));
  }, []);

  return (
    <Page title="Portfolio" note="Aggregate equity, exposure and funding appear after a venue account is linked. Nothing is invented here.">
      {err && <p className="text-[13px] text-[var(--mk-signal)]">{err}</p>}
      <div className="grid sm:grid-cols-3 gap-3">
        <Card k="Equity" v={p?.equity != null ? `$${p.equity.toFixed(2)}` : "—"} n="venue balance" />
        <Card k="Gross notional" v={p?.gross_notional != null ? `$${p.gross_notional.toFixed(2)}` : "—"} n={`cap $${p?.cap_notional_usd ?? 2000}`} />
        <Card k="Effective leverage" v={p?.effective_leverage != null ? `${p.effective_leverage.toFixed(2)}x` : "—"} n="null while equity is unknown" />
      </div>
      <p className="text-[13px] text-[var(--mk-muted)]">{p?.note}</p>
    </Page>
  );
}

function Card({ k, v, n }: { k: string; v: string; n: string }) {
  return (
    <div className="clay p-4">
      <div className="text-[11px] uppercase tracking-[0.1em] font-mono text-[var(--mk-muted)]">{k}</div>
      <div className="mt-2 text-[24px]" style={{ fontFamily: "var(--font-display)", fontWeight: 800 }}>
        {v}
      </div>
      <div className="mt-1 text-[12px] text-[var(--mk-muted)]">{n}</div>
    </div>
  );
}
