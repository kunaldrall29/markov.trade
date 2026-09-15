/**
 * `amend_policy` is tighten-only on chain; this form refuses to build a
 * widening amendment and names the field, so the wallet never sees a
 * transaction the program would reject with PolicyNotTightened.
 */
import { useState } from "react";
import type { Address } from "@solana/kit";
import { USDC_D_DECIMALS, buildAmend, formatUnits, parseUnits, wideningFields, type Policy } from "@markov/sdk";
import { useVerbSender } from "@/components/desk/verbs";
import { Button } from "@/components/ui/button";
import { Field, Input, Notice } from "@/components/ui/field";

export function AmendRules({ mandate, policy, onDone }: { mandate: Address; policy: Policy; onDone: () => void }) {
  const d = USDC_D_DECIMALS;
  const sender = useVerbSender();
  const [perTx, setPerTx] = useState(formatUnits(policy.perTxCap, d, { min: 0, max: d }));
  const [daily, setDaily] = useState(formatUnits(policy.dailyCap, d, { min: 0, max: d }));
  const [slippage, setSlippage] = useState(String(policy.maxSlippageBps));
  const [markAge, setMarkAge] = useState(policy.maxMarkAgeSecs.toString());

  const perTxRaw = parseUnits(perTx, d);
  const dailyRaw = parseUnits(daily, d);
  const slippageBps = Number.parseInt(slippage, 10);
  const markAgeSecs = Number.parseInt(markAge, 10);
  const next = perTxRaw != null && dailyRaw != null && Number.isFinite(slippageBps) && Number.isFinite(markAgeSecs) && markAgeSecs > 0
    ? { ...policy, perTxCap: perTxRaw, dailyCap: dailyRaw, maxSlippageBps: slippageBps, maxMarkAgeSecs: BigInt(markAgeSecs) }
    : null;
  const widening = next ? wideningFields(policy, next) : [];
  const unchanged = next != null && next.perTxCap === policy.perTxCap && next.dailyCap === policy.dailyCap && next.maxSlippageBps === policy.maxSlippageBps && next.maxMarkAgeSecs === policy.maxMarkAgeSecs;

  async function submit() {
    if (!next) return;
    await sender.run(async () => buildAmend(sender.signer!.signer, mandate, next));
  }

  return (
    <div className="grid gap-3 rounded-sm bg-bg p-3 shadow-hairline" data-testid="amend-rules">
      <p className="text-xs leading-relaxed text-muted">Rules can only get tighter. Widening a cap needs a new mandate.</p>
      <div className="grid gap-3 sm:grid-cols-2">
        <Field label="per-trade cap" htmlFor="am-pertx">
          <Input id="am-pertx" inputMode="decimal" value={perTx} onChange={(e) => setPerTx(e.target.value)} data-testid="am-pertx" />
        </Field>
        <Field label="daily cap" htmlFor="am-daily">
          <Input id="am-daily" inputMode="decimal" value={daily} onChange={(e) => setDaily(e.target.value)} />
        </Field>
        <Field label="max slippage (bps)" htmlFor="am-slip">
          <Input id="am-slip" inputMode="numeric" value={slippage} onChange={(e) => setSlippage(e.target.value)} />
        </Field>
        <Field label="max mark age (s)" htmlFor="am-age">
          <Input id="am-age" inputMode="numeric" value={markAge} onChange={(e) => setMarkAge(e.target.value)} />
        </Field>
      </div>
      {widening.length ? <Notice tone="refuse">would widen: {widening.join(", ")} — the program refuses this (PolicyNotTightened)</Notice> : null}
      {sender.phase ? <Notice>{sender.phase === "signing" ? "waiting for your wallet…" : sender.phase === "confirming" ? "confirming on devnet…" : "building…"}</Notice> : null}
      {sender.error ? <Notice tone="refuse">{sender.error}</Notice> : null}
      {sender.outcome ? <Notice tone={sender.outcome.err ? "refuse" : "allow"}>{sender.outcome.err ? `landed with an error: ${sender.outcome.err}` : `amended · ${sender.outcome.signature.slice(0, 12)}…`}</Notice> : null}
      <div className="flex justify-end gap-2">
        <Button type="button" size="sm" variant="ghost" onClick={onDone}>
          {sender.outcome ? "Done" : "Cancel"}
        </Button>
        {!sender.outcome ? (
          <Button type="button" size="sm" onClick={submit} disabled={!next || widening.length > 0 || unchanged || sender.phase != null || !sender.signer} data-testid="am-submit">
            Sign the tighter rules
          </Button>
        ) : null}
      </div>
    </div>
  );
}
