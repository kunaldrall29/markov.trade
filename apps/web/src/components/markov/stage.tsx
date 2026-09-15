import { stageLine } from "@/lib/config";

/** docs/13 §7: every page carries this above the fold. */
export function StageLine() {
  return (
    <p className="inline-flex items-center gap-2 rounded-full px-3 py-1 font-mono text-micro text-muted shadow-hairline" data-testid="stage-line">
      <span className="size-1.5 rounded-full bg-allow" aria-hidden="true" />
      {stageLine}
    </p>
  );
}
