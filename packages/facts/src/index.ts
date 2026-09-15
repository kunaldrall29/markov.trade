/** Typed constants copied from docs/FACTS.md. CI fails if this file drifts. */

export const FACTS_VERIFIED = "2026-09-15" as const;

export const DEVNET_GENESIS = "EtWTRABZaYq6iMfeYKouRu166VU2xqa1wcaWoxPkrZBG";
export const MAINNET_GENESIS = "5eykt4UsFv8P8NJdTREpY1vzqKqZKvdpKuc147dw2N9d";

export const PROGRAM_ID = "37rW4ETzh8o7ebWFnrJRKt7iCUvWdxsv37vPENjAYz1J";

export const DRIFT_V2_PROGRAM = "dRiftyHA39MWEi3m9aunc5MzRF1JYuBsbn6VPcn33UH";
export const PHOENIX_PROD = "EtrnLzgbS7nMMy5fbD42kXiUzGg8XQzJ972Xtk1cjWih";
export const PHOENIX_BETA = "phDEVv4w6BcfkLrLNeXr8HhhgQxnxziVGXpGPcaadMf";
export const HAWKEYE = "RiSeVw3ZjNfsaXPRb4mgaqYaEEt41pNNJoDvVh7pgQj";
export const EMBER = "EMBERpYNE6ehWmXymZZS2skiFmCa9V5dp14e1iduM5qy";
export const FLIGHT = "F1ightu9cujFYo34k9CabifLrJT8qzfDVM2Q7BqhJn2W";
export const SUBSCRIPTIONS_PROGRAM = "De1egAFMkMWZSN5rYXRj9CAdheBamobVNubTsi9avR44";
export const TOKEN_2022 = "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb";
export const TOKEN_PROGRAM = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";
export const JUPITER_PERPS = "PERPHjGBqRHArX4DySjwM6UJHiR3sWAatqfdBS2qQJu";

export const PHOENIX_REST = "https://perp-api.phoenix.trade";
export const PHOENIX_WS = "wss://perp-api.phoenix.trade/v1/ws";
export const PACIFICA_REST = "https://api.pacifica.fi/api/v1";
export const PACIFICA_TESTNET = "https://test-api.pacifica.fi/api/v1";
export const JUPITER_TOKENS_V2 = "https://lite-api.jup.ag/tokens/v2/search";
export const JUPITER_SWAP_QUOTE = "https://lite-api.jup.ag/swap/v1/quote";
export const JUPITER_SWAP = "https://lite-api.jup.ag/swap/v1/swap";
export const JUPITER_SWAP_INSTRUCTIONS = "https://lite-api.jup.ag/swap/v1/swap-instructions";

export const USDC_MAINNET = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
export const TSLAX = "XsDoVfqeBukxuZHWhdvWHBhgEHjGNst4MLodqsJHzoB";
export const NVDAX = "Xsc9qvGR1efVDFGLrVsmkzv3qi45LTBjeUKSPmx9qEh";
export const SPYX = "XsoCS1TfEyfFhfvj8EtZ528L3CaKBDBRqRapnBbDF2W";
export const AAPLX = "XsbEhLAtcf6HdfpFZ5xEMdqW8nfAvcsP5bdudRLJzJp";

export const XSTOCKS = [
  { symbol: "TSLAx", mint: TSLAX, decimals: 8 },
  { symbol: "NVDAx", mint: NVDAX, decimals: 8 },
  { symbol: "SPYx", mint: SPYX, decimals: 8 },
  { symbol: "AAPLx", mint: AAPLX, decimals: 8 },
] as const;

export const CAPS = {
  perAccountGrossNotionalUsd: 2000,
  perPositionLeverageMajors: 2.0,
  investMonthlyBudgetUsd: 500,
  investExecutionMinUsd: 5,
  investExecutionMaxUsd: 100,
  investKeeperGlobalDailyUsd: 2000,
  freshnessPacificaMs: 3000,
  freshnessDriftSlots: 2,
  freshnessPhoenixMs: 3000,
  freshnessJupiterQuoteMs: 10000,
  freshnessOfficialPriceMs: 15000,
  investMaxCostBpsDefault: 40,
} as const;

export const STAGE = "TEST STAGE" as const;

export const VENUE = {
  PACIFICA: 1,
  DRIFT: 2,
  PHOENIX: 3,
  JUPITER: 4,
  SUBSCRIPTIONS: 5,
} as const;

export const VENUE_BIT = {
  PACIFICA: 1 << 0,
  DRIFT: 1 << 1,
  PHOENIX: 1 << 2,
  JUPITER: 1 << 3,
} as const;

export const REASON = {
  MARKET_NOT_ALLOWED: 1,
  MAX_LEVERAGE_EXCEEDED: 2,
  MAX_NOTIONAL_EXCEEDED: 3,
  MIN_SAFETY_BUFFER: 4,
  DAILY_LOSS_BUDGET_EXCEEDED: 5,
  STALE_MARKET_DATA: 6,
  SLIPPAGE_LIMIT: 7,
  ACTOR_SCOPE_DENIED: 8,
  ACTOR_CAP_EXCEEDED: 9,
  VENUE_NOT_ALLOWED: 10,
  ACCOUNT_PAUSED: 11,
  GLOBAL_PAUSED: 12,
  ASSET_NOT_ALLOWLISTED: 20,
  BUDGET_EXHAUSTED: 21,
  MONTHLY_CEILING_EXCEEDED: 22,
  RESERVE_FLOOR: 23,
  COST_LIMIT: 24,
  MARKET_CLOSED: 25,
  OFF_HOURS_DEVIATION: 26,
  ASSET_PAUSED: 27,
  NO_ROUTE: 28,
  STALE_QUOTE: 29,
  DELEGATION_INSUFFICIENT: 30,
  SINGLE_ASSET_CAP: 31,
  EXECUTION_CONFIRMED: 100,
  EXECUTION_FAILED: 101,
} as const;

export const REASON_NAME: Record<number, string> = Object.fromEntries(
  Object.entries(REASON).map(([k, v]) => [v, k]),
);

export const CANONICAL_MARKETS = [
  { id: "SOL-PERP", pacifica: "SOL", phoenix: "SOL", driftIndex: 0, category: "majors" as const, tier: 1 },
  { id: "BTC-PERP", pacifica: "BTC", phoenix: "BTC", driftIndex: 1, category: "majors" as const, tier: 1 },
  { id: "ETH-PERP", pacifica: "ETH", phoenix: "ETH", driftIndex: 2, category: "majors" as const, tier: 1 },
  { id: "DOGE-PERP", pacifica: "DOGE", phoenix: "DOGE", driftIndex: 7, category: "long-tail" as const, tier: 3 },
  { id: "FARTCOIN-PERP", pacifica: "FARTCOIN", phoenix: "FARTCOIN", driftIndex: null, category: "long-tail" as const, tier: 4 },
  { id: "PUMP-PERP", pacifica: "PUMP", phoenix: "PUMP", driftIndex: null, category: "long-tail" as const, tier: 4 },
] as const;

export function marketIdBytes(id: string): Uint8Array {
  const out = new Uint8Array(16);
  const enc = new TextEncoder().encode(id);
  out.set(enc.slice(0, 16));
  return out;
}

export const SEEDS = {
  account: "account",
  mandate: "mandate",
  invest: "invest",
  perm: "perm",
  receipt: "receipt",
  day: "day",
  config: "config",
} as const;

export const FORBIDDEN_CLAIMS = [
  "best execution",
  "guaranteed",
  "never get liquidated",
  "ai-powered trading",
  "exchange price 24/7",
  "nasdaq price 24/7",
  "jupiter perps",
  "illustrative",
  "demo mode",
  "mock mode",
] as const;
