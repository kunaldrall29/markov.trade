import { useState } from "react";
import { Film } from "@/components/markov/film";
import { TapeMarquee } from "@/components/markov/marquee";
import { Reveal } from "@/components/markov/reveal";
import { Hero } from "./hero";
import { CinematicReel } from "./reel";

const FAQ = [
  {
    q: "Is this yield I can take home?",
    a: "No. This is Solana devnet. Marks are marked PnL, not a promised rate. Unaudited. If a number cannot be checked from chain or the venue, it is not on this desk.",
  },
  {
    q: "Who can withdraw?",
    a: "Only the owner. Withdrawal authority never leaves the owner. It stays enabled in Active, Paused, Revoked, and Expired.",
  },
  {
    q: "Is this a vault?",
    a: "MVP is separately managed accounts: one mandate per depositor. Funds do not commingle. Pooling is a later decision, default still SMA.",
  },
  {
    q: "Is the agent an LLM?",
    a: "No. The book-core is deterministic and must run if every model is down. The risk guard is thresholds. A model may propose. An LLM never signs.",
  },
  {
    q: "Where is the marketplace?",
    a: "Not until the house book has a public curve. Book One is the only strategy on this page.",
  },
  {
    q: "What's the APY?",
    a: "There isn't one on this page.",
  },
];

export function LandingPage() {
  const [replay, setReplay] = useState(0);

  function watchRefusal() {
    setReplay((n) => n + 1);
    document.getElementById("blotter")?.scrollIntoView({
      behavior: window.matchMedia("(prefers-reduced-motion: reduce)").matches ? "auto" : "smooth",
      block: "center",
    });
  }

  return (
    <>
      <CinematicReel />
      <Hero onWatchRefusal={watchRefusal} replayNonce={replay} />
      <TapeMarquee />

      <section className="mx-auto max-w-6xl px-5 py-24 md:py-32">
        <Reveal>
          <p className="eyebrow">The pitch</p>
        </Reveal>
        <div className="mt-6 space-y-2">
          {[
            "I earn the book.",
            "They cannot take the pile.",
            "Every no is on the tape.",
          ].map((line, i) => (
            <Reveal key={line} delay={i * 90}>
              <p className="text-4xl font-semibold leading-[1.05] tracking-tight md:text-6xl">{line}</p>
            </Reveal>
          ))}
        </div>
      </section>

      <section className="relative min-h-[78svh] overflow-hidden">
        <Film src="/images/room.jpg" alt="Empty dealing room after hours, one lamp still on." className="absolute inset-0" imgClassName="brightness-110" />
        <div className="absolute inset-0 bg-linear-to-t from-bg via-bg/25 to-transparent" />
        <div className="relative mx-auto flex min-h-[78svh] max-w-6xl items-end px-5 py-16">
          <Reveal className="max-w-xl">
            <p className="eyebrow">House book</p>
            <h2 className="mt-3 text-4xl font-semibold tracking-tight md:text-5xl">The room the book sits in.</h2>
            <p className="mt-4 max-w-md text-base leading-relaxed text-muted">
              A house agent runs one bounded book on Solana perps. You park USDC. You keep withdraw. The operator cannot take the pile.
            </p>
          </Reveal>
        </div>
      </section>

      <section className="mx-auto max-w-6xl px-5 py-24 md:py-32">
        <div className="grid gap-16">
          <article className="grid items-center gap-8 lg:grid-cols-2 lg:gap-14">
            <Reveal>
              <Film
                src="/images/still-life.jpg"
                alt="Overhead still life of a trading blotter, pen, and thermal tape."
                className="aspect-photo rounded-md"
              />
            </Reveal>
            <Reveal delay={80}>
              <p className="eyebrow">Job</p>
              <h2 className="mt-3 text-3xl font-semibold tracking-tight md:text-4xl">Idle USDC, no keys handed over.</h2>
              <p className="mt-4 max-w-md text-base leading-relaxed text-muted">
                Same job as the wrappers people already use. Someone good runs the book. You keep withdraw.
              </p>
            </Reveal>
          </article>

          <article className="grid items-center gap-8 lg:grid-cols-2 lg:gap-14">
            <Reveal className="lg:order-2">
              <Film
                src="/images/mandate.jpg"
                alt="Sealed mandate folder on a desk. The owner keeps the key."
                className="aspect-still rounded-md"
              />
            </Reveal>
            <Reveal delay={80} className="lg:order-1">
              <p className="eyebrow">Wedge</p>
              <h2 className="mt-3 text-3xl font-semibold tracking-tight md:text-4xl">Runtime bounds. Not a rest-time audit.</h2>
              <p className="mt-4 max-w-md text-base leading-relaxed text-muted">
                April 2026 passed its audits and still drained. This account will not let the operator empty it.
              </p>
            </Reveal>
          </article>

          <article className="grid items-center gap-8 lg:grid-cols-2 lg:gap-14">
            <Reveal>
              <Film
                src="/images/tape.jpg"
                alt="Thermal receipt tape coming off a printer, a red refusal stamp on the strip."
                className="aspect-still rounded-md"
              />
            </Reveal>
            <Reveal delay={80}>
              <p className="eyebrow">Proof</p>
              <h2 className="mt-3 text-3xl font-semibold tracking-tight md:text-4xl">Refusals are on the tape.</h2>
              <p className="mt-4 max-w-md text-base leading-relaxed text-muted">
                A zero-refusal book is ordinary. The interesting row is the one that says no.
              </p>
            </Reveal>
          </article>
        </div>
      </section>

      <section className="mx-auto max-w-6xl px-5 py-8 md:py-16">
        <Reveal>
          <p className="eyebrow">How the desk is allowed to work</p>
          <h2 className="mt-3 max-w-xl text-4xl font-semibold tracking-tight md:text-5xl">An LLM never signs.</h2>
        </Reveal>
        <div className="mt-10 grid gap-3 md:grid-cols-2">
          {[
            ["01", "Core", "Inventory bands, hedge ratio, max gross, kill conditions. Runs if every model is dead."],
            ["02", "Veto", "A model may propose. The guard may refuse. The refusal is a first-class event."],
            ["03", "Sidecar", "Funding, OI, liquidations, basis. Features — never an order router."],
            ["04", "Proof", "If a number cannot be checked from chain or venue APIs, it is not on this desk."],
          ].map(([n, title, body], i) => (
            <Reveal key={n} delay={i * 70} as="article" className="rounded-md bg-surface p-5 shadow-hairline transition-shadow hover:shadow-hairline-strong">
              <p className="font-mono text-micro text-subtle">{n}</p>
              <h3 className="mt-2 text-xl font-semibold tracking-tight">{title}</h3>
              <p className="mt-2 text-sm leading-relaxed text-muted">{body}</p>
            </Reveal>
          ))}
        </div>
      </section>

      <section className="mx-auto grid max-w-6xl items-center gap-10 px-5 py-24 md:py-32 lg:grid-cols-[0.9fr_1.1fr]">
        <Reveal>
          <Film
            src="/images/mark.jpg"
            alt="Machined brass letter M, the Markov mark as a desk object."
            className="aspect-square rounded-md"
          />
        </Reveal>
        <div>
          <Reveal>
            <p className="eyebrow">Not that</p>
            <h2 className="mt-3 text-4xl font-semibold tracking-tight md:text-5xl">A book. Not a costume.</h2>
          </Reveal>
          <div className="mt-8 grid gap-3 sm:grid-cols-2">
            {[
              ["HLP", "Native order-book MM", "We don't pretend a pool venue is that book"],
              ["JLP", "Long-biased basket + fees", "Different risk. Near delta-neutral"],
              ["Marketplace", "Strangers selling books", "Phase 3. After a public curve"],
              ["Token", "Ticker as the product", "No ticker in MVP"],
            ].map(([name, them, us], i) => (
              <Reveal key={name} delay={i * 60} className="rounded-md bg-surface p-5 shadow-hairline">
                <p className="text-lg font-semibold">{name}</p>
                <p className="mt-2 text-sm text-subtle">{them}</p>
                <p className="mt-1 text-sm text-allow">{us}</p>
              </Reveal>
            ))}
          </div>
        </div>
      </section>

      <section className="relative min-h-[62svh] overflow-hidden">
        <Film src="/images/leaving.jpg" alt="Owner walking away from the desk. The pile stays." className="absolute inset-0" imgClassName="brightness-105" />
        <div className="absolute inset-0 bg-linear-to-r from-bg via-bg/55 to-transparent" />
        <div className="relative mx-auto flex min-h-[62svh] max-w-6xl items-center px-5 py-16">
          <Reveal className="max-w-lg">
            <p className="eyebrow">Owner verbs</p>
            <h2 className="mt-3 text-4xl font-semibold tracking-tight md:text-5xl">Always able to leave.</h2>
            <div className="mt-8 grid gap-4 sm:grid-cols-2">
              {[
                ["Fund", "USDC-d in the mandate PDA — never the operator."],
                ["Pause", "Owner-only unpause."],
                ["Revoke", "Next intent refused. Receipt says Revoked."],
                ["Withdraw", "On in every state. Coins to you."],
              ].map(([name, body]) => (
                <div key={name}>
                  <p className="text-lg font-semibold">{name}</p>
                  <p className="mt-1 text-sm leading-relaxed text-muted">{body}</p>
                </div>
              ))}
            </div>
          </Reveal>
        </div>
      </section>

      <section className="relative h-48 overflow-hidden md:h-72">
        <Film src="/images/window.jpg" alt="Rain on the office window at night." className="absolute inset-0" />
        <div className="absolute inset-0 bg-bg/25" />
      </section>

      <section className="mx-auto max-w-6xl px-5 py-24 md:py-32" id="faq">
        <Reveal>
          <p className="eyebrow">FAQ</p>
          <h2 className="mt-3 max-w-xl text-4xl font-semibold tracking-tight md:text-5xl">No APY on this page.</h2>
        </Reveal>
        <div className="faq mt-10 overflow-hidden rounded-md bg-surface shadow-hairline">
          {FAQ.map((item) => (
            <details key={item.q} className="group border-b border-line last:border-b-0">
              <summary className="flex min-h-14 cursor-pointer list-none items-center justify-between gap-4 px-5 py-3 text-sm font-medium">
                {item.q}
                <span className="font-mono text-accent transition-transform duration-150 group-open:rotate-45">+</span>
              </summary>
              <p className="max-w-2xl px-5 pb-4 text-sm leading-relaxed text-muted">{item.a}</p>
            </details>
          ))}
        </div>
      </section>
    </>
  );
}
