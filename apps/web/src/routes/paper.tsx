import { createFileRoute } from "@tanstack/react-router";
import { PAPER_DAYS } from "@/lib/paper";
import { Film } from "@/components/markov/film";
import { Reveal } from "@/components/markov/reveal";
import { SiteShell } from "@/components/markov/site-shell";
import { StageLine } from "@/components/markov/stage";

export const Route = createFileRoute("/paper")({ component: PaperPage });

function PaperPage() {
  return (
    <SiteShell>
      <section className="mx-auto max-w-6xl px-5 pb-8 pt-10">
        <StageLine />
        <h1 className="mt-4 text-4xl font-semibold tracking-tight md:text-6xl">The daily log.</h1>
        <p className="mt-4 max-w-lg text-sm leading-relaxed text-muted">
          One file per UTC day, written by the paper runner and committed unedited. Marks are marked. If a sentence would not survive a chain check or a paper log, it does not belong here.
        </p>
      </section>

      <section className="mx-auto grid max-w-6xl gap-8 px-5 pb-20 lg:grid-cols-[0.8fr_1.2fr]">
        <Film src="/images/still-life.jpg" alt="Desk still life. Pen, tape, cold coffee." className="aspect-photo rounded-md lg:sticky lg:top-24 lg:aspect-auto lg:h-[28rem]" />
        <ol className="space-y-3" data-testid="paper-days">
          {PAPER_DAYS.length === 0 ? <li className="font-mono text-sm text-subtle">no paper days committed yet</li> : null}
          {PAPER_DAYS.map((day, i) => (
            <Reveal key={day.date} delay={i * 60} as="li" className="rounded-md bg-surface p-5 shadow-hairline">
              <p className="font-mono text-micro text-subtle">{day.date}</p>
              <dl className="mt-3 grid gap-x-4 gap-y-1 font-mono text-xs sm:grid-cols-[max-content_1fr]">
                {day.fields
                  .filter((f) => f.key !== "date")
                  .map((f) => (
                    <div key={f.key} className="contents">
                      <dt className="text-subtle">{f.key}</dt>
                      <dd className="break-words text-fg">{f.value || "—"}</dd>
                    </div>
                  ))}
              </dl>
            </Reveal>
          ))}
        </ol>
      </section>
    </SiteShell>
  );
}
