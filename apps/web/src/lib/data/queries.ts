/**
 * React Query hooks. Feed and book stats come from the same-origin read API
 * (cached chain reads); the connected owner's own state is read from the
 * chain directly so the wallet page never depends on the cache.
 */
import { useQuery } from "@tanstack/react-query";
import {
  DEMO_PERPS_PROGRAM_ID,
  PYTH_SOL_USD_PRICE_UPDATE,
  SOL_PERP_MARKET_ID,
  USDC_D_MINT,
  fetchMandatesByOwner,
  fetchPythPrice,
  fetchRegistry,
  fetchTokenBalance,
  fetchVenueView,
  ownerAta,
  walkReceipts,
  type MandateView,
} from "@markov/sdk";
import type { Address } from "@solana/kit";
import { api } from "./api";
import { FAST_POLL, SLOW_POLL } from "./polling";
import { readEither } from "@/lib/rpc";

export function useHealth() {
  return useQuery({ queryKey: ["health"], queryFn: api.health, ...SLOW_POLL });
}

export function useBookStats() {
  return useQuery({ queryKey: ["book-stats"], queryFn: api.bookStats, ...FAST_POLL });
}

export function useReceiptFeed(opts: { mandate?: string; limit?: number; before?: string } = {}) {
  return useQuery({ queryKey: ["receipts", opts.mandate ?? "program", opts.limit ?? 50, opts.before ?? ""], queryFn: () => api.receipts(opts), ...FAST_POLL });
}

export function useOwnerMandates(owner: Address | null) {
  return useQuery({
    queryKey: ["owner-mandates", owner],
    queryFn: () => readEither((rpc) => fetchMandatesByOwner(rpc, owner!)),
    enabled: !!owner,
    ...FAST_POLL,
  });
}

export function useTokenBalance(account: Address | null, mint: Address = USDC_D_MINT) {
  return useQuery({
    queryKey: ["token-balance", account, mint],
    queryFn: () => readEither((rpc) => fetchTokenBalance(rpc, account!, mint)),
    enabled: !!account,
    ...FAST_POLL,
  });
}

export function useOwnerUsdcBalance(owner: Address | null) {
  return useQuery({
    queryKey: ["owner-usdc", owner],
    queryFn: async () => {
      const ata = await ownerAta(owner!, USDC_D_MINT);
      return readEither((rpc) => fetchTokenBalance(rpc, ata, USDC_D_MINT));
    },
    enabled: !!owner,
    ...FAST_POLL,
  });
}

export function useVenueView(mandate: Address | null) {
  return useQuery({
    queryKey: ["venue", mandate],
    queryFn: () => readEither((rpc) => fetchVenueView(rpc, mandate!, SOL_PERP_MARKET_ID, DEMO_PERPS_PROGRAM_ID)),
    enabled: !!mandate,
    ...FAST_POLL,
  });
}

export function usePythMark(account: Address = PYTH_SOL_USD_PRICE_UPDATE) {
  return useQuery({ queryKey: ["pyth", account], queryFn: () => readEither((rpc) => fetchPythPrice(rpc, account)), ...FAST_POLL });
}

export function useRegistry() {
  return useQuery({ queryKey: ["registry"], queryFn: () => readEither((rpc) => fetchRegistry(rpc)), ...SLOW_POLL });
}

/** Receipts for one mandate read straight from the chain (the owner's page). */
export function useMandateReceipts(mandate: Address | null, limit = 50) {
  return useQuery({
    queryKey: ["mandate-receipts", mandate, limit],
    queryFn: () => readEither((rpc) => walkReceipts(rpc, { address: mandate!, limit, concurrency: 3 })),
    enabled: !!mandate,
    ...FAST_POLL,
  });
}

export type { MandateView };
