"use client";

import { useEffect, useState } from "react";
import { useWallet } from "@solana/wallet-adapter-react";
import { api } from "@/lib/api";
import { Page } from "@/components/page";

type Proposal = { id: string; kind: string; actor: string; status: string; created_at: string; payload: Record<string, unknown> };

export default function Approvals() {
  const { connected } = useWallet();
  const [rows, setRows] = useState<Proposal[] | null>(null);
  const [msg, setMsg] = useState<string | null>(null);

  const load = () =>
    api<{ proposals: Proposal[] }>("/approvals")
      .then((r) => setRows(r.proposals ?? []))
      .catch((e: Error) => setMsg(e.message));

  useEffect(() => {
    load();
    const t = setInterval(load, 4000);
    return () => clearInterval(t);
  }, []);

  async function act(id: string, verb: "sign" | "decline") {
    const res = await api<{ status: string }>(`/approvals/${id}/${verb}`, { method: "POST", body: "{}" });
    setMsg(`${id.slice(0, 8)} · ${res.status}`);
    await load();
  }

  return (
    <Page title="Approvals" note="MCP proposals land here. The model never receives a signable. Sign or decline; both are receipted.">
      {rows === null ? (
        <div className="clay p-6 text-[14px] text-[var(--mk-muted)]">Loading…</div>
      ) : rows.length === 0 ? (
        <div className="clay p-6 text-[14px] text-[var(--mk-muted)]">Inbox empty.</div>
      ) : (
        <ul className="grid gap-3">
          {rows.map((p) => (
            <li key={p.id} className="clay p-4">
              <div className="flex flex-wrap justify-between gap-2">
                <span className="font-medium">{p.kind}</span>
                <span className="chip" style={{ background: "white" }}>
                  {p.status}
                </span>
              </div>
              <pre className="mt-2 overflow-auto text-[11px] font-mono max-h-40">{JSON.stringify(p.payload, null, 2)}</pre>
              <div className="mt-3 flex gap-2">
                <button className="btn" disabled={!connected} onClick={() => void act(p.id, "sign")}>
                  Sign
                </button>
                <button className="btn btn-ghost" disabled={!connected} onClick={() => void act(p.id, "decline")}>
                  Decline
                </button>
              </div>
            </li>
          ))}
        </ul>
      )}
      {msg && <p className="text-[12px] font-mono">{msg}</p>}
    </Page>
  );
}
