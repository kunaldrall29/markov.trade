import { createFileRoute } from "@tanstack/react-router";
import { useMemo, useState } from "react";
import { Film } from "@/components/markov/film";
import { SiteShell } from "@/components/markov/site-shell";
import { rowLabel, SAMPLE_TAPE, type TapeRow } from "@/lib/tape";
import { cn } from "@/lib/utils";

export const Route = createFileRoute("/receipts")({ component: ReceiptsPage });

type Filter = "all" | TapeRow["result"];

function ReceiptsPage() {
  const [filter, setFilter] = useState<Filter>("all");
  const rows = useMemo(
    () => (filter === "all" ? SAMPLE_TAPE : SAMPLE_TAPE.filter((r) => r.result === filter)),
    [filter],
  );

  return (
    <SiteShell>
      <section className="mx-auto max-w-6xl px-5 pb-8 pt-10">
        <p className="eyebrow">Receipts</p>
        <h1 className="mt-3 text-4xl font-semibold tracking-tight md:text-6xl">The tape.</h1>
        <p className="mt-4 max-w-lg text-sm leading-relaxed text-muted">
          Every fill and every refusal. Sample tape, devnet. A zero-refusal book is ordinary. The interesting row is the one that says no.
        </p>
      </section>

      <div className="mx-auto grid max-w-6xl gap-8 px-5 pb-20 lg:grid-cols-[0.9fr_1.1fr]">
        <Film
          src="/images/tape.jpg"
          alt="Thermal receipt tape with a red refusal stamp."
          className="aspect-still rounded-md lg:aspect-auto lg:min-h-[28rem]"
        />
        <div>
          <div className="mb-4 flex flex-wrap gap-2">
            {(["all", "allowed", "blocked", "skip"] as const).map((id) => (
              <button
                key={id}
                type="button"
                onClick={() => setFilter(id)}
                className={cn(
                  "min-h-10 rounded-sm px-3 font-mono text-micro uppercase tracking-eye",
                  filter === id ? "bg-accent text-accent-fg" : "bg-raised text-muted hover:text-fg",
                )}
              >
                {id}
              </button>
            ))}
          </div>
          <ol className="overflow-hidden rounded-md bg-surface shadow-hairline">
            {rows.length === 0 ? (
              <li className="px-4 py-6 font-mono text-sm text-subtle">no refusals in this window</li>
            ) : (
              rows.map((row, i) => (
                <li
                  key={`${row.t}-${i}`}
                  className="grid grid-cols-[3rem_1fr_auto] items-center gap-2 border-b border-line px-4 py-3 font-mono text-xs last:border-b-0 md:grid-cols-[3rem_1fr_4.5rem_auto]"
                >
                  <span className="text-subtle">{row.t}</span>
                  <span className="truncate">{row.action}</span>
                  <span className="hidden text-right text-muted md:block">{row.qty}</span>
                  <span
                    className={cn(
                      "text-right text-micro font-medium uppercase",
                      row.result === "blocked" && "text-refuse",
                      row.result === "allowed" && "text-allow",
                      row.result === "skip" && "text-subtle",
                    )}
                  >
                    {rowLabel(row)}
                  </span>
                </li>
              ))
            )}
          </ol>
          <p className="mt-3 font-mono text-nano text-subtle">sample tape, devnet · not a live feed</p>
        </div>
      </div>
    </SiteShell>
  );
}
