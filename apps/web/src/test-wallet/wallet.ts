/**
 * A Wallet Standard wallet for Playwright. Bundled only when the build sets
 * `VITE_TEST_WALLET=1`; never part of a production bundle.
 *
 * Two modes, chosen by `VITE_TEST_WALLET_MODE`:
 *  - `capture` (default): the wallet signs nothing and sends nothing. Every
 *    transaction it is handed is decoded and recorded on
 *    `window.__markovTestWallet.captured`, then the call fails with a message
 *    the UI shows verbatim. Tests assert on what was built, not on a fake
 *    signature.
 *  - `send`: signs with the keypair in `VITE_TEST_WALLET_SECRET` (64 bytes,
 *    base58) and submits to the configured RPC. Real devnet, real receipts.
 */
import {
  createKeyPairSignerFromBytes,
  createKeyPairSignerFromPrivateKeyBytes,
  getBase58Encoder,
  getBase64EncodedWireTransaction,
  getTransactionDecoder,
  getCompiledTransactionMessageDecoder,
  type KeyPairSigner,
} from "@solana/kit";
import type { Wallet, WalletAccount } from "@wallet-standard/base";
import { registerWallet } from "@wallet-standard/wallet";
import { rpcUrl, solanaChain } from "@/lib/config";

type Captured = { instructions: { programAddress: string; data: number[]; accounts: string[] }[]; feePayer: string; wire: string };

declare global {
  interface Window {
    __markovTestWallet?: { address: string; mode: "capture" | "send"; captured: Captured[] };
  }
}

const ICON = "data:image/svg+xml;base64," + btoa('<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 32"><rect width="32" height="32" rx="6" fill="#2c6b48"/><text x="16" y="21" font-size="14" text-anchor="middle" fill="#fff" font-family="monospace">T</text></svg>');

async function loadSigner(): Promise<KeyPairSigner> {
  const secret = import.meta.env.VITE_TEST_WALLET_SECRET as string | undefined;
  if (secret) return createKeyPairSignerFromBytes(getBase58Encoder().encode(secret) as Uint8Array);
  // No configured key: a throwaway seed that survives reloads within this tab,
  // so a test can reload the page and still be "the same wallet".
  const key = "markov-test-wallet-seed";
  let hex = sessionStorage.getItem(key);
  if (!hex) {
    const seed = crypto.getRandomValues(new Uint8Array(32));
    hex = Array.from(seed, (b) => b.toString(16).padStart(2, "0")).join("");
    sessionStorage.setItem(key, hex);
  }
  const bytes = new Uint8Array(hex.match(/.{2}/g)!.map((h) => parseInt(h, 16)));
  return createKeyPairSignerFromPrivateKeyBytes(bytes);
}

function decodeForCapture(bytes: Uint8Array, feePayer: string): Captured {
  const tx = getTransactionDecoder().decode(bytes);
  const msg = getCompiledTransactionMessageDecoder().decode(tx.messageBytes) as unknown as {
    staticAccounts: readonly string[];
    instructions: readonly { programAddressIndex: number; accountIndices?: readonly number[]; data?: Uint8Array }[];
  };
  const keys = msg.staticAccounts.map(String);
  return {
    feePayer,
    wire: getBase64EncodedWireTransaction(tx),
    instructions: msg.instructions.map((ix) => ({
      programAddress: keys[ix.programAddressIndex] ?? "?",
      data: Array.from(ix.data ?? []),
      accounts: (ix.accountIndices ?? []).map((i) => keys[i] ?? "?"),
    })),
  };
}

export async function registerTestWallet(): Promise<void> {
  if (typeof window === "undefined" || window.__markovTestWallet) return;
  const mode = (import.meta.env.VITE_TEST_WALLET_MODE as "capture" | "send" | undefined) ?? "capture";
  const signer = await loadSigner();
  const account: WalletAccount = {
    address: signer.address,
    publicKey: getBase58Encoder().encode(signer.address) as Uint8Array,
    chains: [solanaChain],
    features: ["solana:signTransaction", "solana:signAndSendTransaction"],
    label: "Markov test account",
  };
  const state = { address: signer.address, mode, captured: [] as Captured[] };
  window.__markovTestWallet = state;
  let connected: WalletAccount[] = [];
  const listeners = new Set<(props: { accounts: readonly WalletAccount[] }) => void>();
  const emit = () => listeners.forEach((l) => l({ accounts: connected }));

  async function sendWire(wire: string): Promise<Uint8Array> {
    const res = await fetch(rpcUrl, {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ jsonrpc: "2.0", id: 1, method: "sendTransaction", params: [wire, { encoding: "base64", preflightCommitment: "confirmed" }] }),
    });
    const body = (await res.json()) as { result?: string; error?: { message: string } };
    if (!body.result) throw new Error(`sendTransaction failed: ${body.error?.message ?? "no result"}`);
    return getBase58Encoder().encode(body.result) as Uint8Array;
  }

  const wallet: Wallet = {
    version: "1.0.0",
    name: "Markov Test Wallet",
    icon: ICON as Wallet["icon"],
    chains: [solanaChain],
    get accounts() {
      return connected;
    },
    features: {
      "standard:connect": {
        version: "1.0.0",
        connect: async (input?: { silent?: boolean }) => {
          // Like a trusted wallet: a silent connect succeeds only if this tab
          // already granted access; a normal connect grants it.
          const granted = sessionStorage.getItem("markov-test-wallet-granted") === "1";
          if (input?.silent && !granted) return { accounts: connected };
          sessionStorage.setItem("markov-test-wallet-granted", "1");
          connected = [account];
          emit();
          return { accounts: connected };
        },
      },
      "standard:disconnect": {
        version: "1.0.0",
        disconnect: async () => {
          sessionStorage.removeItem("markov-test-wallet-granted");
          connected = [];
          emit();
        },
      },
      "standard:events": {
        version: "1.0.0",
        on: (_event: "change", listener: (props: { accounts: readonly WalletAccount[] }) => void) => {
          listeners.add(listener);
          return () => listeners.delete(listener);
        },
      },
      "solana:signTransaction": {
        version: "1.0.0",
        supportedTransactionVersions: ["legacy", 0],
        signTransaction: async (...inputs: { transaction: Uint8Array }[]) => {
          const out = [];
          for (const input of inputs) {
            if (mode === "capture") {
              state.captured.push(decodeForCapture(input.transaction, signer.address));
              throw new Error("test wallet (capture mode): transaction recorded, not signed");
            }
            const tx = getTransactionDecoder().decode(input.transaction) as never;
            const [signed] = await signer.signTransactions([tx]);
            const bytes = getBase58Encoder().encode(getBase64EncodedWireTransaction({ ...(tx as object), signatures: { ...(tx as { signatures: object }).signatures, ...signed } } as never));
            out.push({ signedTransaction: bytes as Uint8Array });
          }
          return out;
        },
      },
      "solana:signAndSendTransaction": {
        version: "1.0.0",
        supportedTransactionVersions: ["legacy", 0],
        signAndSendTransaction: async (...inputs: { transaction: Uint8Array }[]) => {
          const out = [];
          for (const input of inputs) {
            const captured = decodeForCapture(input.transaction, signer.address);
            state.captured.push(captured);
            if (mode === "capture") throw new Error("test wallet (capture mode): transaction recorded, not sent");
            const tx = getTransactionDecoder().decode(input.transaction) as never;
            const [sigs] = await signer.signTransactions([tx]);
            const wire = getBase64EncodedWireTransaction({ ...(tx as object), signatures: { ...(tx as { signatures: object }).signatures, ...sigs } } as never);
            out.push({ signature: await sendWire(wire) });
          }
          return out;
        },
      },
    } as Wallet["features"],
  };
  registerWallet(wallet);
}
