import { PublicKey } from "@solana/web3.js";
import { PROGRAM_ID, SEEDS } from "@markov/facts";
import type { Receipt, MarketStateDto, PolicyPreview, VenueCapabilities } from "./types.ts";

export * from "./types.ts";

export function programId(override?: string): PublicKey {
  return new PublicKey(override || PROGRAM_ID);
}

export function deriveAccount(owner: PublicKey, pid?: PublicKey): PublicKey {
  return PublicKey.findProgramAddressSync(
    [Buffer.from(SEEDS.account), owner.toBuffer()],
    pid ?? programId(),
  )[0];
}

export function deriveConfig(pid?: PublicKey): PublicKey {
  return PublicKey.findProgramAddressSync([Buffer.from(SEEDS.config)], pid ?? programId())[0];
}

export function deriveMandate(account: PublicKey, version: number, pid?: PublicKey): PublicKey {
  const v = Buffer.alloc(4);
  v.writeUInt32LE(version);
  return PublicKey.findProgramAddressSync(
    [Buffer.from(SEEDS.mandate), account.toBuffer(), v],
    pid ?? programId(),
  )[0];
}

export function deriveReceipt(account: PublicKey, requestId: bigint, pid?: PublicKey): PublicKey {
  const id = Buffer.alloc(16);
  id.writeBigUInt64LE(requestId & 0xffffffffffffffffn, 0);
  id.writeBigUInt64LE(requestId >> 64n, 8);
  return PublicKey.findProgramAddressSync(
    [Buffer.from(SEEDS.receipt), account.toBuffer(), id],
    pid ?? programId(),
  )[0];
}

export function explorerTx(signature: string, cluster: "devnet" | "mainnet-beta"): string {
  const c = cluster === "devnet" ? "?cluster=devnet" : "";
  return `https://explorer.solana.com/tx/${signature}${c}`;
}

export type ApiClientOpts = { baseUrl: string; token?: string };

async function req<T>(opts: ApiClientOpts, path: string, init?: RequestInit): Promise<T> {
  const headers: Record<string, string> = { accept: "application/json", ...(init?.headers as Record<string, string>) };
  if (opts.token) headers.authorization = `Bearer ${opts.token}`;
  if (init?.body && !headers["content-type"]) headers["content-type"] = "application/json";
  const res = await fetch(`${opts.baseUrl}${path}`, { ...init, headers });
  if (!res.ok) {
    const text = await res.text();
    throw new Error(`${res.status} ${path}: ${text}`);
  }
  return res.json() as Promise<T>;
}

export function createApiClient(opts: ApiClientOpts) {
  return {
    health: () => req<{ ok: boolean; env: string; slot: number }>(opts, "/health"),
    markets: () => req<{ env: string; data_slot: number; markets: MarketStateDto[] }>(opts, "/markets"),
    market: (id: string) => req<{ env: string; market: MarketStateDto; venues: MarketStateDto[] }>(opts, `/markets/${id}`),
    candles: (id: string) => req<{ candles: Array<{ openTime: number; open: number; high: number; low: number; close: number }> }>(opts, `/markets/${id}/candles`),
    capabilities: () => req<{ env: string; venues: VenueCapabilities[] }>(opts, "/venues/capabilities"),
    policyCheck: (body: unknown) =>
      req<PolicyPreview>(opts, "/policy/check", { method: "POST", body: JSON.stringify(body) }),
    routes: (body: unknown) => req<unknown>(opts, "/routes/compare", { method: "POST", body: JSON.stringify(body) }),
    receipts: () => req<{ env: string; receipts: Receipt[] }>(opts, "/receipts"),
    receipt: (id: string) => req<{ env: string; receipt: Receipt }>(opts, `/receipts/${id}`),
    account: () => req<unknown>(opts, "/account"),
    portfolio: () => req<unknown>(opts, "/portfolio"),
    positions: () => req<unknown>(opts, "/positions"),
    mandate: () => req<unknown>(opts, "/mandate"),
    investPropose: (body: unknown) =>
      req<unknown>(opts, "/invest/propose", { method: "POST", body: JSON.stringify(body) }),
    investRules: () => req<unknown>(opts, "/invest/rules"),
    risk: () => req<unknown>(opts, "/risk"),
    challenge: (pubkey: string) =>
      req<{ nonce: string; message: string; expires_at: string }>(opts, "/auth/challenge", {
        method: "POST",
        body: JSON.stringify({ pubkey }),
      }),
    verify: (pubkey: string, signature: string, nonce: string) =>
      req<{ token: string; refresh: string }>(opts, "/auth/verify", {
        method: "POST",
        body: JSON.stringify({ pubkey, signature, nonce }),
      }),
  };
}
