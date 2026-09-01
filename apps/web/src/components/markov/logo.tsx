import { Link } from "@tanstack/react-router";
import { cn } from "@/lib/utils";

export function MarkGlyph({ className }: { className?: string }) {
  return (
    <svg viewBox="0 0 28 32" className={cn("h-[0.96em] w-[0.84em] shrink-0", className)} aria-hidden="true">
      <path
        fill="currentColor"
        d="M2.4 30V2h4.9L14 17.6 20.7 2h4.9v28h-3.7V12.6L14 26.4 6.1 12.6V30H2.4z"
      />
      <circle cx="14" cy="17.6" r="1.35" fill="var(--mk-refuse)" />
    </svg>
  );
}

export function Wordmark({
  className,
  to = "/",
}: {
  className?: string;
  to?: "/";
}) {
  return (
    <Link
      to={to}
      aria-label="Markov"
      className={cn(
        "inline-flex items-center gap-[0.12em] font-semibold tracking-mark text-[15px]",
        className,
      )}
    >
      <MarkGlyph />
      <span>ARKOV</span>
    </Link>
  );
}
