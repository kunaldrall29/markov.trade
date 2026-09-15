/**
 * "Your Rules": create a mandate for the connected wallet. Defaults are the
 * Gate B template (FACTS `GATE_B_POLICY`) against the mock venue; every cap
 * is editable before signing and the program validates the result again.
 */
import { useState } from "react";
import type { Address } from "@solana/kit";
import {
  ACTION_BITS,
  BOOK_ONE_EMERGENCY,
  BOOK_ONE_OPERATOR,
  BOOK_ONE_STRATEGY_ID,
  DEMO_PERPS_PROGRAM_ID,
  GATE_B_POLICY,
  USDC_D_DECIMALS,
  USDC_D_MINT,
  buildCreateMandate,
  explorerAccount,
  formatUnits,
  parseUnits,
  policyFromSpec,
  short,
} from "@markov/sdk";
import { Button } from "@/components/ui/button";
import { Field, Input, Notice } from "@/components/ui/field";
import { useVerbSender } from "@/components/desk/verbs";

export function CreateMandate({ nextNonce, onCreated }: { nextNonce: bigint; onCreated?: (mandate: string) => void }) {
  const sender = useVerbSender();
  const [perTx, setPerTx] = useState(formatUnits(GATE_B_POLICY.perTxCap, USDC_D_DECIMALS, { min: 0 }));
  const [daily, setDaily] = useState(formatUnits(GATE_B_POLICY.dailyCap, USDC_D_DECIMALS, { min: 0 }));
  const [slippage, setSlippage] = useState(String(GATE_B_POLICY.maxSlippageBps));
  const [markAge, setMarkAge] = useState(String(GATE_B_POLICY.maxMarkAgeSecs));
  const [days, setDays] = useState("30");
  const [operator, setOperator] = useState<string>(BOOK_ONE_OPERATOR);
  const [emergency, setEmergency] = useState<string>(BOOK_ONE_EMERGENCY);
  const [created, setCreated] = useState<string | null>(null);

  const perTxRaw = parseUnits(perTx, USDC_D_DECIMALS);
  const dailyRaw = parseUnits(daily, USDC_D_DECIMALS);
  const slippageBps = Number.parseInt(slippage, 10);
  const markAgeSecs = Number.parseInt(markAge, 10);
  const expiryDays = Number.parseInt(days, 10);
  const valid =
    perTxRaw != null && perTxRaw > 0n && dailyRaw != null && dailyRaw >= perTxRaw && Number.isFinite(slippageBps) && slippageBps >= 0 && slippageBps <= 10_000 &&
    Number.isFinite(markAgeSecs) && markAgeSecs > 0 && Number.isFinite(expiryDays) && expiryDays > 0 && /^[1-9A-HJ-NP-Za-km-z]{32,44}$/.test(operator) && /^[1-9A-HJ-NP-Za-km-z]{32,44}$/.test(emergency);

  async function submit() {
    await sender.run(async () => {
      const signer = sender.signer!.signer;
      const expiryTs = BigInt(Math.floor(Date.now() / 1000) + expiryDays * 86_400);
      const policy = policyFromSpec({
        venues: [DEMO_PERPS_PROGRAM_ID],
        tokens: [USDC_D_MINT],
        allowedActions: ACTION_BITS.all,
        perTxCap: perTxRaw!,
        dailyCap: dailyRaw!,
        spendPerCall: GATE_B_POLICY.spendPerCall,
        spendDaily: GATE_B_POLICY.spendDaily,
        maxSlippageBps: slippageBps,
        maxMarkAgeSecs: BigInt(markAgeSecs),
        expiryTs,
      });
      const { instructions, mandate } = await buildCreateMandate(signer, {
        operator: operator as Address,
        emergency: emergency as Address,
        strategyId: BOOK_ONE_STRATEGY_ID,
        nonce: nextNonce,
        mint: USDC_D_MINT,
        policy,
      });
      setCreated(mandate);
      onCreated?.(mandate);
      return instructions;
    });
  }

  return (
    <div className="grid gap-4" data-testid="create-mandate">
      <div className="grid gap-3 sm:grid-cols-2">
        <Field label="per-trade cap (USDC-d)" htmlFor="cm-pertx" hint="the program refuses any single action above this (OverTxCap)">
          <Input id="cm-pertx" inputMode="decimal" value={perTx} onChange={(e) => setPerTx(e.target.value)} data-testid="cm-pertx" />
        </Field>
        <Field label="daily cap (USDC-d)" htmlFor="cm-daily" hint="rolling UTC day (OverDailyCap)">
          <Input id="cm-daily" inputMode="decimal" value={daily} onChange={(e) => setDaily(e.target.value)} data-testid="cm-daily" />
        </Field>
        <Field label="max slippage (bps)" htmlFor="cm-slip" hint="limit price must sit within this of the mark (SlippageExceeded)">
          <Input id="cm-slip" inputMode="numeric" value={slippage} onChange={(e) => setSlippage(e.target.value)} />
        </Field>
        <Field label="max mark age (seconds)" htmlFor="cm-age" hint="older marks refuse (StaleOracle)">
          <Input id="cm-age" inputMode="numeric" value={markAge} onChange={(e) => setMarkAge(e.target.value)} />
        </Field>
        <Field label="expires in (days)" htmlFor="cm-days" hint="after this every proposal refuses with Expired; withdraw stays on">
          <Input id="cm-days" inputMode="numeric" value={days} onChange={(e) => setDays(e.target.value)} />
        </Field>
        <Field label="strategy · venue · token" hint="fixed in this build">
          <p className="min-h-11 rounded-sm bg-raised px-3 py-2 font-mono text-xs leading-relaxed text-muted">
            {BOOK_ONE_STRATEGY_ID} · venue {short(DEMO_PERPS_PROGRAM_ID)} (mock, zero custody) · USDC-d {short(USDC_D_MINT)}
          </p>
        </Field>
        <Field label="operator (proposes only)" htmlFor="cm-op" hint="the house agent's key; it can never withdraw">
          <Input id="cm-op" value={operator} onChange={(e) => setOperator(e.target.value.trim())} className="text-xs" />
        </Field>
        <Field label="emergency key (pause and revoke only)" htmlFor="cm-em" hint="cannot unpause, cannot withdraw">
          <Input id="cm-em" value={emergency} onChange={(e) => setEmergency(e.target.value.trim())} className="text-xs" />
        </Field>
      </div>
      <p className="font-mono text-nano text-subtle">
        spend budgets {formatUnits(GATE_B_POLICY.spendPerCall, USDC_D_DECIMALS, { min: 0 })} per call / {formatUnits(GATE_B_POLICY.spendDaily, USDC_D_DECIMALS, { min: 0 })} per day · nonce {nextNonce.toString()} · mark bound to the Pyth SOL/USD devnet account
      </p>
      {sender.phase ? <Notice>{sender.phase === "signing" ? "waiting for your wallet…" : sender.phase === "confirming" ? "confirming on devnet…" : "building…"}</Notice> : null}
      {sender.error ? <Notice tone="refuse">{sender.error}</Notice> : null}
      {sender.outcome ? (
        <Notice tone={sender.outcome.err ? "refuse" : "allow"}>
          {sender.outcome.err ? `landed with an error: ${sender.outcome.err}` : "mandate created"} ·{" "}
          {created ? (
            <a href={explorerAccount(created)} target="_blank" rel="noreferrer" className="underline underline-offset-2">
              {short(created, 8, 8)} ↗
            </a>
          ) : null}
        </Notice>
      ) : null}
      <div className="flex justify-end">
        <Button type="button" onClick={submit} disabled={!valid || sender.phase != null || !sender.signer} data-testid="cm-submit">
          Sign the rules
        </Button>
      </div>
    </div>
  );
}
