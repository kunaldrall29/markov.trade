"use client";

import { Page } from "@/components/page";

export default function Portfolio() {
  return (
    <Page title="Portfolio" note="Aggregate equity, exposure and funding appear after a venue account is linked. Nothing is invented here.">
      <div className="grid sm:grid-cols-3 gap-3">
        <Card k="Equity" v="—" />
        <Card k="Gross notional" v="—" />
        <Card k="Net delta" v="—" />
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
