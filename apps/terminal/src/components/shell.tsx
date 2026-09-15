"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { useState } from "react";
import { WalletMultiButton } from "@solana/wallet-adapter-react-ui";
import { useWallet } from "@solana/wallet-adapter-react";
import { programId, stageLabel } from "@/lib/env";

const NAV = [
  ["Overview", "/overview"],
  ["Markets", "/markets"],
  ["Portfolio", "/portfolio"],
  ["Positions", "/positions"],
  ["Invest", "/invest"],
  ["Mandate", "/mandate"],
  ["Risk", "/risk"],
  ["Receipts", "/receipts"],
  ["Approvals", "/approvals"],
  ["Venues", "/venues"],
  ["Integrations", "/integrations"],
  ["Settings", "/settings"],
] as const;

const MOBILE = [
  ["Overview", "/overview"],
  ["Markets", "/markets"],
  ["Invest", "/invest"],
  ["Receipts", "/receipts"],
] as const;

export function Shell({ children }: { children: React.ReactNode }) {
  const path = usePathname();
  const { connected, publicKey } = useWallet();
  const [more, setMore] = useState(false);
  const pk = publicKey?.toBase58();
  return (
    <div className="min-h-dvh lg:grid lg:grid-cols-[220px_1fr]">
      <aside className="hidden lg:flex flex-col gap-6 p-5">
        <Link href="/overview" className="flex items-center gap-3">
          <img src="/brand/markov-mark.svg" width={28} height={28} alt="Markov" />
          <span style={{ fontFamily: "var(--font-display)", fontWeight: 800, letterSpacing: "-0.04em", fontSize: 22 }}>
            markov
          </span>
        </Link>
        <nav className="flex flex-col gap-1 text-[14px] font-medium">
          {NAV.map(([label, href]) => {
            const on = path === href || path.startsWith(`${href}/`);
            return (
              <Link
                key={href}
                href={href}
                className={`rounded-full px-3 py-2 ${on ? "bg-[var(--mk-blue)] text-white" : "text-[var(--mk-graphite)] hover:bg-white/50"}`}
              >
                {label}
              </Link>
            );
          })}
        </nav>
        <p className="mt-auto text-[11px] text-[var(--mk-muted)] leading-5">
          Rules survive the model. Phoenix is integrated, not executable. No token.
        </p>
      </aside>
      <div className="flex min-h-dvh flex-col">
        <header className="sticky top-0 z-20 flex items-center gap-3 px-4 py-3 backdrop-blur-md bg-[color:rgb(236,235,230,0.72)]">
          <Link href="/overview" className="lg:hidden flex items-center gap-2">
            <img src="/brand/markov-mark.svg" width={24} height={24} alt="" />
          </Link>
          <span className="chip" style={{ background: "var(--mk-ink)", color: "var(--mk-blue-on-ink)" }}>
            {stageLabel}
          </span>
          <span className="chip hidden sm:inline-flex" style={{ background: "white" }}>
            mandate v0
          </span>
          <span className="num hidden md:inline text-[11px] text-[var(--mk-muted)] truncate max-w-[220px]">{programId}</span>
          <div className="ml-auto flex items-center gap-2">
            <span className="hidden sm:inline text-[12px] text-[var(--mk-muted)]">
              {connected && pk ? `${pk.slice(0, 4)}…${pk.slice(-4)}` : "read-only until connected"}
            </span>
            <WalletMultiButton />
          </div>
        </header>
        <main className="flex-1 px-4 pb-24 lg:pb-10 pt-2 max-w-[1400px] w-full mx-auto">{children}</main>
        <nav className="lg:hidden fixed bottom-0 inset-x-0 z-30 bg-[var(--mk-cream)]/95 backdrop-blur border-t border-black/5 grid grid-cols-5 text-[11px] font-medium">
          {MOBILE.map(([label, href]) => (
            <Link
              key={href}
              href={href}
              className={`py-3 text-center ${path.startsWith(href) ? "text-[var(--mk-blue)]" : "text-[var(--mk-muted)]"}`}
            >
              {label}
            </Link>
          ))}
          <button
            type="button"
            className={`py-3 ${more ? "text-[var(--mk-blue)]" : "text-[var(--mk-muted)]"}`}
            onClick={() => setMore((v) => !v)}
          >
            More
          </button>
        </nav>
        {more && (
          <div className="lg:hidden fixed inset-0 z-20 bg-black/30" onClick={() => setMore(false)}>
            <div
              className="absolute bottom-14 inset-x-0 clay mx-3 p-4 grid grid-cols-2 gap-2"
              onClick={(e) => e.stopPropagation()}
            >
              {NAV.filter(([, href]) => !MOBILE.some((m) => m[1] === href)).map(([label, href]) => (
                <Link
                  key={href}
                  href={href}
                  className="rounded-2xl bg-white/60 px-3 py-3 text-[13px]"
                  onClick={() => setMore(false)}
                >
                  {label}
                </Link>
              ))}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
