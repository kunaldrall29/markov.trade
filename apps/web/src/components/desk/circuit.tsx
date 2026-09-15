import type { Circuit } from "@/lib/api-types";
import { cn } from "@/lib/utils";

const COPY: Record<Circuit, { label: string; tone: "allow" | "refuse" | "muted" }> = {
  live: { label: "circuit live", tone: "allow" },
  paused: { label: "paused", tone: "muted" },
  revoked: { label: "revoked", tone: "refuse" },
  expired: { label: "expired", tone: "refuse" },
  global_halt: { label: "global halt", tone: "refuse" },
  stale_mark: { label: "stale mark", tone: "refuse" },
};

export function CircuitChip({ circuit, markAge }: { circuit: Circuit; markAge: number | null }) {
  const c = COPY[circuit];
  return (
    <div className="flex flex-wrap items-center justify-between gap-x-4 gap-y-1 px-4 py-2 font-mono text-micro uppercase tracking-wider">
      <span className={cn(c.tone === "allow" && "text-allow", c.tone === "refuse" && "text-refuse", c.tone === "muted" && "text-muted")} data-testid="circuit">
        {c.label}
      </span>
      <span className="text-subtle">skip = default</span>
      <span className="text-subtle" data-testid="mark-age">
        {markAge == null ? "mark —" : `mark ${markAge}s old`}
      </span>
    </div>
  );
}
