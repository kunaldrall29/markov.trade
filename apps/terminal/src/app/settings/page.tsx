"use client";

import { useEffect, useState } from "react";
import { cluster, rpcUrl, programId, apiUrl } from "@/lib/env";
import { api } from "@/lib/api";
import { Page } from "@/components/page";
import { getSessionPubkey } from "@/lib/session";

export default function Settings() {
  const [me, setMe] = useState<string | null>(getSessionPubkey());
  const [err, setErr] = useState<string | null>(null);
  useEffect(() => {
    api<{ pubkey: string }>("/auth/me")
      .then((r) => setMe(r.pubkey))
      .catch((e: Error) => setErr(e.message.includes("401") ? "no SIWS session" : e.message));
  }, []);
  return (
    <Page title="Settings" note="RPC is from config, not a free-form field that could silently switch clusters.">
      <div className="clay p-5 grid gap-2 text-[13px] font-mono break-all">
        <div>cluster {cluster}</div>
        <div>rpc {rpcUrl}</div>
        <div>program {programId}</div>
        <div>api {apiUrl}</div>
        <div>session {me ?? "none"}</div>
        {err && <div className="text-[var(--mk-muted)]">{err}</div>}
      </div>
    </Page>
  );
}
