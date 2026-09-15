import { ExternalLink } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { formatUnits, short } from "@markov/sdk";
import type { ReceiptRow } from "@/lib/api-types";
import { cn } from "@/lib/utils";

function when(row: ReceiptRow): string {
  if (row.blockTime == null) return `slot ${row.slot}`;
  const d = new Date(row.blockTime * 1000);
  return d.toISOString().slice(11, 16) + "Z";
}

function dateOf(row: ReceiptRow): string {
  return row.blockTime == null ? "" : new Date(row.blockTime * 1000).toISOString().slice(0, 10);
}

export function verdictOf(row: ReceiptRow): { text: string; tone: "allow" | "refuse" | "owner" | "error" } {
  if (row.txError) return { text: "tx failed", tone: "error" };
  if (row.kind === "refusal") return { text: row.reason ?? "refused", tone: "refuse" };
  if (row.kind === "action") return { text: "allowed", tone: "allow" };
  return { text: row.ownerKind ?? "owner", tone: "owner" };
}

function label(row: ReceiptRow): string {
  if (row.kind === "action") return `${row.action} ${row.market} ${row.side}`;
  if (row.kind === "refusal") return `${row.action}${row.forced ? " (forced)" : ""}`;
  return `owner · ${row.ownerKind?.toLowerCase()}`;
}

function amount(row: ReceiptRow, decimals: number): string {
  if (row.kind === "owner") return row.amount && row.amount.raw !== "0" ? formatUnits(BigInt(row.amount.raw), decimals) : "—";
  return row.notional ? formatUnits(BigInt(row.notional.raw), decimals) : "—";
}

export function ReceiptLine({ row, animate, decimals = 6, showMandate = false }: { row: ReceiptRow; animate?: boolean; decimals?: number; showMandate?: boolean }) {
  const v = verdictOf(row);
  return (
    <li
      className={cn("grid grid-cols-[3.4rem_1fr_auto] items-center gap-2 px-4 py-2.5 font-mono text-xs tabular-nums md:grid-cols-[3.4rem_1fr_5.5rem_auto_1.5rem]", animate && "tape-in")}
      data-testid="receipt-row"
      data-kind={row.kind}
    >
      <span className="text-subtle" title={dateOf(row)}>
        {when(row)}
      </span>
      <span className="min-w-0 truncate">
        {label(row)}
        {showMandate ? <span className="ml-2 text-subtle">{short(row.mandate)}</span> : null}
        {row.kind === "refusal" && row.gateIndex != null ? <span className="ml-2 text-subtle">gate {row.gateIndex}</span> : null}
      </span>
      <span className="hidden text-right text-muted md:block">{amount(row, decimals)}</span>
      <span
        className={cn(
          "text-right text-micro font-medium",
          v.tone === "refuse" && "text-refuse",
          v.tone === "allow" && "text-allow",
          v.tone === "owner" && "text-muted",
          v.tone === "error" && "text-refuse line-through",
        )}
        data-testid="receipt-verdict"
      >
        {v.text}
      </span>
      <a
        href={row.explorer}
        target="_blank"
        rel="noreferrer"
        className="hidden justify-self-end text-subtle hover:text-fg md:block"
        aria-label={`Transaction ${short(row.signature, 6, 6)} on the explorer`}
        data-testid="receipt-explorer"
      >
        <ExternalLink className="size-3.5" strokeWidth={1.75} />
      </a>
      <a href={row.explorer} target="_blank" rel="noreferrer" className="col-span-3 -mt-1 truncate text-nano text-subtle underline-offset-2 hover:underline md:hidden">
        {short(row.signature, 8, 8)} ↗
      </a>
    </li>
  );
}

/**
 * The tape. Rows appear one at a time on first render (docs/13 §4) unless the
 * user prefers reduced motion; later refreshes append without re-animating.
 */
export function ReceiptList({
  rows,
  empty = "no receipts in this window",
  decimals = 6,
  showMandate = false,
  reveal = true,
  maxHeight,
}: {
  rows: ReceiptRow[];
  empty?: string;
  decimals?: number;
  showMandate?: boolean;
  reveal?: boolean;
  maxHeight?: string;
}) {
  const [shown, setShown] = useState(reveal ? 0 : rows.length);
  const seen = useRef<Set<string>>(new Set());
  const total = rows.length;

  useEffect(() => {
    if (!reveal) {
      setShown(total);
      return;
    }
    const reduced = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    if (reduced || seen.current.size > 0) {
      setShown(total);
      rows.forEach((r) => seen.current.add(r.signature + r.eventIndex));
      return;
    }
    let i = 0;
    setShown(0);
    const iv = window.setInterval(() => {
      i += 1;
      setShown(i);
      if (i >= total) {
        window.clearInterval(iv);
        rows.forEach((r) => seen.current.add(r.signature + r.eventIndex));
      }
    }, 350);
    return () => window.clearInterval(iv);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [total, reveal]);

  const visible = rows.slice(0, Math.min(shown, total));
  return (
    <ol className={cn("overflow-auto", maxHeight)} aria-live="polite" data-testid="receipt-list">
      {total === 0 ? (
        <li className="px-4 py-6 font-mono text-sm text-subtle" data-testid="receipt-empty">
          {empty}
        </li>
      ) : (
        visible.map((row, idx) => (
          <ReceiptLine key={`${row.signature}-${row.eventIndex}`} row={row} animate={idx === visible.length - 1 && !seen.current.has(row.signature + row.eventIndex)} decimals={decimals} showMandate={showMandate} />
        ))
      )}
    </ol>
  );
}
