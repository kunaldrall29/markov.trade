import { createFileRoute } from "@tanstack/react-router";
import { useMemo, useState } from "react";
import { BLOCK_REASON_NAMES, MANDATE_PROGRAM_ID, explorerAccount, short } from "@markov/sdk";
import type { ReceiptRow } from "@/lib/api-types";
import { useHealth, useReceiptFeed } from "@/lib/data/queries";
import { cn } from "@/lib/utils";
import { DegradedBanner } from "@/components/desk/degraded";
import { ReceiptList } from "@/components/desk/receipt-list";
import { SiteShell } from "@/components/markov/site-shell";
import { StageLine } from "@/components/markov/stage";
import { Button } from "@/components/ui/button";

export const Route = createFileRoute("/receipts")({ component: ReceiptsPage });

type Filter = "all" | "allowed" | "refused" | "owner";

function matches(row: ReceiptRow, filter: Filter, reason: string): boolean {
  if (filter === "allowed" && row.kind !== "action") return false;
  if (filter === "refused" && row.kind !== "refusal") return false;
  if (filter === "owner" && row.kind !== "owner") return false;
  if (reason && row.reason !== reason) return false;
  return true;
}

function ReceiptsPage() {
  const [filter, setFilter] = useState<Filter>("all");
  const [reason, setReason] = useState("");
  const [pages, setPages] = useState<string[]>([]);
  const before = pages[pages.length - 1];
  const feed = useReceiptFeed({ limit: 50, before });
  const health = useHealth();
  const rows = useMemo(() => (feed.data?.receipts ?? []).filter((r) => matches(r, filter, reason)), [feed.data, filter, reason]);
  const failing = [feed.isError ? `receipts: ${feed.error?.message}` : null, health.data && !health.data.chainReady ? `rpc: ${health.data.rpc.error ?? health.data.failing.join(", ")}` : null].filter((x): x is string => !!x);

  return (
    <SiteShell>
      <section className="mx-auto max-w-6xl px-5 pb-8 pt-10">
        <StageLine />
        <h1 className="mt-4 text-4xl font-semibold tracking-tight md:text-6xl">Activity.</h1>
        <p className="mt-4 max-w-lg text-sm leading-relaxed text-muted">
          Every allow, every refusal and every owner action the program has emitted, decoded from inner-instruction data by IDL. Newest first. A zero-refusal book is ordinary; the interesting row is the one that says no.
        </p>
        <p className="mt-3 font-mono text-nano text-subtle">
          program{" "}
          <a href={explorerAccount(MANDATE_PROGRAM_ID)} target="_blank" rel="noreferrer" className="underline underline-offset-2">
            {short(MANDATE_PROGRAM_ID, 8, 8)}
          </a>
          {feed.data ? ` · slot ${feed.data.data_slot} · ${feed.data.signatures} signatures in this page` : ""}
        </p>
      </section>

      <div className="mx-auto max-w-6xl px-5 pb-20">
        {failing.length ? (
          <div className="mb-4">
            <DegradedBanner failing={failing} lastUpdated={feed.dataUpdatedAt || null} />
          </div>
        ) : null}
        <div className="mb-4 flex flex-wrap items-center gap-2">
          {(["all", "allowed", "refused", "owner"] as const).map((id) => (
            <button
              key={id}
              type="button"
              onClick={() => setFilter(id)}
              className={cn("min-h-10 rounded-sm px-3 font-mono text-micro uppercase tracking-eye", filter === id ? "bg-accent text-accent-fg" : "bg-raised text-muted hover:text-fg")}
              data-testid={`filter-${id}`}
            >
              {id}
            </button>
          ))}
          <select
            value={reason}
            onChange={(e) => setReason(e.target.value)}
            aria-label="Filter by refusal reason"
            className="min-h-10 rounded-sm bg-raised px-3 font-mono text-micro uppercase tracking-eye text-muted"
          >
            <option value="">any reason</option>
            {BLOCK_REASON_NAMES.map((n) => (
              <option key={n} value={n}>
                {n}
              </option>
            ))}
          </select>
        </div>
        <div className="overflow-hidden rounded-md bg-surface shadow-hairline">
          <ReceiptList rows={rows} showMandate empty={feed.isLoading ? "reading receipts…" : feed.isError ? "receipts could not be read" : filter === "refused" ? "no refusals in this window" : "no receipts in this window"} />
        </div>
        <div className="mt-3 flex flex-wrap items-center justify-between gap-2">
          <p className="font-mono text-nano text-subtle">live feed · chain · refreshes every 5 s while visible · explorer links open on devnet</p>
          <div className="flex gap-2">
            {pages.length ? (
              <Button type="button" size="sm" variant="ghost" onClick={() => setPages((p) => p.slice(0, -1))}>
                Newer
              </Button>
            ) : null}
            {feed.data?.before && feed.data.signatures >= 50 ? (
              <Button type="button" size="sm" variant="ghost" onClick={() => setPages((p) => [...p, feed.data!.before!])} data-testid="older">
                Older
              </Button>
            ) : null}
          </div>
        </div>
      </div>
    </SiteShell>
  );
}
