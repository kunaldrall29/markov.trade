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
        <div className="bg-raised px-4 py-3">
          <dt className="font-mono text-nano uppercase tracking-wider text-subtle">net delta</dt>
          <dd className="mt-1 font-mono text-xl tabular-nums">{s ? formatUnits(net, d) : "—"}</dd>
          <p className="font-mono text-nano text-subtle">{s ? `±${formatUnits(BigInt(s.offchain_limits.delta_band.raw), d, { min: 0 })} · off-chain guard` : ""}</p>
        </div>
        <div className="bg-raised px-4 py-3">
          <dt className="font-mono text-nano uppercase tracking-wider text-subtle">gross</dt>
          <dd className="mt-1 font-mono text-xl tabular-nums">{s ? formatUnits(gross, d) : "—"}</dd>
          <p className="font-mono text-nano text-subtle">{s ? `cap ${formatUnits(BigInt(s.offchain_limits.max_gross.raw), d, { min: 0 })} · off-chain guard` : ""}</p>
        </div>
        <div className="bg-raised px-4 py-3">
          <dt className="font-mono text-nano uppercase tracking-wider text-subtle">vault</dt>
          <dd className="mt-1 font-mono text-xl tabular-nums">{s ? formatUnits(BigInt(s.mandate.vault.raw), d) : "—"}</dd>
          <p className="font-mono text-nano text-subtle">USDC-d · chain</p>
        </div>
        <div className="bg-raised px-4 py-3">
          <dt className="font-mono text-nano uppercase tracking-wider text-subtle">refusals</dt>
          <dd className={`mt-1 font-mono text-xl tabular-nums ${s && s.window.refusals > 0 ? "text-refuse" : "text-subtle"}`}>{s ? s.window.refusals : "—"}</dd>
          <p className="font-mono text-nano text-subtle">{s ? (s.window.refusals === 0 ? "none in 24h" : "24h") : ""}</p>
        </div>
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
