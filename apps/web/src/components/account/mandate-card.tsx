/**
 * One of the owner's mandates: state, vault, the rules as the program holds
 * them, the day's counters, the venue position, the verbs and its receipts.
 */
import { useState } from "react";
import type { Address } from "@solana/kit";
import { USDC_D_DECIMALS, explorerAccount, formatPriceE6, formatUnits, policyLists, short, type MandateView } from "@markov/sdk";
import { serializeReceipt } from "@/lib/receipt-rows";
import { useMandateReceipts, useTokenBalance, useVenueView } from "@/lib/data/queries";
import { ReceiptList } from "@/components/desk/receipt-list";
import { VerbBar, VerbButton } from "@/components/desk/verbs";
import { Stat } from "@/components/ui/field";
import { Button } from "@/components/ui/button";
import { AmendRules } from "./amend-rules";

const STATE = ["Active", "Paused", "Revoked"] as const;
const ACTIONS = ["open", "increase", "reduce", "close", "flatten"];

export function MandateCard({ view, defaultOpen = false }: { view: MandateView; defaultOpen?: boolean }) {
  const m = view.data;
  const state = STATE[m.state] ?? "Active";
  const [open, setOpen] = useState(defaultOpen);
  const [amend, setAmend] = useState(false);
  const vault = useTokenBalance(view.address ? (m.vault as Address) : null, m.mint);
  const venue = useVenueView(open ? (view.address as Address) : null);
  const receipts = useMandateReceipts(open ? (view.address as Address) : null, 30);
  const lists = policyLists(m.policy);
  const now = Math.floor(Date.now() / 1000);
  const expired = now >= Number(m.policy.expiryTs);
  const decimals = USDC_D_DECIMALS;
  const target = { address: view.address, vault: m.vault, mint: m.mint, decimals, owner: m.owner, state };
  const pos = venue.data?.position;
  const gross = pos ? pos.notional : 0n;
  const net = pos ? (pos.side === 1 ? -gross : gross) : 0n;

  return (
    <article className="overflow-hidden rounded-md bg-surface shadow-hairline" data-testid="mandate-card" data-mandate={view.address}>
      <button type="button" className="flex w-full flex-wrap items-center justify-between gap-2 px-4 py-3 text-left" onClick={() => setOpen((v) => !v)} aria-expanded={open}>
        <span className="flex items-center gap-2 font-mono text-xs">
          <span className={`size-2 rounded-full ${state === "Active" && !expired ? "bg-allow live-dot" : state === "Revoked" || expired ? "bg-refuse" : "bg-subtle"}`} />
          <span className="text-fg">{short(view.address, 6, 6)}</span>
          <span className="text-subtle">nonce {m.nonce.toString()}</span>
        </span>
        <span className="font-mono text-micro uppercase tracking-eye text-muted" data-testid="mandate-state">
          {expired && state === "Active" ? "expired" : state.toLowerCase()} · vault {vault.data ? formatUnits(vault.data.amount, decimals) : "…"}
        </span>
      </button>

      {open ? (
        <div className="border-t border-line">
          <dl className="grid grid-cols-2 gap-px bg-line sm:grid-cols-4">
            <Stat label="vault" value={vault.data ? formatUnits(vault.data.amount, decimals) : "…"} note={vault.data?.exists === false ? "vault account missing" : "USDC-d · chain"} />
            <Stat label="day used" value={formatUnits(m.dayNotionalUsed, decimals)} note={`of ${formatUnits(m.policy.dailyCap, decimals, { min: 0 })} · chain`} />
            <Stat label="net delta" value={venue.data ? formatUnits(net, decimals) : "…"} note="venue position · off-chain guard" />
            <Stat label="gross" value={venue.data ? formatUnits(gross, decimals) : "…"} note={pos ? `entry ${formatPriceE6(pos.entryPrice)} · funding ${pos.fundingAccrued.toString()}` : "no position"} />
          </dl>

          <div className="grid gap-4 px-4 py-4 md:grid-cols-2">
            <section>
              <h3 className="font-mono text-micro uppercase tracking-eye text-subtle">Your rules · enforced on chain</h3>
              <dl className="mt-2 grid grid-cols-[max-content_1fr] gap-x-4 gap-y-1 font-mono text-xs tabular-nums">
                <dt className="text-subtle">per-trade cap</dt>
                <dd>{formatUnits(m.policy.perTxCap, decimals)} USDC-d</dd>
                <dt className="text-subtle">daily cap</dt>
                <dd>{formatUnits(m.policy.dailyCap, decimals)} USDC-d</dd>
                <dt className="text-subtle">spend</dt>
                <dd>
                  {formatUnits(m.policy.spendPerCall, decimals)} / call · {formatUnits(m.policy.spendDaily, decimals)} / day
                </dd>
                <dt className="text-subtle">max slippage</dt>
                <dd>{m.policy.maxSlippageBps} bps</dd>
                <dt className="text-subtle">mark age</dt>
                <dd>≤ {m.policy.maxMarkAgeSecs.toString()} s</dd>
                <dt className="text-subtle">expires</dt>
                <dd className={expired ? "text-refuse" : undefined}>{new Date(Number(m.policy.expiryTs) * 1000).toISOString().slice(0, 16).replace("T", " ")}Z</dd>
                <dt className="text-subtle">actions</dt>
                <dd>{ACTIONS.filter((_, i) => (m.policy.allowedActions & (1 << i)) !== 0).join(" ") || "none"}</dd>
                <dt className="text-subtle">venues</dt>
                <dd>{lists.venues.map((v) => short(v)).join(", ")}</dd>
                <dt className="text-subtle">tokens</dt>
                <dd>{lists.tokens.map((v) => short(v)).join(", ")}</dd>
              </dl>
              <div className="mt-3 flex flex-wrap gap-2">
                <Button type="button" size="sm" variant="ghost" onClick={() => setAmend((v) => !v)} disabled={state === "Revoked"} data-testid="verb-amend">
                  {amend ? "Cancel amend" : "Tighten rules"}
                </Button>
                <VerbButton verb="close" target={target} variant="ghost" />
              </div>
              {amend ? (
                <div className="mt-3">
                  <AmendRules mandate={view.address as Address} policy={m.policy} onDone={() => setAmend(false)} />
                </div>
              ) : null}
            </section>
            <section>
              <h3 className="font-mono text-micro uppercase tracking-eye text-subtle">Roles</h3>
              <dl className="mt-2 grid grid-cols-[max-content_1fr] gap-x-4 gap-y-1 font-mono text-xs">
                <dt className="text-subtle">owner</dt>
                <dd className="truncate">{short(m.owner, 8, 8)}</dd>
                <dt className="text-subtle">operator</dt>
                <dd className="truncate">{short(m.operator, 8, 8)} · proposes only</dd>
                <dt className="text-subtle">emergency</dt>
                <dd className="truncate">{m.emergency === "11111111111111111111111111111111" ? "none" : `${short(m.emergency, 8, 8)} · pause and revoke only`}</dd>
                <dt className="text-subtle">vault</dt>
                <dd className="truncate">
                  <a href={explorerAccount(m.vault)} target="_blank" rel="noreferrer" className="underline underline-offset-2">
                    {short(m.vault, 8, 8)}
                  </a>
                </dd>
                <dt className="text-subtle">mandate</dt>
                <dd className="truncate">
                  <a href={explorerAccount(view.address)} target="_blank" rel="noreferrer" className="underline underline-offset-2">
                    {short(view.address, 8, 8)}
                  </a>
                </dd>
                <dt className="text-subtle">receipts</dt>
                <dd>{m.actionSeq.toString()} agent decisions so far</dd>
              </dl>
            </section>
          </div>

          <div className="border-t border-line">
            <ReceiptList
              rows={(receipts.data?.receipts ?? []).map((r) => serializeReceipt(r, m.mint))}
              empty={receipts.isLoading ? "reading receipts…" : receipts.isError ? `receipts could not be read: ${receipts.error.message}` : "no receipts yet for this mandate"}
              reveal={false}
              maxHeight="max-h-72"
            />
          </div>

          <div className="border-t border-line p-3">
            <VerbBar target={target} />
            <p className="mt-2 px-1 font-mono text-nano text-subtle">Withdraw stays on in every state.</p>
          </div>
        </div>
      ) : null}
    </article>
  );
}
