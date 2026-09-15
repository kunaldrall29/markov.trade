/**
 * Solana Mobile Wallet Adapter as a Wallet Standard wallet, so a phone's
 * wallet app (Phantom, Solflare, Seed Vault…) shows up next to injected
 * browser wallets. Registered once per page; the library dedupes.
 */
import {
  createDefaultAuthorizationCache,
  createDefaultChainSelector,
  createDefaultWalletNotFoundHandler,
  registerMwa,
} from "@solana-mobile/wallet-standard-mobile";
import { solanaChain } from "@/lib/config";

let registered = false;

export function registerMobileWalletAdapter(): void {
  if (registered || typeof window === "undefined") return;
  registered = true;
  try {
    registerMwa({
      appIdentity: { name: "Markov", uri: window.location.origin, icon: "favicon.svg" },
      authorizationCache: createDefaultAuthorizationCache(),
      chains: [solanaChain],
      chainSelector: createDefaultChainSelector(),
      onWalletNotFound: createDefaultWalletNotFoundHandler(),
    });
  } catch (e) {
    // A registration failure leaves the injected wallets untouched.
    if (import.meta.env.DEV) console.info("mobile wallet adapter not registered:", e instanceof Error ? e.message : e);
  }
}
