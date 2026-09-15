"use client";

import { useParams } from "next/navigation";
import { useEffect, useState } from "react";
import { api } from "@/lib/api";
import { Page } from "@/components/page";

type Rule = {
  id: string;
  mint: string;
  usd_per_period: number;
  period_seconds: number;
  status: string;
  note?: string;
  on_chain?: boolean;
};

type Rec = { request_id: string; decision: string; reason: string; kind: string };

export default function RulePage() {
  const { id } = useParams<{ id: string }>();
  const [rule, setRule] = useState<Rule | null>(null);
  const [history, setHistory] = useState<Rec[]>([]);
  const [note, setNote] = useState<string>("");
  const [err, setErr] = useState<string | null>(null);

  useEffect(() => {
    api<{ rule: Rule; history: Rec[]; note?: string }>(`/invest/rules/${id}`)
      .then((r) => {
        setRule(r.rule);
        setHistory(r.history ?? []);
        setNote(r.note ?? "");
      })
      .catch((e: Error) => setErr(e.message));
  }, [id]);

  return (
    <Page title="Invest rule" note="A rule is identified by the on-chain invest mandate version plus schedule. Empty until one exists.">
      {err && <p className="text-[13px] text-[var(--mk-signal)]">{err}</p>}
      {!err && !rule && <div className="clay p-6 text-[14px] text-[var(--mk-muted)]">Loading…</div>}
      {rule && (
        <div className="clay p-5 grid gap-2 text-[14px]">
          <Row k="Id" v={rule.id} />
          <Row k="Mint" v={rule.mint} />
          <Row k="USD / period" v={`$${rule.usd_per_period}`} />
          <Row k="Period" v={`${rule.period_seconds}s`} />
          <Row k="Status" v={rule.status} />
          <Row k="On chain" v={rule.on_chain ? "yes" : "no"} />
          <p className="text-[12px] text-[var(--mk-muted)] mt-2">{rule.note ?? note}</p>
        </div>
      )}
      {history.length > 0 && (
        <ul className="clay p-4 font-mono text-[12px] grid gap-1">
          {history.map((h) => (
            <li key={h.request_id}>
              {h.kind} · {h.decision} · {h.reason}
            </li>
          ))}
        </ul>
      )}
    </Page>
  );
}

function Row({ k, v }: { k: string; v: string }) {
  return (
    <div className="flex justify-between gap-4 border-b border-black/5 py-2">
      <span className="text-[var(--mk-muted)]">{k}</span>
      <span className="font-medium text-right break-all">{v}</span>
    </div>
  );
}
