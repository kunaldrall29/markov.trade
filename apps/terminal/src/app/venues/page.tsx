"use client";

import { useEffect, useState } from "react";
import { useWallet } from "@solana/wallet-adapter-react";
import { api } from "@/lib/api";
import { Page } from "@/components/page";

type Cap = { id: string; env: string; executable: boolean; execution_model: string; onchain_enforceable: boolean };
type Account = { venue: string; linked: boolean; env: string; note: string; venue_account_found?: boolean | null };

export default function Venues() {
  const { connected } = useWallet();
  const [caps, setCaps] = useState<Cap[]>([]);
  const [accounts, setAccounts] = useState<Account[]>([]);
  const [msg, setMsg] = useState<string | null>(null);

  const load = () => {
    api<{ venues: Cap[] }>("/venues/capabilities").then((r) => setCaps(r.venues)).catch(() => undefined);
    api<{ accounts: Account[] }>("/venues/accounts").then((r) => setAccounts(r.accounts)).catch(() => undefined);
  };

  useEffect(() => {
    load();
  }, []);

  async function bind() {
    const res = await api<{ note: string; venue_account_found: boolean | null }>("/venues/pacifica/bind", {
      method: "POST",
      body: "{}",
    });
    setMsg(res.note);
    load();
  }

  return (
    <Page title="Venues" note="Deposit and withdraw happen on the venue. Markov does not custody. Phoenix stays read-only until gate P1.">
      <div className="grid md:grid-cols-3 gap-3">
        {caps.map((v) => {
          const acct = accounts.find((a) => a.venue === v.id);
          return (
            <div key={v.id} className="clay p-4">
              <div className="font-semibold capitalize">{v.id}</div>
              <div className="text-[12px] font-mono mt-1">{v.env}</div>
              <div className="mt-2 text-[13px]">{v.execution_model}</div>
              <div className="chip mt-3" style={{ background: v.executable ? "#e8edff" : "white" }}>
                {v.executable ? "executable" : v.id === "phoenix" ? "INTEGRATED · read-only" : "not executable"}
              </div>
              <p className="mt-2 text-[12px] text-[var(--mk-muted)]">on-chain enforce {v.onchain_enforceable ? "yes" : "no"}</p>
              {acct && <p className="mt-2 text-[12px] text-[var(--mk-muted)]">{acct.note}</p>}
              {v.id === "pacifica" && (
                <button className="btn mt-3 w-full" disabled={!connected} onClick={() => void bind()}>
                  Bind this wallet
                </button>
              )}
            </div>
          );
        })}
      </div>
      {msg && <p className="text-[13px] font-mono">{msg}</p>}
    </Page>
  );
}
