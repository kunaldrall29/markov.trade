import { useReceiptFeed } from "@/lib/data/queries";
import { verdictOf } from "@/components/desk/receipt-list";
import { cn } from "@/lib/utils";

/** The program's last receipts, scrolling. Decorative repetition of real rows; hidden when there is nothing real to show. */
export function TapeMarquee() {
  const feed = useReceiptFeed({ limit: 20 });
  const rows = feed.data?.receipts ?? [];
  if (rows.length === 0) return null;
  const loop = [...rows, ...rows];
  return (
    <div className="relative overflow-hidden border-y border-line bg-raised" aria-hidden="true">
      <div className="marquee-track flex w-max gap-8 py-3">
        {loop.map((row, i) => {
          const v = verdictOf(row);
          const t = row.blockTime ? new Date(row.blockTime * 1000).toISOString().slice(11, 16) : `slot ${row.slot}`;
          const label = row.kind === "owner" ? `owner ${row.ownerKind?.toLowerCase()}` : `${row.action} ${row.market ?? ""}`.trim();
          return (
            <span key={`${row.signature}-${row.eventIndex}-${i}`} className="flex items-center gap-3 font-mono text-xs whitespace-nowrap">
              <span className="text-subtle">{t}</span>
              <span>{label}</span>
              <span className={cn(v.tone === "refuse" && "text-refuse", v.tone === "allow" && "text-allow", (v.tone === "owner" || v.tone === "error") && "text-subtle")}>{v.text}</span>
            </span>
          );
        })}
      </div>
    </div>
  );
}
