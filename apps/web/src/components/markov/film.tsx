import { useEffect, useRef } from "react";
import { cn } from "@/lib/utils";

/**
 * Landing images ship a 1280-wide variant next to the original (generated
 * from the originals; see docs/SESSION_LOG.md 2026-09-15). Phones get the
 * small one, wide screens the original.
 */
const WIDTHS: Record<string, number> = {
  "/images/hero-desk.jpg": 1792,
  "/images/leaving.jpg": 1728,
  "/images/mandate.jpg": 1728,
  "/images/mark.jpg": 1408,
  "/images/pile.jpg": 1728,
  "/images/room.jpg": 1792,
  "/images/still-life.jpg": 1600,
  "/images/tape.jpg": 1728,
  "/images/window.jpg": 2128,
};

export function responsiveSrc(src: string): { src: string; srcSet?: string; sizes?: string } {
  const w = WIDTHS[src];
  if (!w) return { src };
  const small = src.replace(/\.jpg$/, "-1280.jpg");
  return { src: small, srcSet: `${small} 1280w, ${src} ${w}w`, sizes: "100vw" };
}

export function Film({
  src,
  alt,
  className,
  imgClassName,
  priority = false,
}: {
  src: string;
  alt: string;
  className?: string;
  imgClassName?: string;
  /** The page's largest paint: fetch it first, never lazily. */
  priority?: boolean;
}) {
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const el = ref.current;
    if (!el) return;
    const reduced = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    if (reduced) return;

    const onScroll = () => {
      const r = el.getBoundingClientRect();
      const p = (r.top + r.height / 2 - window.innerHeight / 2) / window.innerHeight;
      el.style.setProperty("--shift", `${(p * -28).toFixed(2)}px`);
    };
    onScroll();
    window.addEventListener("scroll", onScroll, { passive: true });
    return () => window.removeEventListener("scroll", onScroll);
  }, []);

  return (
    <div ref={ref} className={cn("overflow-hidden bg-raised", className)}>
      <img
        {...responsiveSrc(src)}
        alt={alt}
        loading={priority ? "eager" : "lazy"}
        fetchPriority={priority ? "high" : "auto"}
        decoding="async"
        className={cn(
          "h-full w-full object-cover will-change-transform ken origin-center",
          imgClassName,
        )}
        style={{ translate: "0 var(--shift, 0px)" }}
      />
    </div>
  );
}
