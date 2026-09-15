import {
  CAPS,
  DRIFT_V2_PROGRAM,
  JUPITER_SWAP_QUOTE,
  JUPITER_SWAP,
  JUPITER_SWAP_INSTRUCTIONS,
  JUPITER_TOKENS_V2,
  PACIFICA_REST,
  PACIFICA_TESTNET,
  PHOENIX_REST,
  PHOENIX_WS,
  PROGRAM_ID,
  STAGE,
  SUBSCRIPTIONS_PROGRAM,
} from "@markov/facts";

export type MarkovEnvName = "devnet" | "rehearsal" | "mainnet";

export type EnvConfig = {
  name: MarkovEnvName;
  stageLabel: string;
  cluster: "devnet" | "mainnet-beta";
  genesis: string;
  rpcHttp: string;
  rpcWs: string;
  rpcFallback: string;
  programId: string;
  subscriptionsProgram: string;
  driftProgram: string;
  pacificaApi: string;
  phoenixApi: string;
  phoenixWs: string;
  jupiterQuote: string;
  jupiterSwap: string;
  jupiterSwapIx: string;
  jupiterTokens: string;
  phoenixExecutable: boolean;
  pacificaWrites: boolean;
  driftWrites: boolean;
  investLiveSwaps: boolean;
  caps: typeof CAPS;
};

const GENESIS = {
  devnet: "EtWTRABZaYq6iMfeYKouRu166VU2xqa1wcaWoxPkrZBG",
  "mainnet-beta": "5eykt4UsFv8P8NJdTREpY1vzqKqZKvdpKuc147dw2N9d",
} as const;

export function loadEnv(name: MarkovEnvName = readName()): EnvConfig {
  const cluster: EnvConfig["cluster"] = name === "devnet" ? "devnet" : "mainnet-beta";
  const mainnetish = name !== "devnet";
  return {
    name,
    stageLabel: name === "devnet" ? STAGE : name === "rehearsal" ? "REHEARSAL" : "CAPPED MAINNET",
    cluster,
    genesis: GENESIS[cluster],
    rpcHttp: process.env.RPC_HTTP_URL || (cluster === "devnet"
      ? "https://api.devnet.solana.com"
      : "https://api.mainnet-beta.solana.com"),
    rpcWs: process.env.RPC_WS_URL || (cluster === "devnet"
      ? "wss://api.devnet.solana.com"
      : "wss://api.mainnet-beta.solana.com"),
    rpcFallback: process.env.RPC_HTTP_FALLBACK || "https://api.devnet.solana.com",
    programId: process.env.MARKOV_PROGRAM_ID || PROGRAM_ID,
    subscriptionsProgram: SUBSCRIPTIONS_PROGRAM,
    driftProgram: DRIFT_V2_PROGRAM,
    pacificaApi: mainnetish ? PACIFICA_REST : PACIFICA_TESTNET,
    phoenixApi: PHOENIX_REST,
    phoenixWs: PHOENIX_WS,
    jupiterQuote: JUPITER_SWAP_QUOTE,
    jupiterSwap: JUPITER_SWAP,
    jupiterSwapIx: JUPITER_SWAP_INSTRUCTIONS,
    jupiterTokens: JUPITER_TOKENS_V2,
    phoenixExecutable: false,
    pacificaWrites: true,
    driftWrites: false,
    investLiveSwaps: mainnetish,
    caps: CAPS,
  };
}

export function readName(): MarkovEnvName {
  const v = (process.env.MARKOV_ENV || process.env.NEXT_PUBLIC_ENV || "devnet").toLowerCase();
  if (v === "mainnet" || v === "rehearsal" || v === "devnet") return v;
  throw new Error(`unknown MARKOV_ENV ${v}`);
}
