/**
 * The connected account as a `@solana/kit` signer. Wallets that implement
 * `solana:signAndSendTransaction` sign and submit themselves (they may add
 * their own guards and priority fees); wallets that only sign hand the bytes
 * back and the Terminal submits them to the configured RPC.
 */
import { createContext, useContext, type ReactNode } from "react";
import type { TransactionModifyingSigner, TransactionSendingSigner } from "@solana/kit";
import { useWalletAccountTransactionSendingSigner, useWalletAccountTransactionSigner } from "@solana/react";
import type { UiWalletAccount } from "@wallet-standard/react";
import { solanaChain } from "@/lib/config";
import { useWallet } from "./provider";

export type WalletSigner =
  | { kind: "sending"; signer: TransactionSendingSigner; address: string }
  | { kind: "signing"; signer: TransactionModifyingSigner; address: string };

const SignerContext = createContext<WalletSigner | null>(null);

function SendingBridge({ account, children }: { account: UiWalletAccount; children: ReactNode }) {
  const signer = useWalletAccountTransactionSendingSigner(account, solanaChain);
  return <SignerContext.Provider value={{ kind: "sending", signer, address: account.address }}>{children}</SignerContext.Provider>;
}

function SigningBridge({ account, children }: { account: UiWalletAccount; children: ReactNode }) {
  const signer = useWalletAccountTransactionSigner(account, solanaChain);
  return <SignerContext.Provider value={{ kind: "signing", signer, address: account.address }}>{children}</SignerContext.Provider>;
}

export function SignerProvider({ children }: { children: ReactNode }) {
  const { selected } = useWallet();
  if (!selected) return <SignerContext.Provider value={null}>{children}</SignerContext.Provider>;
  const { account } = selected;
  if (account.features.includes("solana:signAndSendTransaction")) {
    return <SendingBridge account={account}>{children}</SendingBridge>;
  }
  if (account.features.includes("solana:signTransaction")) {
    return <SigningBridge account={account}>{children}</SigningBridge>;
  }
  return <SignerContext.Provider value={null}>{children}</SignerContext.Provider>;
}

export function useSigner(): WalletSigner | null {
  return useContext(SignerContext);
}
