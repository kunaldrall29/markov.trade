"use client";

import { useEffect, useMemo, useState } from "react";
import { useRouter } from "next/navigation";

const ITEMS: Array<{ label: string; href: string; hint: string }> = [
  { label: "Overview", href: "/overview", hint: "equity, venues, receipts" },
  { label: "Markets", href: "/markets", hint: "live marks" },
  { label: "SOL-PERP", href: "/markets/SOL-PERP", hint: "ticket" },
  { label: "BTC-PERP", href: "/markets/BTC-PERP", hint: "ticket" },
  { label: "ETH-PERP", href: "/markets/ETH-PERP", hint: "ticket" },
  { label: "Portfolio", href: "/portfolio", hint: "empty until linked" },
  { label: "Positions", href: "/positions", hint: "venue positions" },
  { label: "Invest", href: "/invest", hint: "xStocks rules" },
  { label: "Mandate", href: "/mandate", hint: "hard rules" },
  { label: "Risk", href: "/risk", hint: "headroom" },
  { label: "Receipts", href: "/receipts", hint: "decisions" },
  { label: "Approvals", href: "/approvals", hint: "MCP inbox" },
  { label: "Venues", href: "/venues", hint: "link Pacifica" },
  { label: "Integrations", href: "/integrations", hint: "MCP tools" },
  { label: "Settings", href: "/settings", hint: "RPC, cluster" },
];

export function Palette() {
  const router = useRouter();
  const [open, setOpen] = useState(false);
  const [q, setQ] = useState("");
  const hits = useMemo(
    () => ITEMS.filter((i) => `${i.label} ${i.hint} ${i.href}`.toLowerCase().includes(q.toLowerCase())),
    [q],
  );

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "k") {
        e.preventDefault();
        setOpen((v) => !v);
      }
      if (e.key === "Escape") setOpen(false);
    };
    const onOpen = () => setOpen(true);
    window.addEventListener("keydown", onKey);
    window.addEventListener("markov:palette", onOpen);
    return () => {
      window.removeEventListener("keydown", onKey);
      window.removeEventListener("markov:palette", onOpen);
    };
  }, []);

  if (!open) return null;
  return (
    <div className="fixed inset-0 z-50 bg-black/30 p-4" onClick={() => setOpen(false)}>
      <div className="mx-auto mt-[12vh] max-w-lg clay p-3" onClick={(e) => e.stopPropagation()}>
        <input
          autoFocus
          className="w-full rounded-2xl bg-white/70 px-3 py-3 text-[14px] outline-none"
          placeholder="Go to…"
          value={q}
          onChange={(e) => setQ(e.target.value)}
        />
        <ul className="mt-2 max-h-72 overflow-auto">
          {hits.map((i) => (
            <li key={i.href}>
              <button
                type="button"
                className="flex w-full items-center justify-between rounded-xl px-3 py-2 text-left text-[14px] hover:bg-white/70"
                onClick={() => {
                  router.push(i.href);
                  setOpen(false);
                  setQ("");
                }}
              >
                <span>{i.label}</span>
                <span className="text-[11px] font-mono text-[var(--mk-muted)]">{i.hint}</span>
              </button>
            </li>
          ))}
        </ul>
        <p className="mt-2 px-1 text-[11px] text-[var(--mk-muted)]">⌘K / Ctrl+K</p>
      </div>
    </div>
  );
}
