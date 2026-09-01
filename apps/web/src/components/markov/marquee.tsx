import { rowLabel, SAMPLE_TAPE } from "@/lib/tape";
import { cn } from "@/lib/utils";

export function TapeMarquee() {
  const loop = [...SAMPLE_TAPE, ...SAMPLE_TAPE, ...SAMPLE_TAPE, ...SAMPLE_TAPE];

  return (
    <div className="relative overflow-hidden border-y border-line bg-raised" aria-hidden="true">
      <div className="marquee-track flex w-max gap-8 py-3">
        {loop.map((row, i) => (
          <span key={`${row.t}-${i}`} className="flex items-center gap-3 font-mono text-xs whitespace-nowrap">
            <span className="text-subtle">{row.t}</span>
            <span>{row.action}</span>
            <span
              className={cn(
                row.result === "blocked" && "text-refuse",
                row.result === "allowed" && "text-allow",
                row.result === "skip" && "text-subtle",
              )}
            >
              {rowLabel(row)}
            </span>
          </span>
        ))}
      </div>
    </div>
  );
}
