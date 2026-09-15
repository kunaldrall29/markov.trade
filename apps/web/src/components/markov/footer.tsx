import { Link } from "@tanstack/react-router";
import { MANDATE_PROGRAM_ID, explorerAccount, short } from "@markov/sdk";
import { Wordmark } from "./logo";

export function Footer() {
  return (
    <footer className="mt-4 border-t border-line py-14">
      <div className="mx-auto flex max-w-6xl flex-col gap-8 px-5 md:flex-row md:items-end md:justify-between">
        <div className="max-w-sm">
          <Wordmark className="text-sm" />
          <p className="mt-4 text-2xl font-semibold tracking-tight">Capital may only do what the owner allowed.</p>
          <p className="mt-2 text-sm leading-relaxed text-muted">
            Your rules live in an on-chain account. Every allow and every refusal is a receipt. Withdraw stays on in every state.
          </p>
          <p className="mt-5 font-mono text-nano uppercase tracking-eye text-subtle">Markov · test stage · Solana devnet · unaudited</p>
        </div>
        <nav aria-label="Footer" className="flex flex-wrap gap-5 text-sm text-muted">
          <Link to="/book" className="hover:text-fg">
            Desk
          </Link>
          <Link to="/receipts" className="hover:text-fg">
            Activity
          </Link>
          <Link to="/account" className="hover:text-fg">
            Account
          </Link>
          <Link to="/paper" className="hover:text-fg">
            Paper
          </Link>
          <a href={explorerAccount(MANDATE_PROGRAM_ID)} target="_blank" rel="noreferrer" className="font-mono text-xs hover:text-fg" title="Mandate program on devnet">
            program {short(MANDATE_PROGRAM_ID)}
          </a>
        </nav>
      </div>
    </footer>
  );
}
