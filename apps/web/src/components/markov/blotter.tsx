/**
 * The live blotter on the landing page: the house book's counters and its
 * last receipts, from the same read API the Desk uses. There is no sample
 * tape; when the chain cannot be read the blotter says so.
 */
import { Link } from "@tanstack/react-router";
import { GATE_B_MANDATE, formatUnits } from "@markov/sdk";
import { useBookStats, useReceiptFeed } from "@/lib/data/queries";
import { ReceiptList } from "@/components/desk/receipt-list";
import { CircuitChip } from "@/components/desk/circuit";
import { Button } from "@/components/ui/button";
import { Stat } from "@/components/ui/field";

export function Blotter({ limit = 8 }: { limit?: number }) {
  const stats = useBookStats();
  const feed = useReceiptFeed({ mandate: GATE_B_MANDATE, limit });
  const s = stats.data;
  const d = s?.mandate.vault.decimals ?? 6;
  const pos = s?.position ?? null;
  const gross = pos ? BigInt(pos.notional.raw) : 0n;
  const net = pos ? (pos.side === "short" ? -gross : gross) : 0n;
  const degraded = stats.isError || feed.isError;

  return (
    <div className="relative overflow-hidden rounded-md bg-surface shadow-hairline" data-testid="blotter">
      <div className="pointer-events-none absolute inset-x-0 top-0 h-px overflow-hidden">
        <div className="scan-bar h-8 w-full bg-linear-to-b from-accent/40 to-transparent" />
      </div>

      <div className="flex items-center justify-between gap-3 border-b border-line px-4 py-3">
        <span className="flex items-center gap-2">
          <span className={`live-dot size-2 rounded-full ${degraded ? "bg-refuse" : "bg-allow"}`} />
          <span className="font-mono text-micro text-allow">BOOK_ONE</span>
        </span>
        <span className="font-mono text-micro text-subtle">{degraded ? "chain not readable" : s ? `live · slot ${s.data_slot}` : "reading…"}</span>
      </div>

      <dl className="grid grid-cols-2 gap-px bg-line sm:grid-cols-4">
        <Stat label="net delta" value={s ? formatUnits(net, d) : "—"} note={s ? `±${formatUnits(BigInt(s.offchain_limits.delta_band.raw), d, { min: 0 })} · off-chain guard` : undefined} />
        <Stat label="gross" value={s ? formatUnits(gross, d) : "—"} note={s ? `cap ${formatUnits(BigInt(s.offchain_limits.max_gross.raw), d, { min: 0 })} · off-chain guard` : undefined} />
        <Stat label="vault" value={s ? formatUnits(BigInt(s.mandate.vault.raw), d) : "—"} note="USDC-d · chain" />
        <Stat label="refusals" value={s ? s.window.refusals : "—"} tone={s && s.window.refusals > 0 ? "refuse" : "muted"} note={s ? (s.window.refusals === 0 ? "none in 24h" : "24h") : undefined} />
      </dl>

      {s ? <CircuitChip circuit={s.circuit} markAge={s.mark.pyth?.age_secs ?? null} /> : null}

      <div className="border-t border-line">
        <ReceiptList rows={feed.data?.receipts ?? []} empty={feed.isLoading ? "reading receipts…" : feed.isError ? "receipts could not be read" : "no receipts in this window"} maxHeight="max-h-56" />
      </div>

      <div className="flex items-center justify-between gap-2 border-t border-line p-3">
        <p className="font-mono text-nano text-subtle">devnet · chain-read · every 5 s</p>
        <Button asChild size="sm" variant="ghost">
          <Link to="/book">Open the desk</Link>
        </Button>
      </div>
    </div>
  );
}
