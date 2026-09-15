"use client";

import { useEffect, useState } from "react";
import { api } from "@/lib/api";
import { Page } from "@/components/page";

export default function Positions() {
  const [rows, setRows] = useState<unknown[]>([]);
  const [source, setSource] = useState<string>("");
  useEffect(() => {
    api<{ positions: unknown[]; source?: string }>("/positions")
      .then((r) => {
        setRows(r.positions ?? []);
        setSource(r.source ?? "");
      })
      .catch(() => undefined);
  }, []);
  return (
    <Page title="Positions" note="Cross-venue table. Liquidation figures are venue estimates. Reduce/close require an owner signature.">
      {rows.length === 0 ? (
        <div className="clay p-6 text-[14px] text-[var(--mk-muted)]">No positions. Link Pacifica on /venues. {source}</div>
      ) : (
        <pre className="clay p-4 overflow-auto text-[12px] font-mono">{JSON.stringify(rows, null, 2)}</pre>
      )}
    </Page>
  );
}
