/**
 * Runtime configuration for the Terminal. Program ids and mints come from
 * `@markov/sdk` (FACTS); only endpoints and the stage label live here.
 */
import { CLUSTER, DEVNET_RPC_FALLBACK, DEVNET_RPC_PRIMARY, SOLANA_CHAIN } from "@markov/sdk";

export const cluster = CLUSTER;
export const solanaChain = SOLANA_CHAIN;

/** Browser RPC. `VITE_RPC_URL` may point at a keyed endpoint; the default is the keyless one FACTS proved. */
export const rpcUrl: string = import.meta.env.VITE_RPC_URL || DEVNET_RPC_PRIMARY;
export const rpcFallbackUrl: string = import.meta.env.VITE_RPC_FALLBACK || DEVNET_RPC_FALLBACK;

/** Stage label. Exact words, everywhere it applies (conventions §8). */
export const stageLabel = "TEST STAGE";
export const stageLine = "devnet · marked PnL, not a promised rate";

/** The build-flagged test wallet (`VITE_TEST_WALLET=1`) is for Playwright only; it is dead code otherwise. */
export const testWalletEnabled = import.meta.env.VITE_TEST_WALLET === "1";
