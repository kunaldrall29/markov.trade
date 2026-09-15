import { createFileRoute } from "@tanstack/react-router";
import { GATE_B_MANDATE, explorerAccount, short } from "@markov/sdk";
import { useBookStats, useReceiptFeed } from "@/lib/data/queries";
import { CircuitChip } from "@/components/desk/circuit";
import { DegradedBanner } from "@/components/desk/degraded";
import { ReceiptList } from "@/components/desk/receipt-list";
import { StatStrip } from "@/components/desk/stat-strip";
import { VerbBar } from "@/components/desk/verbs";
import { Film } from "@/components/markov/film";
import { Reveal } from "@/components/markov/reveal";
import { SiteShell } from "@/components/markov/site-shell";
import { StageLine } from "@/components/markov/stage";

export const Route = createFileRoute("/book")({ component: BookPage });

function BookPage() {
  const stats = useBookStats();
  const feed = useReceiptFeed({ mandate: GATE_B_MANDATE, limit: 50 });
  const degraded = stats.isError || feed.isError;
  const failing = [stats.isError ? `book stats: ${stats.error?.message}` : null, feed.isError ? `receipts: ${feed.error?.message}` : null].filter((x): x is string => !!x);
  const lastUpdated = stats.dataUpdatedAt || feed.dataUpdatedAt || null;
  const s = stats.data;

  return (
    <SiteShell>
      <section className="relative overflow-hidden">
        <div className="absolute inset-0 opacity-40">
          <Film src="/images/hero-desk.jpg" alt="" className="h-full min-h-[24rem]" imgClassName="ken" />
        </div>
        <div className="absolute inset-0 bg-linear-to-b from-bg/40 via-bg/80 to-bg" />
        <div className="relative mx-auto max-w-6xl px-5 pb-10 pt-10">
          <StageLine />
          <h1 className="mt-4 text-4xl font-semibold tracking-tight md:text-6xl">BOOK_ONE</h1>
          <p className="mt-4 max-w-lg text-sm leading-relaxed text-muted">
            The house book on Solana devnet. Read from the chain every five seconds. Marked PnL, not a promised rate. Unaudited. Withdraw stays on in every state.
          </p>
          <p className="mt-3 font-mono text-nano text-subtle">
            mandate{" "}
            <a href={explorerAccount(GATE_B_MANDATE)} target="_blank" rel="noreferrer" className="underline underline-offset-2">
              {short(GATE_B_MANDATE, 8, 8)}
            </a>
            {s ? ` · owner ${short(s.mandate.owner)} · operator ${short(s.mandate.operator)} · slot ${s.data_slot}` : ""}
          </p>
        </div>
      </section>

      <section className="mx-auto max-w-6xl px-5 pb-20">
        {degraded ? (
          <div className="mb-4">
            <DegradedBanner failing={failing} lastUpdated={lastUpdated} />
          </div>
        ) : null}

        <div className="relative overflow-hidden rounded-md bg-surface shadow-hairline" data-testid="desk">
          <div className="pointer-events-none absolute inset-x-0 top-0 h-px overflow-hidden">
            <div className="scan-bar h-8 w-full bg-linear-to-b from-accent/40 to-transparent" />
          </div>
          <div className="flex items-center justify-between gap-3 border-b border-line px-4 py-3">
            <span className="flex items-center gap-2">
              <span className="live-dot size-2 rounded-full bg-allow" />
              <span className="font-mono text-micro text-allow">BOOK_ONE</span>
            </span>
            <span className="font-mono text-micro text-subtle">{s ? `${s.mandate.state.toLowerCase()} · chain` : stats.isLoading ? "reading the chain…" : "not read"}</span>
          </div>

          {s ? (
            <>
              <StatStrip stats={s} stale={degraded} />
              <CircuitChip circuit={s.circuit} markAge={s.mark.pyth?.age_secs ?? null} />
            </>
          ) : (
            <div className="px-4 py-8 font-mono text-sm text-subtle">{stats.isLoading ? "reading the mandate, the vault, the venue position and the mark…" : "the chain could not be read; nothing is shown in its place"}</div>
          )}

          <div className="border-t border-line">
            <ReceiptList rows={feed.data?.receipts ?? []} empty={feed.isLoading ? "reading receipts…" : feed.isError ? "receipts could not be read" : "no receipts in this window"} maxHeight="max-h-80" />
          </div>

          <div className="border-t border-line p-3">
            {s ? (
              <VerbBar target={{ address: s.mandate.address, vault: s.mandate.vault.address, mint: s.mandate.mint, decimals: s.mandate.vault.decimals, owner: s.mandate.owner, state: s.mandate.state }} />
            ) : (
              <VerbBar target={{ address: GATE_B_MANDATE, vault: "", mint: "", decimals: 6, owner: "", state: "Active" }} />
            )}
            <p className="mt-2 px-1 font-mono text-nano text-subtle">
              Withdraw stays on in every state. These verbs act on this mandate only if the connected wallet owns it; your own mandates live under Account.
            </p>
          </div>
        </div>

        <div className="mt-10 grid gap-3 md:grid-cols-3">
          {[
            ["allowed", "A fill that passed every gate, reported by the venue through return data. Public ActionReceipt."],
            ["refused", "The interesting row. The BlockReason, verbatim, in mono, with the gate that said no."],
            ["skip", "Default. Circuit live, nothing to do. Skips are recorded in the paper log, not on chain."],
          ].map(([t, b], i) => (
            <Reveal key={t} delay={i * 70} className="rounded-md bg-surface p-5 shadow-hairline">
              <p className="font-mono text-micro uppercase tracking-eye text-subtle">{t}</p>
              <p className="mt-2 text-sm leading-relaxed text-muted">{b}</p>
            </Reveal>
          ))}
        </div>
        <p className="mt-6 max-w-2xl text-sm leading-relaxed text-muted">
          Net delta, gross and the daily-loss halt are enforced by the agent's guard in v0 and carry the <span className="font-mono text-xs">off-chain guard</span> marker; per-trade cap, daily cap, slippage, mark freshness and the venue and token allowlists are enforced by the program on chain.
        </p>
      </section>
    </SiteShell>
  );
}
