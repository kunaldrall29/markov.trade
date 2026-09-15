import { Link } from "@tanstack/react-router";
import { useEffect, useState } from "react";
import { Blotter } from "@/components/markov/blotter";
import { StageLine } from "@/components/markov/stage";
import { Button } from "@/components/ui/button";

const LINES = ["Set the rules.", "Software trades.", "You keep the key."];

export function Hero() {
  const [ready, setReady] = useState(false);

  useEffect(() => {
    const reduced = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    if (reduced) {
      setReady(true);
      return;
    }
    const t = window.setTimeout(() => setReady(true), 40);
    return () => window.clearTimeout(t);
  }, []);

  return (
    <section className="relative min-h-[88svh] overflow-hidden">
      <div className="absolute inset-0">
        <video className="h-full w-full object-cover" autoPlay muted loop playsInline poster="/images/hero-desk.jpg">
          <source src="/videos/hero-desk.mp4" type="video/mp4" />
        </video>
        <div className="hero-mask absolute inset-0" />
      </div>

      <div className="relative mx-auto grid max-w-6xl items-center gap-10 px-5 pb-16 pt-24 lg:grid-cols-[1.08fr_0.92fr] lg:pb-24 lg:pt-16">
        <div>
          <StageLine />

          <h1 className="mt-6 text-4xl font-semibold leading-display tracking-tight sm:text-6xl lg:text-7xl">
            {LINES.map((line, i) => (
              <span key={line} className={ready ? "word-in block" : "block opacity-0"} style={{ animationDelay: `${120 + i * 110}ms` }}>
                {line}
              </span>
            ))}
          </h1>

          <p className={ready ? "word-in mt-6 max-w-md text-base leading-relaxed text-muted" : "mt-6 max-w-md text-base leading-relaxed text-muted opacity-0"} style={{ animationDelay: "460ms" }}>
            Markov is a control plane for on-chain capital. Your limits live in a Solana account as hard rules; a bot, a model or a house desk may propose, and the program decides what the capital is allowed to do. Every allow and every refusal is a public receipt. The operator cannot withdraw.
          </p>

          <div className={ready ? "word-in mt-8 flex flex-wrap gap-3" : "mt-8 flex flex-wrap gap-3 opacity-0"} style={{ animationDelay: "560ms" }}>
            <Button asChild>
              <Link to="/account">Open your account</Link>
            </Button>
            <Button asChild variant="ghost">
              <Link to="/book">Watch the house book</Link>
            </Button>
          </div>
        </div>

        <div id="blotter">
          <Blotter />
        </div>
      </div>
    </section>
  );
}
