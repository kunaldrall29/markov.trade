/**
 * Owner verbs. Each opens a plain-words dialog that names what stays
 * possible afterwards, builds the instruction with the SDK, and hands it to
 * the wallet. `Withdraw` is never `disabled` (docs/13 §6 rule 1): when no
 * wallet is connected the dialog still opens and offers to connect.
 */
import { useQueryClient } from "@tanstack/react-query";
import { useState, type ReactNode } from "react";
import type { Address } from "@solana/kit";
import {
  buildClose,
  buildFund,
  buildPause,
  buildRevoke,
  buildUnpause,
  buildWithdraw,
  explorerTx,
  formatUnits,
  parseUnits,
  short,
  type MandateRef,
} from "@markov/sdk";
import { useOwnerUsdcBalance, useTokenBalance } from "@/lib/data/queries";
import { describeProgramError, errorText } from "@/lib/wallet/errors";
import { useWallet } from "@/lib/wallet/provider";
import { sendVerb, type SendPhase } from "@/lib/wallet/send";
import { useSigner } from "@/lib/wallet/signer";
import { Button } from "@/components/ui/button";
import { Dialog } from "@/components/ui/dialog";
import { Field, Input, Notice } from "@/components/ui/field";
import { WalletList } from "@/components/wallet/wallet-list";

export type VerbTarget = {
  address: string;
  vault: string;
  mint: string;
  decimals: number;
  owner: string;
  state: "Active" | "Paused" | "Revoked";
};

type Verb = "fund" | "withdraw" | "pause" | "unpause" | "revoke" | "close";

const CONSEQUENCE: Record<Verb, { title: string; body: string; button: string; variant: "primary" | "quiet" | "kill" | "ghost" }> = {
  fund: { title: "Fund the mandate", body: "USDC-d moves from your wallet into the mandate's vault, a token account only the mandate program can move. The operator cannot withdraw it. You can, in every state.", button: "Fund", variant: "primary" },
  withdraw: { title: "Withdraw", body: "Coins leave the vault for your own token account. This works while Active, Paused, Revoked or expired; the program has no state check on this path.", button: "Withdraw", variant: "primary" },
  pause: { title: "Pause the mandate", body: "The agent's next proposal is refused with Paused and a receipt says so. Only you can unpause. Withdraw stays on.", button: "Pause", variant: "quiet" },
  unpause: { title: "Unpause", body: "The agent may propose again, inside the same rules. Nothing widens.", button: "Unpause", variant: "quiet" },
  revoke: { title: "Revoke the mandate", body: "Terminal. The next proposal is refused with Revoked; funding stops; withdraw stays on. There is no un-revoke.", button: "Revoke", variant: "kill" },
  close: { title: "Close the mandate", body: "Reclaims the account rent to your wallet. Only possible once the vault is empty.", button: "Close", variant: "ghost" },
};

type Outcome = { signature: string; err: string | null };

function PhaseNote({ phase }: { phase: SendPhase | null }) {
  if (!phase) return null;
  const text = phase === "building" ? "building the transaction…" : phase === "signing" ? "waiting for your wallet…" : "confirming on devnet…";
  return <Notice>{text}</Notice>;
}

function OutcomeNote({ outcome }: { outcome: Outcome }) {
  return (
    <Notice tone={outcome.err ? "refuse" : "allow"}>
      {outcome.err ? `landed with an error: ${outcome.err}` : "confirmed"} ·{" "}
      <a href={explorerTx(outcome.signature)} target="_blank" rel="noreferrer" className="underline underline-offset-2" data-testid="outcome-explorer">
        {short(outcome.signature, 8, 8)} ↗
      </a>
    </Notice>
  );
}

export function useVerbSender() {
  const signer = useSigner();
  const qc = useQueryClient();
  const [phase, setPhase] = useState<SendPhase | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [outcome, setOutcome] = useState<Outcome | null>(null);

  async function run(build: (signerAddress: Address) => Promise<import("@solana/kit").Instruction[]>) {
    if (!signer) {
      setError("connect a wallet that can sign on devnet first");
      return;
    }
    setError(null);
    setOutcome(null);
    try {
      const ixs = await build(signer.address as Address);
      const res = await sendVerb(signer, ixs, { onPhase: setPhase });
      setOutcome({ signature: res.signature, err: res.err });
    } catch (e) {
      const pe = describeProgramError(e);
      setError(pe ? `${pe.message} (${pe.code}) — ${errorText(e)}` : errorText(e));
    } finally {
      setPhase(null);
      void qc.invalidateQueries();
    }
  }

  return { signer, phase, error, outcome, run, reset: () => (setError(null), setOutcome(null)) };
}

function VerbDialog({ verb, target, open, onOpenChange, children }: { verb: Verb; target: VerbTarget; open: boolean; onOpenChange: (o: boolean) => void; children: ReactNode }) {
  const c = CONSEQUENCE[verb];
  return (
    <Dialog open={open} onOpenChange={onOpenChange} title={c.title} description={c.body}>
      <p className="mb-3 font-mono text-nano text-subtle">
        mandate {short(target.address, 6, 6)} · vault {short(target.vault, 6, 6)} · state {target.state}
      </p>
      {children}
    </Dialog>
  );
}

function ConnectInline() {
  return (
    <div className="grid gap-2">
      <Notice>No wallet connected. Pick one to sign; nothing is sent until you approve it there.</Notice>
      <WalletList />
    </div>
  );
}

function NotOwner({ target, address }: { target: VerbTarget; address: string }) {
  return (
    <Notice tone="refuse">
      The connected wallet {short(address)} does not own this mandate (owner {short(target.owner)}). The program would refuse with NotOwner; connect the owner's wallet.
    </Notice>
  );
}

export function VerbButton({ verb, target, label, variant, size = "sm", className, testId }: { verb: Verb; target: VerbTarget; label?: string; variant?: "primary" | "quiet" | "kill" | "ghost"; size?: "sm" | "default"; className?: string; testId?: string }) {
  const [open, setOpen] = useState(false);
  const c = CONSEQUENCE[verb];
  return (
    <>
      <Button type="button" size={size} variant={variant ?? c.variant} className={className} onClick={() => setOpen(true)} data-testid={testId ?? `verb-${verb}`}>
        {label ?? c.button}
      </Button>
      {open ? <VerbFlow verb={verb} target={target} open={open} onOpenChange={setOpen} /> : null}
    </>
  );
}

function VerbFlow({ verb, target, open, onOpenChange }: { verb: Verb; target: VerbTarget; open: boolean; onOpenChange: (o: boolean) => void }) {
  const { selected } = useWallet();
  const sender = useVerbSender();
  const [amountText, setAmountText] = useState("");
  const owner = (selected?.account.address ?? null) as Address | null;
  const usdc = useOwnerUsdcBalance(verb === "fund" ? owner : null);
  const vault = useTokenBalance(verb === "withdraw" || verb === "close" ? (target.vault as Address) : null, target.mint as Address);
  const ref: MandateRef = { mandate: target.address as Address, vault: target.vault as Address, mint: target.mint as Address };
  const amount = parseUnits(amountText, target.decimals);
  const isOwner = !!owner && owner === target.owner;

  const needsAmount = verb === "fund" || verb === "withdraw";
  const max = verb === "fund" ? usdc.data?.amount ?? null : vault.data?.amount ?? null;

  async function submit() {
    await sender.run(async (signerAddress) => {
      const signer = sender.signer!.signer;
      switch (verb) {
        case "fund":
          return buildFund(signer, ref, amount!);
        case "withdraw":
          return buildWithdraw(signer, ref, amount!);
        case "pause":
          return buildPause(signer, ref.mandate);
        case "unpause":
          return buildUnpause(signer, ref.mandate);
        case "revoke":
          return buildRevoke(signer, ref.mandate);
        case "close":
          return buildClose(signer, ref);
        default:
          throw new Error(`unknown verb ${String(verb)} for ${signerAddress}`);
      }
    });
  }

  return (
    <VerbDialog verb={verb} target={target} open={open} onOpenChange={onOpenChange}>
      {!selected ? (
        <ConnectInline />
      ) : (
        <div className="grid gap-3">
          {!isOwner ? <NotOwner target={target} address={selected.account.address} /> : null}
          {needsAmount ? (
            <Field
              label={`amount (USDC-d, ${target.decimals} decimals)`}
              htmlFor="verb-amount"
              hint={
                max == null ? (verb === "fund" ? "reading your USDC-d balance…" : "reading the vault…") : (
                  <button type="button" className="underline underline-offset-2" onClick={() => setAmountText(formatUnits(max, target.decimals, { min: 0, max: target.decimals }))} data-testid="amount-max">
                    max {formatUnits(max, target.decimals)}
                  </button>
                )
              }
            >
              <Input id="verb-amount" inputMode="decimal" placeholder="0.00" value={amountText} onChange={(e) => setAmountText(e.target.value)} data-testid="verb-amount" autoFocus />
            </Field>
          ) : null}
          <PhaseNote phase={sender.phase} />
          {sender.error ? <Notice tone="refuse">{sender.error}</Notice> : null}
          {sender.outcome ? <OutcomeNote outcome={sender.outcome} /> : null}
          <div className="flex flex-wrap justify-end gap-2">
            <Button type="button" variant="ghost" onClick={() => onOpenChange(false)}>
              {sender.outcome ? "Done" : "Cancel"}
            </Button>
            {!sender.outcome ? (
              <Button
                type="button"
                variant={CONSEQUENCE[verb].variant}
                onClick={submit}
                disabled={sender.phase != null || (needsAmount && (amount == null || amount <= 0n))}
                data-testid="verb-submit"
              >
                {CONSEQUENCE[verb].button}
              </Button>
            ) : null}
          </div>
        </div>
      )}
    </VerbDialog>
  );
}

/** Fund / Pause or Unpause / Revoke / Withdraw, in that order, on one row. */
export function VerbBar({ target }: { target: VerbTarget }) {
  const revoked = target.state === "Revoked";
  return (
    <div className="grid grid-cols-2 gap-2 sm:grid-cols-4" data-testid="verb-bar">
      <VerbButton verb="fund" target={target} variant="ghost" className={revoked ? "opacity-60" : undefined} />
      {target.state === "Paused" ? <VerbButton verb="unpause" target={target} variant="quiet" /> : <VerbButton verb="pause" target={target} variant="quiet" className={revoked ? "opacity-60" : undefined} />}
      <VerbButton verb="revoke" target={target} variant="kill" className={revoked ? "opacity-60" : undefined} />
      <VerbButton verb="withdraw" target={target} variant="primary" />
    </div>
  );
}
