"use client";

import { useEffect, useState } from "react";
import { api } from "@/lib/api";
import { Page } from "@/components/page";

export default function Venues() {
  const [caps, setCaps] = useState<Array<{ id: string; env: string; executable: boolean; execution_model: string; onchain_enforceable: boolean }>>([]);
  useEffect(() => {
    api<{ venues: typeof caps }>("/venues/capabilities").then((r) => setCaps(r.venues)).catch(() => undefined);
  }, []);
  return (
    <Page title="Venues" note="Deposit and withdraw happen on the venue. Markov does not custody. Phoenix stays read-only until gate P1.">
      <div className="grid md:grid-cols-3 gap-3">
        {caps.map((v) => (
          <div key={v.id} className="clay p-4">
            <div className="font-semibold capitalize">{v.id}</div>
            <div className="text-[12px] font-mono mt-1">{v.env}</div>
            <div className="mt-2 text-[13px]">{v.execution_model}</div>
            <div className="chip mt-3" style={{ background: v.executable ? "#e8edff" : "white" }}>
              {v.executable ? "executable" : v.id === "phoenix" ? "INTEGRATED · read-only" : "not executable"}
            </div>
            <p className="mt-2 text-[12px] text-[var(--mk-muted)]">on-chain enforce {v.onchain_enforceable ? "yes" : "no"}</p>
          </div>
        ))}
      </div>
    </Page>
  );
}
