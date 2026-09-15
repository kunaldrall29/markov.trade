/**
 * Build, sign, send and confirm one transaction of owner-verb instructions.
 *
 * Confirmation is a poll on `getSignatureStatuses` bounded by the blockhash's
 * `lastValidBlockHeight`, because keyless devnet WebSockets are capped at ten
 * sockets (FACTS `RPC endpoints`) and the Terminal must not hold one per tab.
 * A landed transaction with a runtime error is reported as exactly that.
 */
import {
  appendTransactionMessageInstructions,
  createTransactionMessage,
  getBase64EncodedWireTransaction,
  pipe,
  setTransactionMessageFeePayerSigner,
  setTransactionMessageLifetimeUsingBlockhash,
  signAndSendTransactionMessageWithSigners,
  signTransactionMessageWithSigners,
  getBase58Decoder,
  type Instruction,
  type Signature,
} from "@solana/kit";
import { getSetComputeUnitLimitInstruction } from "@solana-program/compute-budget";
import { readEither, rpc } from "@/lib/rpc";
import type { WalletSigner } from "./signer";

export type SendResult = { signature: Signature; slot: bigint; err: string | null };

export type SendPhase = "building" | "signing" | "confirming";

export async function sendVerb(
  wallet: WalletSigner,
  instructions: Instruction[],
  opts: { computeUnits?: number; onPhase?: (p: SendPhase) => void } = {},
): Promise<SendResult> {
  opts.onPhase?.("building");
  const { value: blockhash } = await readEither((r) => r.getLatestBlockhash({ commitment: "confirmed" }).send());
  const message = pipe(
    createTransactionMessage({ version: 0 }),
    (m) => setTransactionMessageFeePayerSigner(wallet.signer, m),
    (m) => setTransactionMessageLifetimeUsingBlockhash(blockhash, m),
    (m) =>
      appendTransactionMessageInstructions(
        [getSetComputeUnitLimitInstruction({ units: opts.computeUnits ?? 120_000 }), ...instructions],
        m,
      ),
  );

  opts.onPhase?.("signing");
  let signature: Signature;
  if (wallet.kind === "sending") {
    const bytes = await signAndSendTransactionMessageWithSigners(message);
    signature = getBase58Decoder().decode(bytes) as Signature;
  } else {
    const signed = await signTransactionMessageWithSigners(message);
    const wire = getBase64EncodedWireTransaction(signed);
    signature = await rpc().sendTransaction(wire, { encoding: "base64", preflightCommitment: "confirmed" }).send();
  }

  opts.onPhase?.("confirming");
  return confirmSignature(signature, blockhash.lastValidBlockHeight);
}

export async function confirmSignature(signature: Signature, lastValidBlockHeight: bigint): Promise<SendResult> {
  const started = Date.now();
  for (;;) {
    const { value } = await readEither((r) => r.getSignatureStatuses([signature]).send());
    const status = value[0];
    if (status && (status.confirmationStatus === "confirmed" || status.confirmationStatus === "finalized")) {
      return { signature, slot: status.slot, err: status.err == null ? null : JSON.stringify(status.err) };
    }
    const height = await readEither((r) => r.getBlockHeight({ commitment: "confirmed" }).send());
    if (height > lastValidBlockHeight) {
      throw new Error(`transaction ${signature} expired before it was confirmed (block height ${height} > ${lastValidBlockHeight})`);
    }
    if (Date.now() - started > 90_000) {
      throw new Error(`transaction ${signature} not confirmed after 90 s; check the explorer before retrying`);
    }
    await new Promise((res) => setTimeout(res, 1500));
  }
}
