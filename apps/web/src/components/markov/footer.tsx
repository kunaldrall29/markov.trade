import { Link } from "@tanstack/react-router";
import { Wordmark } from "./logo";

export function Footer() {
  return (
    <footer className="mt-4 border-t border-line py-14">
      <div className="mx-auto flex max-w-6xl flex-col gap-8 px-5 md:flex-row md:items-end md:justify-between">
        <div className="max-w-sm">
          <Wordmark className="text-sm" />
          <p className="mt-4 text-2xl font-semibold tracking-tight">Keep the pile.</p>
          <p className="mt-2 text-sm leading-relaxed text-muted">
            Yield is the product. Authority is the lock. Receipts are the proof. AI is a desk, not a brand.
          </p>
          <p className="mt-5 font-mono text-nano uppercase tracking-eye text-subtle">Markov Book</p>
        </div>
        <nav aria-label="Footer" className="flex flex-wrap gap-5 text-sm text-muted">
          <Link to="/book" className="hover:text-fg">
            Desk
          </Link>
          <Link to="/receipts" className="hover:text-fg">
            Receipts
          </Link>
          <Link to="/paper" className="hover:text-fg">
            Paper
          </Link>
        </nav>
      </div>
    </footer>
  );
}
