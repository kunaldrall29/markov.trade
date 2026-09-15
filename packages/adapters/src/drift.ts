import { CAPS } from "@markov/facts";
import type { AdapterCapabilities } from "./types.ts";

/**
 * Drift hosted HTTP did not resolve from this network on 2026-09-15
 * (`dlob.drift.trade`, `data.api.drift.trade`). Reads go through Solana RPC +
 * the official SDK in the API process when RPC is configured. Until then the
 * adapter is present and **not executable**, failing closed.
 */
export function driftCapabilities(env: "devnet" | "mainnet"): AdapterCapabilities {
  return {
    id: "drift",
    env,
    execution_model: "onchain-amm+dlob",
    order_types: ["market", "limit", "oracle"],
    margin_modes: ["cross"],
    delegation_model: "subaccount-delegate-unverified",
    onchain_enforceable: false,
    executable: false,
    freshness_ms: CAPS.freshnessDriftSlots * 400,
  };
}
