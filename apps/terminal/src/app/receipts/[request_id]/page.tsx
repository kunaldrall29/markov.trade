"use client";

import { useParams } from "next/navigation";
import { useEffect, useState } from "react";
import { api } from "@/lib/api";

export default function Receipt() {
  const { request_id } = useParams<{ request_id: string }>();
  const [raw, setRaw] = useState<unknown>(null);
  const [err, setErr] = useState<string | null>(null);
  useEffect(() => {
    api(`/receipts/${request_id}`).then(setRaw).catch((e: Error) => setErr(e.message));
  }, [request_id]);
  return (
    <div className="grid gap-4">
      <h1 className="text-[28px] tracking-[-0.04em]" style={{ fontFamily: "var(--font-display)", fontWeight: 800 }}>Receipt</h1>
      {err && <p className="text-[var(--mk-signal)]">{err}</p>}
      <pre className="ink p-4 overflow-auto text-[12px] font-mono">{raw ? JSON.stringify(raw, null, 2) : "…"}</pre>
    </div>
  );
}
