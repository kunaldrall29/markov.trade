/**
 * Wallet Standard connection state for the Terminal.
 *
 * Every injected wallet (Phantom, Solflare, Backpack, …) registers itself
 * through the Wallet Standard; Solana Mobile's Mobile Wallet Adapter is
 * registered here so Android and iOS wallets appear in the same list. The
 * browser stores only the *name* of the chosen wallet and the *address* of the
 * chosen account, never key material (conventions §5).
 */
import { createContext, useCallback, useContext, useEffect, useMemo, useState, type ReactNode } from "react";
import { getWalletFeature, useWallets } from "@wallet-standard/react";
import type { UiWallet, UiWalletAccount } from "@wallet-standard/react";
import { solanaChain } from "@/lib/config";

const STORAGE_KEY = "markov-wallet";

type Persisted = { wallet: string; address: string };

function readPersisted(): Persisted | null {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return null;
    const v = JSON.parse(raw) as Persisted;
    return typeof v.wallet === "string" && typeof v.address === "string" ? v : null;
  } catch {
    return null;
  }
}

function writePersisted(v: Persisted | null) {
  try {
    if (v) localStorage.setItem(STORAGE_KEY, JSON.stringify(v));
    else localStorage.removeItem(STORAGE_KEY);
  } catch {
    /* storage may be unavailable; the session still works */
  }
}

export type Selected = { wallet: UiWallet; account: UiWalletAccount };

type WalletContextValue = {
  /** Wallets that can act on this cluster. */
  wallets: readonly UiWallet[];
  selected: Selected | null;
  select: (wallet: UiWallet, account: UiWalletAccount) => void;
  disconnect: () => Promise<void>;
  /** The connect dialog. */
  open: boolean;
  setOpen: (open: boolean) => void;
  /** True once the client has hydrated; before that nothing is known about wallets. */
  ready: boolean;
};

const WalletContext = createContext<WalletContextValue | null>(null);

export function supportsCluster(wallet: UiWallet): boolean {
  return wallet.chains.includes(solanaChain) && wallet.features.includes("standard:connect");
}

export function accountOnCluster(account: UiWalletAccount): boolean {
  return account.chains.includes(solanaChain);
}

export function WalletProvider({ children }: { children: ReactNode }) {
  const all = useWallets();
  const wallets = useMemo(() => all.filter(supportsCluster), [all]);
  const [persisted, setPersisted] = useState<Persisted | null>(null);
  const [open, setOpen] = useState(false);
  const [ready, setReady] = useState(false);

  // Register Mobile Wallet Adapter on the client: eagerly on phones, where it
  // is the main connector and a persisted session should resume; otherwise
  // only once the connect dialog opens, so desktop pages do not load it.
  useEffect(() => {
    const phone = /android|iphone|ipad|ipod/i.test(navigator.userAgent);
    if (phone) void import("./mwa").then((m) => m.registerMobileWalletAdapter());
    setPersisted(readPersisted());
    setReady(true);
  }, []);
  useEffect(() => {
    if (open) void import("./mwa").then((m) => m.registerMobileWalletAdapter());
  }, [open]);

  // A wallet that was authorised before may expose its accounts only after a
  // silent connect (Wallet Standard `connect({ silent: true })`); ask once per
  // page load and never prompt — a prompt is the user's click, not ours.
  const [silentTried, setSilentTried] = useState<string | null>(null);
  useEffect(() => {
    if (!persisted) return;
    const wallet = wallets.find((w) => w.name === persisted.wallet);
    if (!wallet || wallet.accounts.some((a) => a.address === persisted.address)) return;
    const key = `${persisted.wallet}:${persisted.address}`;
    if (silentTried === key) return;
    setSilentTried(key);
    try {
      const feature = getWalletFeature(wallet, "standard:connect") as { connect: (input?: { silent?: boolean }) => Promise<unknown> };
      void feature.connect({ silent: true }).catch(() => undefined);
    } catch {
      /* the wallet has no connect feature; nothing to resume */
    }
  }, [persisted, wallets, silentTried]);

  // Resolve the persisted choice against wallets that are already authorised
  // for this origin (their `accounts` are populated without a prompt).
  const selected = useMemo<Selected | null>(() => {
    if (!persisted) return null;
    const wallet = wallets.find((w) => w.name === persisted.wallet);
    if (!wallet) return null;
    const account = wallet.accounts.find((a) => a.address === persisted.address);
    return account ? { wallet, account } : null;
  }, [persisted, wallets]);

  const select = useCallback((wallet: UiWallet, account: UiWalletAccount) => {
    const v = { wallet: wallet.name, address: account.address };
    writePersisted(v);
    setPersisted(v);
    setOpen(false);
  }, []);

  const disconnect = useCallback(async () => {
    const current = selected;
    writePersisted(null);
    setPersisted(null);
    if (current && current.wallet.features.includes("standard:disconnect")) {
      try {
        const feature = getWalletFeature(current.wallet, "standard:disconnect") as { disconnect: () => Promise<void> };
        await feature.disconnect();
      } catch {
        /* the wallet may refuse; the Terminal has already forgotten it */
      }
    }
  }, [selected]);

  const value = useMemo<WalletContextValue>(
    () => ({ wallets, selected, select, disconnect, open, setOpen, ready }),
    [wallets, selected, select, disconnect, open, ready],
  );

  return <WalletContext.Provider value={value}>{children}</WalletContext.Provider>;
}

export function useWallet(): WalletContextValue {
  const ctx = useContext(WalletContext);
  if (!ctx) throw new Error("useWallet must be used within WalletProvider");
  return ctx;
}
