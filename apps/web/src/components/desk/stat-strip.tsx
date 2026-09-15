import { formatPriceE6, formatUnits } from "@markov/sdk";
import type { BookStats } from "@/lib/api-types";
import { Stat } from "@/components/ui/field";
import { cn } from "@/lib/utils";

/** The four counters docs/13 §6 draws, plus the off-chain-guard marker where ADR-005 requires it. */
export function StatStrip({ stats, stale }: { stats: BookStats; stale: boolean }) {
  const d = stats.mandate.vault.decimals;
  const pos = stats.position;
  const gross = pos ? BigInt(pos.notional.raw) : 0n;
  const net = pos ? (pos.side === "short" ? -gross : gross) : 0n;
  const mark = stats.mark.pyth?.price_e6 ?? stats.mark.venue?.price_e6 ?? null;
  return (
    <dl className={cn("grid grid-cols-2 gap-px bg-line sm:grid-cols-4", stale && "opacity-60")} data-testid="stat-strip">
      <Stat label="net delta" value={formatUnits(net, d)} note={`±${formatUnits(BigInt(stats.offchain_limits.delta_band.raw), d, { min: 0 })} · off-chain guard`} />
      <Stat label="gross" value={formatUnits(gross, d)} note={`cap ${formatUnits(BigInt(stats.offchain_limits.max_gross.raw), d, { min: 0 })} · off-chain guard`} />
      <Stat label="vault" value={formatUnits(BigInt(stats.mandate.vault.raw), d)} note={stats.mandate.vault.exists ? "USDC-d · chain" : "vault account missing"} />
      <Stat
        label="refusals"
        value={stats.window.refusals}
        tone={stats.window.refusals > 0 ? "refuse" : "muted"}
        note={stats.window.refusals === 0 ? "no refusals in this window" : `24h${stats.window.truncated ? " · window truncated" : ""}`}
      />
      <Stat label="mark" value={mark ? formatPriceE6(BigInt(mark)) : "—"} note={stats.mark.pyth ? `pyth · ${stats.mark.pyth.verification.toLowerCase()} · ${stats.mark.pyth.age_secs}s` : "no pyth account"} />
      <Stat label="day used" value={formatUnits(BigInt(stats.mandate.day.notional_used.raw), d)} note={`of ${formatUnits(BigInt(stats.mandate.policy.daily_cap.raw), d, { min: 0 })} daily cap · chain`} />
      <Stat label="per trade" value={formatUnits(BigInt(stats.mandate.policy.per_tx_cap.raw), d, { min: 0 })} note={`slippage ≤ ${stats.mandate.policy.max_slippage_bps} bps · chain`} />
      <Stat label="actions" value={stats.window.actions} note={`24h · fills reported by the venue`} tone="muted" />
    </dl>
  );
}
