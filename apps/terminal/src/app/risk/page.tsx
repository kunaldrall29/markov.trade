"use client";

import { useEffect, useState } from "react";
import { api } from "@/lib/api";
import { Page } from "@/components/page";

type Risk = {
  equity: number | null;
  effective_leverage: number | null;
  liquidation: number | null;
  cap_leverage: number;
  cap_notional_usd: number;
  daily_loss_usd: number;
  note: string;
};

type Shock = {
  equityUsd: number | null;
  leverage: number | null;
  price_shock_bps: number;
  note: string;
};

export default function RiskPage() {
  const [r, setR] = useState<Risk | null>(null);
  const [shock, setShock] = useState("500");
  const [sim, setSim] = useState<Shock | null>(null);

  useEffect(() => {
    api<Risk>("/risk").then(setR).catch(() => undefined);
  }, []);

  async function run() {
    const out = await api<Shock>("/risk/simulate-scenario", {
      method: "POST",
      body: JSON.stringify({ price_shock_bps: Number(shock) }),
    });
    setSim(out);
  }

  return (
    <Page title="Risk" note="Leverage is null when equity is unknown — never displayed as 0x. Stress numbers are simulated and labelled as such.">
      <div className="grid sm:grid-cols-3 gap-3">
        <Card k="Equity" v={r?.equity != null ? `$${r.equity.toFixed(2)}` : "—"} />
        <Card k="Effective leverage" v={r?.effective_leverage != null ? `${r.effective_leverage.toFixed(2)}x` : "—"} />
        <Card k="Liquidation" v={r?.liquidation != null ? r.liquidation.toFixed(2) : "—"} />
      </div>
      <div className="clay p-5 grid gap-2 text-[14px]">
        <Row k="Cap leverage" v={`${(r?.cap_leverage ?? 2).toFixed(1)}x`} />
        <Row k="Cap notional" v={`$${r?.cap_notional_usd ?? 2000}`} />
        <Row k="Daily loss budget" v={`$${r?.daily_loss_usd ?? 50}`} />
        <p className="text-[12px] text-[var(--mk-muted)] mt-2">{r?.note}</p>
      </div>
      <div className="clay p-5 grid gap-3">
        <h2 className="font-semibold">Price shock (labelled SIMULATED)</h2>
        <label className="text-[12px] text-[var(--mk-muted)]">
          Shock bps
          <input className="mt-1 w-full clay px-3 py-2 num" value={shock} onChange={(e) => setShock(e.target.value)} inputMode="numeric" />
        </label>
        <button className="btn w-fit" onClick={() => void run()}>
          Simulate
        </button>
        {sim && (
          <div className="text-[13px] font-mono grid gap-1">
            <p>
              SIMULATED · {sim.price_shock_bps} bps · equity {sim.equityUsd ?? "null"} · leverage {sim.leverage ?? "null"}
            </p>
            <p className="text-[var(--mk-muted)]">{sim.note}</p>
          </div>
        )}
      </div>
    </Page>
  );
}

function Card({ k, v }: { k: string; v: string }) {
  return (
    <div className="clay p-4">
      <div className="text-[11px] uppercase tracking-[0.1em] font-mono text-[var(--mk-muted)]">{k}</div>
      <div className="mt-2 text-[24px]" style={{ fontFamily: "var(--font-display)", fontWeight: 800 }}>
        {v}
      </div>
    </div>
  );
}

function Row({ k, v }: { k: string; v: string }) {
  return (
    <div className="flex justify-between gap-4 border-b border-black/5 py-2">
      <span className="text-[var(--mk-muted)]">{k}</span>
      <span className="font-medium">{v}</span>
    </div>
  );
}
