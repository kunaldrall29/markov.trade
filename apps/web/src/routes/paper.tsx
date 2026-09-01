import { createFileRoute } from "@tanstack/react-router";
import { Film } from "@/components/markov/film";
import { Reveal } from "@/components/markov/reveal";
import { SiteShell } from "@/components/markov/site-shell";
import { PAPER_LOG } from "@/lib/tape";

export const Route = createFileRoute("/paper")({ component: PaperPage });

function PaperPage() {
  return (
    <SiteShell>
      <section className="mx-auto max-w-6xl px-5 pb-8 pt-10">
        <p className="eyebrow">Paper</p>
        <h1 className="mt-3 text-4xl font-semibold tracking-tight md:text-6xl">The daily log.</h1>
        <p className="mt-4 max-w-lg text-sm leading-relaxed text-muted">
          Marks are marked. If a sentence would not survive a chain check or a paper log, it does not belong here.
        </p>
      </section>

      <section className="mx-auto grid max-w-6xl gap-8 px-5 pb-20 lg:grid-cols-[0.8fr_1.2fr]">
        <Film
          src="/images/still-life.jpg"
          alt="Desk still life. Pen, tape, cold coffee."
          className="aspect-photo rounded-md lg:sticky lg:top-24 lg:aspect-auto lg:h-[28rem]"
        />
        <ol className="space-y-3">
          {PAPER_LOG.map((entry, i) => (
            <Reveal key={entry.date} delay={i * 60} as="li" className="rounded-md bg-surface p-5 shadow-hairline">
              <p className="font-mono text-micro text-subtle">{entry.date}</p>
              <h2 className="mt-2 text-xl font-semibold tracking-tight">{entry.title}</h2>
              <p className="mt-2 text-sm leading-relaxed text-muted">{entry.body}</p>
            </Reveal>
          ))}
        </ol>
      </section>
    </SiteShell>
  );
}
