import { createFileRoute } from "@tanstack/react-router";
import { useMemo, useState } from "react";
import type { Address } from "@solana/kit";
import { USDC_D_DECIMALS, USDC_D_MINT, explorerAccount, formatUnits, short } from "@markov/sdk";
import { useOwnerMandates, useOwnerUsdcBalance } from "@/lib/data/queries";
import { useWallet } from "@/lib/wallet/provider";
import { CreateMandate } from "@/components/account/create-mandate";
import { MandateCard } from "@/components/account/mandate-card";
import { DegradedBanner } from "@/components/desk/degraded";
import { SiteShell } from "@/components/markov/site-shell";
import { StageLine } from "@/components/markov/stage";
import { Button } from "@/components/ui/button";
import { Notice } from "@/components/ui/field";
import { WalletList } from "@/components/wallet/wallet-list";

export const Route = createFileRoute("/account")({ component: AccountPage });

function AccountPage() {
  const { selected, ready } = useWallet();
  const owner = (selected?.account.address ?? null) as Address | null;
  const mandates = useOwnerMandates(owner);
  const usdc = useOwnerUsdcBalance(owner);
  const [creating, setCreating] = useState(false);
  const nextNonce = useMemo(() => {
    const max = (mandates.data ?? []).reduce((m, v) => (v.data.nonce > m ? v.data.nonce : m), 0n);
    return max + 1n;
  }, [mandates.data]);

  return (
    <SiteShell>
      <section className="mx-auto max-w-6xl px-5 pb-8 pt-10">
        <StageLine />
        <h1 className="mt-4 text-4xl font-semibold tracking-tight md:text-6xl">Your account.</h1>
        <p className="mt-4 max-w-lg text-sm leading-relaxed text-muted">
          Your rules live in a mandate account the program enforces. Fund it, tighten it, pause it, revoke it, and take the coins back at any time. Nothing here is stored off chain.
        </p>
      </section>

      <section className="mx-auto max-w-6xl px-5 pb-20">
        {!ready || !selected ? (
          <div className="grid gap-6 md:grid-cols-[0.9fr_1.1fr]">
            <div className="rounded-md bg-surface p-5 shadow-hairline">
              <p className="font-mono text-micro uppercase tracking-eye text-subtle">Connect</p>
              <h2 className="mt-2 text-2xl font-semibold tracking-tight">Pick a wallet.</h2>
              <p className="mt-2 text-sm leading-relaxed text-muted">Devnet only in this stage. The Terminal never holds a key; your wallet signs every action after showing it to you.</p>
              <div className="mt-4 min-h-40">
                <WalletList />
              </div>
            </div>
            <div className="rounded-md bg-surface p-5 shadow-hairline">
              <p className="font-mono text-micro uppercase tracking-eye text-subtle">What you get</p>
              <ul className="mt-3 space-y-2 text-sm leading-relaxed text-muted">
                <li>A mandate with a per-trade cap, a daily cap, a slippage bound, a mark-freshness bound and an expiry, enforced by the program.</li>
                <li>A vault only the program can move; the operator proposes, the ladder decides, and every answer is a receipt.</li>
                <li>Withdraw in every state. Pause and revoke whenever you like. Rules can only get tighter.</li>
              </ul>
            </div>
          </div>
        ) : (
          <div className="grid gap-4">
            <div className="flex flex-wrap items-center justify-between gap-3 rounded-md bg-surface px-4 py-3 shadow-hairline">
              <div className="font-mono text-xs">
                <span className="text-subtle">owner </span>
                <a href={explorerAccount(selected.account.address)} target="_blank" rel="noreferrer" className="underline underline-offset-2">
                  {short(selected.account.address, 8, 8)}
                </a>
                <span className="ml-3 text-subtle">USDC-d </span>
                <span data-testid="owner-usdc">{usdc.data ? (usdc.data.exists ? formatUnits(usdc.data.amount, USDC_D_DECIMALS) : "no token account") : usdc.isError ? "unreadable" : "…"}</span>
                <span className="ml-3 text-subtle">mint {short(USDC_D_MINT)}</span>
              </div>
              <Button type="button" size="sm" onClick={() => setCreating((v) => !v)} data-testid="new-mandate">
                {creating ? "Cancel" : "New mandate"}
              </Button>
            </div>

            {creating ? (
              <div className="rounded-md bg-surface p-5 shadow-hairline">
                <p className="font-mono text-micro uppercase tracking-eye text-subtle">Your rules</p>
                <h2 className="mt-2 text-2xl font-semibold tracking-tight">Set the bounds once.</h2>
                <p className="mt-2 text-sm leading-relaxed text-muted">The program validates every field again on chain. After this you fund the vault; the house agent may then propose actions inside these numbers.</p>
                <div className="mt-4">
                  <CreateMandate nextNonce={nextNonce} onCreated={() => void mandates.refetch()} />
                </div>
              </div>
            ) : null}

            {mandates.isError ? <DegradedBanner failing={[`mandates: ${mandates.error.message}`]} lastUpdated={mandates.dataUpdatedAt || null} /> : null}

            {mandates.isLoading ? (
              <Notice>reading your mandates from the chain…</Notice>
            ) : mandates.data && mandates.data.length === 0 ? (
              <div className="rounded-md bg-surface p-5 shadow-hairline" data-testid="no-mandates">
                <p className="font-mono text-sm text-subtle">no mandates for this wallet on devnet</p>
                <p className="mt-2 text-sm leading-relaxed text-muted">Create one above. You will need devnet SOL for rent and USDC-d to fund it; the mint authority for USDC-d is the deployer key recorded in FACTS.</p>
              </div>
            ) : (
              <div className="grid gap-3" data-testid="mandate-list">
                {(mandates.data ?? []).map((v, i) => (
                  <MandateCard key={v.address} view={v} defaultOpen={i === 0} />
                ))}
              </div>
            )}
          </div>
        )}
      </section>
    </SiteShell>
  );
}
