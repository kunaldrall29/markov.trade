"use client";

import { useEffect, useState } from "react";
import { useWallet } from "@solana/wallet-adapter-react";
import { api } from "@/lib/api";
import { Page } from "@/components/page";

type Mandate = {
  version: number;
  on_chain: boolean;
  program_deployed: boolean;
  note: string;
  rules: {
    max_leverage_bps: number;
    max_notional_usd: number;
    min_safety_buffer_bps: number;
    max_daily_loss_usd: number;
    approved_markets: string[];
  };
};

export default function MandatePage() {
  const { connected } = useWallet();
  const [m, setM] = useState<Mandate | null>(null);
  const [lev, setLev] = useState("2.0");
  const [notional, setNotional] = useState("2000");
  const [msg, setMsg] = useState<string | null>(null);

  useEffect(() => {
    api<Mandate>("/mandate").then(setM).catch(() => undefined);
  }, []);

  async function save() {
    const res = await api<{ receipt_id: string; draft: { version: number } }>("/mandate", {
      method: "POST",
      body: JSON.stringify({
        max_leverage_bps: Math.round(Number(lev) * 10_000),
        max_notional_usd: Number(notional),
      }),
    });
    setMsg(`draft v${res.draft.version} · ${res.receipt_id}`);
    const next = await api<Mandate>("/mandate");
    setM(next);
  }

  return (
    <Page title="Mandate" note="Hard rules. Changing them is owner-signed. Soft preferences never override these.">
      <div className="clay p-5 grid gap-2 text-[14px]">
        <Row k="Version" v={String(m?.version ?? 0)} />
        <Row k="On chain" v={m?.on_chain ? "yes" : "no"} />
        <Row k="Max leverage" v={`${((m?.rules.max_leverage_bps ?? 20000) / 10000).toFixed(1)}x`} />
        <Row k="Max notional" v={`$${m?.rules.max_notional_usd ?? 2000}`} />
        <Row k="Min safety buffer" v={`${((m?.rules.min_safety_buffer_bps ?? 2000) / 100).toFixed(0)}%`} />
        <Row k="Daily loss" v={`$${m?.rules.max_daily_loss_usd ?? 50}`} />
        <Row k="Approved markets" v={(m?.rules.approved_markets ?? []).join(" · ")} />
        <p className="text-[12px] text-[var(--mk-muted)] mt-2">{m?.note}</p>
      </div>
      <div className="clay p-5 grid gap-3">
        <h2 className="font-semibold">New draft</h2>
        <label className="text-[12px] text-[var(--mk-muted)]">
          Leverage (capped at 2.0)
          <input className="mt-1 w-full clay px-3 py-2 num" value={lev} onChange={(e) => setLev(e.target.value)} />
        </label>
        <label className="text-[12px] text-[var(--mk-muted)]">
          Max notional USD (capped at 2000)
          <input className="mt-1 w-full clay px-3 py-2 num" value={notional} onChange={(e) => setNotional(e.target.value)} />
        </label>
        <button className="btn w-fit" disabled={!connected} onClick={() => void save()}>
          Save draft
        </button>
        {!connected && <p className="text-[12px] text-[var(--mk-muted)]">Connect and SIWS to write a draft.</p>}
        {msg && <p className="text-[12px] font-mono break-all">{msg}</p>}
      </div>
    </Page>
  );
}

function Row({ k, v }: { k: string; v: string }) {
  return (
    <div className="flex justify-between gap-4 border-b border-black/5 py-2">
      <span className="text-[var(--mk-muted)]">{k}</span>
      <span className="font-medium text-right">{v}</span>
    </div>
  );
}
