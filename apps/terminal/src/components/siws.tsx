"use client";

import { useEffect, useRef, useState } from "react";
import { useWallet } from "@solana/wallet-adapter-react";
import bs58 from "bs58";
import { api } from "@/lib/api";
import { clearSession, getSessionPubkey, setSession } from "@/lib/session";

export function SiwsGate({ children }: { children: React.ReactNode }) {
  const { connected, publicKey, signMessage, disconnecting } = useWallet();
  const [status, setStatus] = useState<"idle" | "signing" | "ok" | "err">("idle");
  const [detail, setDetail] = useState<string | null>(null);
  const last = useRef<string | null>(null);

  useEffect(() => {
    if (disconnecting || !connected || !publicKey) {
      if (!connected) {
        clearSession();
        last.current = null;
        setStatus("idle");
      }
      return;
    }
    const pk = publicKey.toBase58();
    if (getSessionPubkey() === pk) {
      last.current = pk;
      setStatus("ok");
      return;
    }
    if (!signMessage) {
      setStatus("err");
      setDetail("This wallet cannot sign messages. Reads still work.");
      return;
    }
    if (last.current === pk) return;
    last.current = pk;
    let cancelled = false;
    (async () => {
      try {
        setStatus("signing");
        const ch = await api<{ nonce: string; message: string }>("/auth/challenge", {
          method: "POST",
          body: JSON.stringify({ pubkey: pk }),
        });
        const sig = await signMessage(new TextEncoder().encode(ch.message));
        const verified = await api<{ token: string }>("/auth/verify", {
          method: "POST",
          body: JSON.stringify({ pubkey: pk, signature: bs58.encode(sig), nonce: ch.nonce }),
        });
        if (cancelled) return;
        setSession(verified.token, pk);
        setStatus("ok");
        setDetail(null);
      } catch (e) {
        if (cancelled) return;
        setStatus("err");
        setDetail(e instanceof Error ? e.message : "SIWS failed");
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [connected, publicKey, signMessage, disconnecting]);

  return (
    <>
      {status === "signing" && (
        <div className="px-4 py-2 text-[12px] text-[var(--mk-muted)]">Signing in with Solana…</div>
      )}
      {status === "err" && detail && (
        <div className="px-4 py-2 text-[12px] text-[var(--mk-signal)]">Wallet connected, session not verified. {detail}</div>
      )}
      {children}
    </>
  );
}
