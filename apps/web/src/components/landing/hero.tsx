import { Link } from "@tanstack/react-router";
import { useEffect, useState } from "react";
import { Blotter } from "@/components/markov/blotter";
import { Button } from "@/components/ui/button";

const LINES = ["Park USDC.", "The book works.", "You leave."];

export function Hero({
  onWatchRefusal,
  replayNonce,
}: {
  onWatchRefusal: () => void;
  replayNonce: number;
}) {
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
        <video
          className="h-full w-full object-cover"
          autoPlay
          muted
          loop
          playsInline
          poster="/images/hero-desk.jpg"
        >
          <source src="/videos/hero-desk.mp4" type="video/mp4" />
        </video>
        <div className="hero-mask absolute inset-0" />
      </div>

      <div className="relative mx-auto grid max-w-6xl items-center gap-10 px-5 pb-16 pt-24 lg:grid-cols-[1.08fr_0.92fr] lg:pb-24 lg:pt-16">
        <div>
          <p className="inline-flex items-center gap-2 rounded-full px-3 py-1 font-mono text-micro text-muted shadow-hairline">
            <span className="size-1.5 rounded-full bg-allow" />
            House book · Solana perps · unaudited
          </p>

          <h1 className="mt-6 text-4xl font-semibold leading-display tracking-tight sm:text-6xl lg:text-7xl">
            {LINES.map((line, i) => (
              <span
                key={line}
                className={ready ? "word-in block" : "block opacity-0"}
                style={{ animationDelay: `${120 + i * 110}ms` }}
              >
                {line}
              </span>
            ))}
          </h1>

          <p
            className={ready ? "word-in mt-6 max-w-md text-base leading-relaxed text-muted" : "mt-6 max-w-md text-base leading-relaxed text-muted opacity-0"}
            style={{ animationDelay: "460ms" }}
          >
            Near delta-neutral intent on Solana perps. Funding when shorts are paid. Inventory cut in trend. The operator cannot withdraw. Every fill and every refusal is a public receipt.
          </p>

          <div
            className={ready ? "word-in mt-8 flex flex-wrap gap-3" : "mt-8 flex flex-wrap gap-3 opacity-0"}
            style={{ animationDelay: "560ms" }}
          >
            <Button asChild>
              <Link to="/book">Open the desk</Link>
            </Button>
            <Button type="button" variant="ghost" onClick={onWatchRefusal}>
              Watch a refusal
            </Button>
          </div>
        </div>

        <div id="blotter">
          <Blotter replayNonce={replayNonce} holdOnRefusal={replayNonce > 0} />
        </div>
      </div>
    </section>
  );
}
