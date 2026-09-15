import { createRpc, type MarkovRpc } from "@markov/sdk";
import { rpcFallbackUrl, rpcUrl } from "./config";

let primary: MarkovRpc | undefined;
let fallback: MarkovRpc | undefined;

export function rpc(): MarkovRpc {
  return (primary ??= createRpc(rpcUrl));
}

/**
 * Read through the primary endpoint, then the fallback (FACTS names both).
 * Both failing is an error that names both, never a stale value.
 */
export async function readEither<T>(fn: (rpc: MarkovRpc) => Promise<T>): Promise<T> {
  try {
    return await fn(rpc());
  } catch (primaryErr) {
    if (rpcFallbackUrl === rpcUrl) throw primaryErr;
    try {
      return await fn((fallback ??= createRpc(rpcFallbackUrl)));
    } catch (fallbackErr) {
      throw new Error(
        `both RPC endpoints failed: ${rpcUrl}: ${describe(primaryErr)}; ${rpcFallbackUrl}: ${describe(fallbackErr)}`,
      );
    }
  }
}

export function describe(e: unknown): string {
  if (e instanceof Error) return e.message;
  return String(e);
}
