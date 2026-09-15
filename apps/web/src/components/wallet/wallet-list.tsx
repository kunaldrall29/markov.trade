import { useConnect, type UiWallet } from "@wallet-standard/react";
import { useState } from "react";
import { accountOnCluster, useWallet } from "@/lib/wallet/provider";
import { errorText } from "@/lib/wallet/errors";
import { Notice } from "@/components/ui/field";

function WalletRow({ wallet }: { wallet: UiWallet }) {
  const [isConnecting, connect] = useConnect(wallet);
  const { select } = useWallet();
  const [error, setError] = useState<string | null>(null);

  async function onClick() {
    setError(null);
    try {
      const accounts = await connect();
      const account = accounts.find(accountOnCluster) ?? accounts[0];
      if (!account) {
        setError("The wallet returned no account for devnet. Switch the wallet's network to devnet and try again.");
        return;
      }
      select(wallet, account);
    } catch (e) {
      setError(errorText(e));
    }
  }

  return (
    <li>
      <button
        type="button"
        onClick={onClick}
        disabled={isConnecting}
        data-testid={`wallet-${wallet.name}`}
        className="flex min-h-12 w-full items-center gap-3 rounded-sm px-3 text-left text-sm hover:bg-raised disabled:opacity-60"
      >
        {wallet.icon ? <img src={wallet.icon} alt="" className="size-6 rounded-xs" /> : <span className="size-6 rounded-xs bg-raised" aria-hidden="true" />}
        <span className="flex-1 font-medium">{wallet.name}</span>
        <span className="font-mono text-nano uppercase tracking-eye text-subtle">{isConnecting ? "connecting" : wallet.accounts.length ? "authorised" : ""}</span>
      </button>
      {error ? (
        <div className="px-3 pb-2">
          <Notice tone="refuse">{error}</Notice>
        </div>
      ) : null}
    </li>
  );
}

export function WalletList() {
  const { wallets } = useWallet();
  if (wallets.length === 0) {
    return (
      <div className="grid gap-3 text-sm leading-relaxed text-muted">
        <p>No wallet is installed in this browser, or none of them speaks Solana devnet.</p>
        <ul className="list-disc space-y-1 pl-5">
          <li>Desktop: install Phantom, Solflare or Backpack, then reload.</li>
          <li>Android: open this page in Chrome with a Mobile Wallet Adapter wallet installed.</li>
          <li>iOS: open this page inside your wallet's in-app browser.</li>
        </ul>
        <p className="font-mono text-nano text-subtle">Nothing on this page is stored except the name of the wallet you pick.</p>
      </div>
    );
  }
  return (
    <ul className="grid gap-1" data-testid="wallet-list">
      {wallets.map((w) => (
        <WalletRow key={w.name} wallet={w} />
      ))}
    </ul>
  );
}
