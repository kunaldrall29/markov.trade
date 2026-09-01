import { useEffect, useRef } from "react";
import { cn } from "@/lib/utils";

export function Film({
  src,
  alt,
  className,
  imgClassName,
}: {
  src: string;
  alt: string;
  className?: string;
  imgClassName?: string;
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
        src={src}
        alt={alt}
        className={cn(
          "h-full w-full object-cover will-change-transform ken origin-center",
          imgClassName,
        )}
        style={{ translate: "0 var(--shift, 0px)" }}
      />
    </div>
  );
}
