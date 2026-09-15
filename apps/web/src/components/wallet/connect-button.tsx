import { Link } from "@tanstack/react-router";
import { Copy, ExternalLink, LogOut, Wallet } from "lucide-react";
import { useState } from "react";
import { explorerAccount, short } from "@markov/sdk";
import { useWallet } from "@/lib/wallet/provider";
import { Button } from "@/components/ui/button";
import { Dialog } from "@/components/ui/dialog";
import { WalletList } from "./wallet-list";

export function ConnectButton({ size = "sm" }: { size?: "sm" | "default" }) {
  const { selected, open, setOpen, ready, disconnect } = useWallet();
  const [menu, setMenu] = useState(false);

  if (!ready) {
    return (
      <Button type="button" size={size} variant="ghost" disabled aria-hidden="true" className="min-w-24">
        <Wallet className="size-4" strokeWidth={1.75} />
        Connect
      </Button>
    );
  }

  if (!selected) {
    return (
      <>
        <Button type="button" size={size} variant="primary" onClick={() => setOpen(true)} data-testid="connect-wallet" className="min-w-24">
          <Wallet className="size-4" strokeWidth={1.75} />
          Connect
        </Button>
        <Dialog
          open={open}
          onOpenChange={setOpen}
          title="Connect a wallet"
          description="Devnet only. The Terminal never sees a key; every action is a transaction your wallet signs."
        >
          <WalletList />
        </Dialog>
      </>
    );
  }

  const { account, wallet } = selected;
  return (
    <>
      <Button type="button" size={size} variant="ghost" onClick={() => setMenu(true)} data-testid="wallet-menu" className="font-mono">
        {wallet.icon ? <img src={wallet.icon} alt="" className="size-4 rounded-xs" /> : <Wallet className="size-4" strokeWidth={1.75} />}
        {short(account.address)}
      </Button>
      <Dialog open={menu} onOpenChange={setMenu} title={wallet.name} description={<span className="font-mono text-xs break-all">{account.address}</span>}>
        <div className="grid gap-2">
          <Button asChild variant="quiet" className="justify-start">
            <Link to="/account" onClick={() => setMenu(false)}>
              <Wallet className="size-4" strokeWidth={1.75} /> Your account
            </Link>
          </Button>
          <Button
            type="button"
            variant="quiet"
            className="justify-start"
            onClick={() => {
              void navigator.clipboard?.writeText(account.address);
            }}
          >
            <Copy className="size-4" strokeWidth={1.75} /> Copy address
          </Button>
          <Button asChild variant="quiet" className="justify-start">
            <a href={explorerAccount(account.address)} target="_blank" rel="noreferrer">
              <ExternalLink className="size-4" strokeWidth={1.75} /> View on explorer
            </a>
          </Button>
          <Button
            type="button"
            variant="ghost"
            className="justify-start"
            data-testid="disconnect-wallet"
            onClick={async () => {
              setMenu(false);
              await disconnect();
            }}
          >
            <LogOut className="size-4" strokeWidth={1.75} /> Disconnect
          </Button>
        </div>
      </Dialog>
    </>
  );
}
