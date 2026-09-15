"use client";

import { useEffect, useState } from "react";
import { api } from "@/lib/api";

export function ApiBanner() {
  const [ok, setOk] = useState<boolean | null>(null);
  const [detail, setDetail] = useState("");
  useEffect(() => {
    let alive = true;
    const load = () =>
      api<{ ok: boolean; pacifica_markets: number; program_deployed: boolean }>("/health")
        .then((h) => {
          if (!alive) return;
          setOk(h.ok);
          setDetail(
            `${h.pacifica_markets} pacifica · program ${h.program_deployed ? "on chain" : "undeployed"}`,
          );
        })
        .catch(() => {
          if (!alive) return;
          setOk(false);
          setDetail("API unreachable");
        });
    load();
    const t = setInterval(load, 8000);
    return () => {
      alive = false;
      clearInterval(t);
    };
  }, []);
  if (ok === null) return null;
  if (ok) {
    return <span className="hidden lg:inline text-[11px] font-mono text-[var(--mk-muted)]">{detail}</span>;
  }
  return (
    <span className="chip" style={{ background: "#fde4dc", color: "var(--mk-signal)" }}>
      API down
    </span>
  );
}
