import { useEffect, useRef, useState } from "react";
import { BOOK_STATS, OWNER_VERBS, rowLabel, SAMPLE_TAPE, type TapeRow } from "@/lib/tape";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";

function TapeLine({ row, animate }: { row: TapeRow; animate: boolean }) {
  const label = rowLabel(row);
  return (
    <li
      className={cn(
        "grid grid-cols-[3rem_1fr_auto] items-center gap-2 px-4 py-2.5 font-mono text-xs tabular-nums md:grid-cols-[3rem_1fr_4.5rem_auto]",
        animate && "tape-in",
      )}
    >
      <span className="text-subtle">{row.t}</span>
      <span className="truncate">{row.action}</span>
      <span className="hidden text-right text-muted md:block">{row.qty}</span>
      <span
        className={cn(
          "text-right text-micro font-medium uppercase",
          row.result === "blocked" && "text-refuse",
          row.result === "allowed" && "text-allow",
          row.result === "skip" && "text-subtle",
        )}
      >
        {label}
      </span>
    </li>
  );
}

export function Blotter({
  replayNonce = 0,
  holdOnRefusal = false,
}: {
  replayNonce?: number;
  holdOnRefusal?: boolean;
}) {
  const [shown, setShown] = useState(0);
  const [note, setNote] = useState("withdraw stays on in every state · sample tape, devnet");
  const reducedRef = useRef(false);

  useEffect(() => {
    reducedRef.current = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
  }, []);

  useEffect(() => {
    if (reducedRef.current) {
      setShown(SAMPLE_TAPE.length);
      return;
    }
    setShown(0);
    let i = 0;
    let hold = false;
    const tick = () => {
      i += 1;
      setShown(i);
      const last = SAMPLE_TAPE[i - 1];
      if (holdOnRefusal && last?.result === "blocked") {
        hold = true;
        window.setTimeout(() => {
          hold = false;
        }, 1200);
      }
      return i < SAMPLE_TAPE.length;
    };
    tick();
    const iv = window.setInterval(() => {
      if (hold) return;
      if (!tick()) window.clearInterval(iv);
    }, holdOnRefusal ? 450 : 700);
    return () => window.clearInterval(iv);
  }, [replayNonce, holdOnRefusal]);

  return (
    <div className="relative overflow-hidden rounded-md bg-surface shadow-hairline">
      <div className="pointer-events-none absolute inset-x-0 top-0 h-px overflow-hidden">
        <div className="scan-bar h-8 w-full bg-linear-to-b from-accent/40 to-transparent" />
      </div>

      <div className="flex items-center justify-between gap-3 border-b border-line px-4 py-3">
        <span className="flex items-center gap-2">
          <span className="live-dot size-2 rounded-full bg-allow" />
          <span className="font-mono text-micro text-allow">BOOK_ONE</span>
        </span>
        <span className="font-mono text-micro text-subtle">marked · not a rate</span>
      </div>

      <dl className="grid grid-cols-2 gap-px bg-line sm:grid-cols-4">
        <div className="bg-raised px-4 py-3">
          <dt className="font-mono text-nano uppercase tracking-wider text-subtle">net delta</dt>
          <dd className="mt-1 font-mono text-xl tabular-nums">{BOOK_STATS.netDelta}</dd>
          <p className="font-mono text-nano text-subtle">{BOOK_STATS.netBand}</p>
        </div>
        <div className="bg-raised px-4 py-3">
          <dt className="font-mono text-nano uppercase tracking-wider text-subtle">gross</dt>
          <dd className="mt-1 font-mono text-xl tabular-nums">{BOOK_STATS.gross}</dd>
          <p className="font-mono text-nano text-subtle">{BOOK_STATS.cap}</p>
        </div>
        <div className="bg-raised px-4 py-3">
          <dt className="font-mono text-nano uppercase tracking-wider text-subtle">funding 7d</dt>
          <dd className="mt-1 font-mono text-xl tabular-nums">{BOOK_STATS.funding7d}</dd>
          <p className="font-mono text-nano text-subtle">{BOOK_STATS.fundingUnit}</p>
        </div>
        <div className="bg-raised px-4 py-3">
          <dt className="font-mono text-nano uppercase tracking-wider text-subtle">refusals</dt>
          <dd className="mt-1 font-mono text-xl tabular-nums text-refuse">{BOOK_STATS.refusals}</dd>
          <p className="font-mono text-nano text-subtle">{BOOK_STATS.refusalsWindow}</p>
        </div>
      </dl>

      <div className="flex items-center justify-between px-4 py-2 font-mono text-micro uppercase tracking-wider">
        <span className="text-allow">circuit live</span>
        <span className="text-subtle">skip = default</span>
      </div>

      <ol className="max-h-64 overflow-auto border-t border-line" aria-live="polite">
        {SAMPLE_TAPE.slice(0, shown).map((row, idx) => (
          <TapeLine key={`${row.t}-${idx}`} row={row} animate={idx === shown - 1} />
        ))}
      </ol>

      <div className="grid grid-cols-2 gap-2 border-t border-line p-3 sm:grid-cols-4">
        {OWNER_VERBS.map((verb) => (
          <Button
            key={verb.id}
            type="button"
            size="sm"
            variant={verb.id === "withdraw" ? "primary" : verb.id === "revoke" ? "kill" : verb.id === "pause" ? "quiet" : "ghost"}
            className="w-full"
            onClick={() => setNote(verb.body)}
          >
            {verb.label}
          </Button>
        ))}
      </div>
      <p className="px-3 pb-3 font-mono text-nano text-subtle">{note}</p>
    </div>
  );
}
