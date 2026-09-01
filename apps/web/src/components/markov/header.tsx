import { Link, useRouterState } from "@tanstack/react-router";
import { Menu, X } from "lucide-react";
import { useEffect, useState } from "react";
import { cn } from "@/lib/utils";
import { Wordmark } from "./logo";
import { ThemeToggle } from "./theme-toggle";

const NAV = [
  { to: "/", label: "Home" },
  { to: "/book", label: "Desk" },
  { to: "/receipts", label: "Receipts" },
  { to: "/paper", label: "Paper" },
] as const;

export function Header() {
  const pathname = useRouterState({ select: (s) => s.location.pathname });
  const [open, setOpen] = useState(false);
  const [scrolled, setScrolled] = useState(false);

  useEffect(() => {
    const onScroll = () => setScrolled(window.scrollY > 12);
    onScroll();
    window.addEventListener("scroll", onScroll, { passive: true });
    return () => window.removeEventListener("scroll", onScroll);
  }, []);

  useEffect(() => {
    setOpen(false);
  }, [pathname]);

  return (
    <header className="sticky top-0 z-40 px-3 pt-3">
      <div
        className={cn(
          "mx-auto flex max-w-6xl items-center justify-between gap-3 rounded-lg px-3 py-1.5 shadow-hairline backdrop-blur-md transition-[background-color,box-shadow] duration-200",
          scrolled ? "bg-surface/90" : "bg-surface/70",
        )}
      >
        <Wordmark />

        <nav aria-label="Primary" className="hidden items-center md:flex">
          {NAV.map((item) => (
            <Link
              key={item.to}
              to={item.to}
              className={cn(
                "inline-flex min-h-10 items-center rounded-sm px-3 text-sm text-muted transition-colors hover:text-fg",
                pathname === item.to && "bg-raised text-fg",
              )}
            >
              {item.label}
            </Link>
          ))}
        </nav>

        <div className="flex items-center gap-0.5">
          <span className="mr-1 hidden items-center gap-2 sm:flex">
            <span className="live-dot size-1.5 rounded-full bg-allow" />
            <span className="font-mono text-micro text-muted">DEVNET</span>
          </span>
          <ThemeToggle />
          <button
            type="button"
            className="grid size-11 place-items-center rounded-sm text-fg hover:bg-raised md:hidden"
            aria-label={open ? "Close menu" : "Open menu"}
            aria-expanded={open}
            onClick={() => setOpen((v) => !v)}
          >
            {open ? <X className="size-4" strokeWidth={1.75} /> : <Menu className="size-4" strokeWidth={1.75} />}
          </button>
        </div>
      </div>

      {open ? (
        <div className="mx-auto mt-2 max-w-6xl rounded-lg bg-surface p-2 shadow-hairline md:hidden">
          <nav aria-label="Mobile" className="flex flex-col">
            {NAV.map((item) => (
              <Link
                key={item.to}
                to={item.to}
                className={cn(
                  "flex min-h-11 items-center rounded-sm px-3 text-sm text-muted",
                  pathname === item.to && "bg-raised text-fg",
                )}
              >
                {item.label}
              </Link>
            ))}
          </nav>
        </div>
      ) : null}
    </header>
  );
}
