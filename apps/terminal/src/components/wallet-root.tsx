"use client";

import { ConnectionProvider, WalletProvider } from "@solana/wallet-adapter-react";
import { WalletModalProvider } from "@solana/wallet-adapter-react-ui";
import { PhantomWalletAdapter } from "@solana/wallet-adapter-phantom";
import { SolflareWalletAdapter } from "@solana/wallet-adapter-solflare";
import { CoinbaseWalletAdapter } from "@solana/wallet-adapter-coinbase";
import { TrustWalletAdapter } from "@solana/wallet-adapter-trust";
import { useMemo } from "react";
import { rpcUrl } from "@/lib/env";
import { SiwsGate } from "@/components/siws";

export function WalletRoot({ children }: { children: React.ReactNode }) {
  const wallets = useMemo(
    () => [
      new PhantomWalletAdapter(),
      new SolflareWalletAdapter(),
      new CoinbaseWalletAdapter(),
      new TrustWalletAdapter(),
    ],
    [],
  );
  return (
    <ConnectionProvider endpoint={rpcUrl}>
      <WalletProvider wallets={wallets} autoConnect>
        <WalletModalProvider>
          <SiwsGate>{children}</SiwsGate>
        </WalletModalProvider>
      </WalletProvider>
    </ConnectionProvider>
  );
}
