import { createFileRoute } from "@tanstack/react-router";
import { Blotter } from "@/components/markov/blotter";
import { Film } from "@/components/markov/film";
import { Reveal } from "@/components/markov/reveal";
import { SiteShell } from "@/components/markov/site-shell";

export const Route = createFileRoute("/book")({ component: BookPage });

function BookPage() {
  return (
    <SiteShell>
      <section className="relative overflow-hidden">
        <div className="absolute inset-0 opacity-40">
          <Film src="/images/hero-desk.jpg" alt="" className="h-full min-h-[28rem]" imgClassName="ken" />
        </div>
        <div className="absolute inset-0 bg-linear-to-b from-bg/40 via-bg/80 to-bg" />
        <div className="relative mx-auto max-w-6xl px-5 pb-12 pt-10">
          <p className="eyebrow">Desk</p>
          <h1 className="mt-3 text-4xl font-semibold tracking-tight md:text-6xl">BOOK_ONE</h1>
          <p className="mt-4 max-w-lg text-sm leading-relaxed text-muted">
            House book on Solana perps. Sample tape, devnet. Marked PnL, not a promised rate. Unaudited. Withdraw stays on in every state.
          </p>
        </div>
      </section>

      <section className="mx-auto max-w-6xl px-5 pb-20">
        <Blotter />
        <div className="mt-10 grid gap-3 md:grid-cols-3">
          {[
            ["Allowed", "A fill that passed every gate. Public ActionReceipt."],
            ["OverTxCap", "The interesting row. BlockReason, verbatim, in mono."],
            ["Skip", "Default. Circuit live, nothing to do. Recorded, not hidden."],
          ].map(([t, b], i) => (
            <Reveal key={t} delay={i * 70} className="rounded-md bg-surface p-5 shadow-hairline">
              <p className="font-mono text-micro uppercase tracking-eye text-subtle">{t}</p>
              <p className="mt-2 text-sm leading-relaxed text-muted">{b}</p>
            </Reveal>
          ))}
        </div>
      </section>
    </SiteShell>
  );
}
