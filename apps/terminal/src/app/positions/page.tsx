"use client";

import { useEffect, useState } from "react";
import { useWallet } from "@solana/wallet-adapter-react";
import bs58 from "bs58";
import { api } from "@/lib/api";
import { Page } from "@/components/page";

type Pos = {
  symbol?: string;
  side?: string;
  amount?: string | number;
  entry_price?: string | number;
  mark?: string | number;
  [k: string]: unknown;
};

type Signable = { compact_json: string | null; display: string };

export default function Positions() {
  const { connected, signMessage } = useWallet();
  const [rows, setRows] = useState<Pos[]>([]);
  const [source, setSource] = useState("");
  const [msg, setMsg] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const load = () =>
    api<{ positions: Pos[]; source?: string; error?: string }>("/positions")
      .then((r) => {
        setRows(Array.isArray(r.positions) ? r.positions : []);
        setSource(r.source ?? r.error ?? "");
      })
      .catch((e: Error) => setMsg(e.message));

  useEffect(() => {
    load();
  }, []);

  async function reduce(row: Pos) {
    if (!signMessage) return;
    const market = `${String(row.symbol ?? "SOL")}-PERP`.replace("-PERP-PERP", "-PERP");
    const side = row.side === "ask" || row.side === "short" ? "long" : "short";
    const amount = Number(row.amount ?? 0);
    const mark = Number(row.mark ?? row.entry_price ?? 0);
    const notional = amount > 0 && mark > 0 ? amount * mark : 50;
    setBusy(true);
    try {
      const res = await api<{ request_id: string; signables?: Signable[] }>("/trades/reduce", {
        method: "POST",
        body: JSON.stringify({ market, side, notional_usd: Number(notional.toFixed(2)) }),
      });
      const compact = res.signables?.[0]?.compact_json;
      if (!compact) {
        setMsg("No signable — Pacifica mark may be stale.");
        return;
      }
      const sig = await signMessage(new TextEncoder().encode(compact));
      const out = await api<{ ok: boolean; note?: string }>("/trades/submit", {
        method: "POST",
        body: JSON.stringify({ request_id: res.request_id, signature: bs58.encode(sig) }),
      });
      setMsg(out.note ?? (out.ok ? "submitted" : "venue rejected"));
      await load();
    } catch (e) {
      setMsg(e instanceof Error ? e.message : "reduce failed");
    } finally {
      setBusy(false);
    }
  }

  return (
    <Page title="Positions" note="Cross-venue table from Pacifica /positions. Liquidation figures are venue estimates. Reduce/close require an owner signature.">
      {rows.length === 0 ? (
        <div className="clay p-6 text-[14px] text-[var(--mk-muted)]">
          No positions. Bind the wallet on Venues, then create the Pacifica testnet account if /account is 404.
          {source ? ` ${source}` : ""}
        </div>
      ) : (
        <div className="clay overflow-x-auto">
          <table className="w-full text-[13px]">
            <thead className="font-mono text-[11px] uppercase tracking-[0.08em] text-[var(--mk-muted)]">
              <tr>
                <th className="p-3 text-left">Symbol</th>
                <th className="p-3 text-left">Side</th>
                <th className="p-3 text-left">Size</th>
                <th className="p-3 text-left">Entry</th>
                <th className="p-3 text-left"></th>
              </tr>
            </thead>
            <tbody>
              {rows.map((row, i) => (
                <tr key={i} className="border-t border-black/5">
                  <td className="p-3 font-medium">{String(row.symbol ?? "—")}</td>
                  <td className="p-3">{String(row.side ?? "—")}</td>
                  <td className="p-3 num">{String(row.amount ?? "—")}</td>
                  <td className="p-3 num">{String(row.entry_price ?? "—")}</td>
                  <td className="p-3">
                    <button className="btn" disabled={!connected || busy} onClick={() => void reduce(row)}>
                      Reduce
                    </button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
      {msg && <p className="text-[12px] font-mono break-all">{msg}</p>}
    </Page>
  );
}
