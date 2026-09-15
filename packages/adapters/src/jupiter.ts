import { CAPS, JUPITER_SWAP_QUOTE, JUPITER_TOKENS_V2, USDC_MAINNET } from "@markov/facts";
import { type AdapterCapabilities, getJson } from "./types.ts";

export function jupiterCapabilities(): AdapterCapabilities {
  return {
    id: "jupiter",
    env: "mainnet",
    execution_model: "aggregator-swap",
    order_types: ["swap"],
    margin_modes: [],
    delegation_model: "subscriptions-allowance",
    onchain_enforceable: false,
    executable: true,
    freshness_ms: CAPS.freshnessJupiterQuoteMs,
  };
}

export type JupiterQuote = {
  inputMint: string;
  outputMint: string;
  inAmount: string;
  outAmount: string;
  priceImpactPct: string | number;
  contextSlot: number;
  timeTaken?: number;
  routePlan: unknown[];
  fetchedAt: number;
  costBps: number;
};

export async function jupiterToken(query: string): Promise<{ id: string; symbol: string; decimals: number; tokenProgram: string } | null> {
  const rows = await getJson<Array<{ id: string; symbol: string; decimals: number; tokenProgram: string }>>(
    `${JUPITER_TOKENS_V2}?query=${encodeURIComponent(query)}`,
  );
  return rows[0] ?? null;
}

export async function jupiterQuote(input: {
  inputMint?: string;
  outputMint: string;
  amount: number;
  slippageBps?: number;
}): Promise<JupiterQuote> {
  const inputMint = input.inputMint ?? USDC_MAINNET;
  const url = `${JUPITER_SWAP_QUOTE}?inputMint=${inputMint}&outputMint=${input.outputMint}&amount=${input.amount}&slippageBps=${input.slippageBps ?? 50}`;
  const q = await getJson<JupiterQuote>(url);
  const impact = Number(q.priceImpactPct) || 0;
  const costBps = Math.round(impact * 10_000);
  return { ...q, fetchedAt: Date.now(), costBps };
}
