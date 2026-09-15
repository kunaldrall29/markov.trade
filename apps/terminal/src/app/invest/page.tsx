"use client";

import { useEffect, useState } from "react";
import { useWallet } from "@solana/wallet-adapter-react";
import { TSLAX, NVDAX, SPYX, AAPLX } from "@markov/facts";
import { api } from "@/lib/api";
import { Page } from "@/components/page";

const ASSETS = [
  { s: "TSLAx", m: TSLAX },
  { s: "NVDAx", m: NVDAX },
  { s: "SPYx", m: SPYX },
  { s: "AAPLx", m: AAPLX },
];

type Rule = { id: string; mint: string; usd_per_period: number; period_seconds: number; status: string; note?: string };

export default function Invest() {
  const { connected } = useWallet();
  const [mint, setMint] = useState(NVDAX);
  const [usd, setUsd] = useState("5");
  const [rules, setRules] = useState<Rule[]>([]);
  const [msg, setMsg] = useState<string | null>(null);
  const [quote, setQuote] = useState<string | null>(null);

  useEffect(() => {
    api<{ rules: Rule[] }>("/invest/rules")
      .then((r) => setRules(r.rules))
      .catch(() => undefined);
  }, []);

  async function propose() {
    const res = await api<{ receipt_id: string; decision: string; rule: Rule }>("/invest/propose", {
      method: "POST",
      body: JSON.stringify({ mint, usd_per_period: Number(usd), period_seconds: 604800 }),
    });
    setMsg(`${res.decision} · ${res.receipt_id}`);
    const next = await api<{ rules: Rule[] }>("/invest/rules");
    setRules(next.rules);
  }

  async function simulateQuote() {
    const res = await api<{ executable: boolean; note?: string; quote?: unknown }>("/invest/quote", {
      method: "POST",
      body: JSON.stringify({ mint, usd: Number(usd) }),
    });
    setQuote(res.executable ? JSON.stringify(res.quote) : (res.note ?? "not executable"));
  }

  return (
    <Page
      title="Invest"
      note="Recurring buys inside the Solana Subscriptions & Allowances cap. Tokenized stocks are price exposure, not shares. Issuers keep pause and delegate powers. xStocks are mainnet-only."
    >
      <div className="clay p-5">
        <h2 className="font-semibold">Allowlist (pinned mints)</h2>
        <ul className="mt-3 grid gap-2 text-[13px] font-mono">
          {ASSETS.map((a) => (
            <li key={a.s} className="flex flex-col sm:flex-row sm:justify-between gap-1">
              <span>{a.s}</span>
              <span className="text-[var(--mk-muted)] break-all">{a.m}</span>
            </li>
          ))}
        </ul>
      </div>
      <div className="clay p-5 grid gap-3">
        <h2 className="font-semibold">Propose a weekly buy</h2>
        <p className="text-[13px] text-[var(--mk-muted)]">
          Two signatures when the program is live: invest mandate, then USDC recurring delegation. Reference Safe is unavailable until an official-price feed is verified (gate I6).
        </p>
        <label className="text-[12px] text-[var(--mk-muted)]">
          Asset
          <select className="mt-1 w-full clay px-3 py-2" value={mint} onChange={(e) => setMint(e.target.value)}>
            {ASSETS.map((a) => (
              <option key={a.m} value={a.m}>
                {a.s}
              </option>
            ))}
          </select>
        </label>
        <label className="text-[12px] text-[var(--mk-muted)]">
          USD per week (5–100)
          <input className="mt-1 w-full clay px-3 py-2 num" value={usd} onChange={(e) => setUsd(e.target.value)} />
        </label>
        <div className="flex flex-wrap gap-2">
          <button className="btn btn-ghost" onClick={() => void simulateQuote()}>
            Quote
          </button>
          <button className="btn" disabled={!connected} onClick={() => void propose()}>
            Propose
          </button>
        </div>
        {quote && <p className="text-[12px] font-mono break-all">{quote}</p>}
        {msg && <p className="text-[12px] font-mono break-all">{msg}</p>}
      </div>
      <div className="clay p-5">
        <h2 className="font-semibold">Rules</h2>
        {rules.length === 0 ? (
          <p className="mt-2 text-[14px] text-[var(--mk-muted)]">None yet.</p>
        ) : (
          <ul className="mt-3 grid gap-2 text-[13px] font-mono">
            {rules.map((r) => (
              <li key={r.id}>
                {r.status} · ${r.usd_per_period} · {r.mint.slice(0, 8)}…
              </li>
            ))}
          </ul>
        )}
      </div>
    </Page>
  );
}
