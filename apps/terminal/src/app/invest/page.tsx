"use client";

import { TSLAX, NVDAX, SPYX, AAPLX } from "@markov/facts";
import { Page } from "@/components/page";

const ASSETS = [
  { s: "TSLAx", m: TSLAX },
  { s: "NVDAx", m: NVDAX },
  { s: "SPYx", m: SPYX },
  { s: "AAPLx", m: AAPLX },
];

export default function Invest() {
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
      <div className="clay p-5 text-[14px] text-[var(--mk-muted)]">
        No rules yet. Creating one takes two signatures: the invest mandate, then the USDC recurring delegation. Reference Safe is unavailable until an official-price feed is verified (gate I6).
      </div>
    </Page>
  );
}
