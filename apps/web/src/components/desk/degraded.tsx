/**
 * docs/13 §6 rule 5: degraded state is loud. When the read failed, the banner
 * names the failing term and the counters below are shown greyed with their
 * last-updated stamp; nothing is invented to fill the gap.
 */
export function DegradedBanner({ failing, lastUpdated }: { failing: string[]; lastUpdated: number | null }) {
  return (
    <div role="alert" className="rounded-md bg-refuse/10 px-4 py-3 font-mono text-xs leading-relaxed text-refuse shadow-hairline" data-testid="degraded-banner">
      <p className="font-medium uppercase tracking-eye">degraded · {failing.join(", ") || "chain read failed"}</p>
      <p className="mt-1 text-refuse/90">
        {lastUpdated ? `showing the state read at ${new Date(lastUpdated).toISOString().slice(11, 19)}Z; ` : "no state has been read yet; "}
        the numbers below are not live until this clears.
      </p>
    </div>
  );
}
