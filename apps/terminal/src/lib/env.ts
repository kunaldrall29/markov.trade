import { STAGE, PROGRAM_ID } from "@markov/facts";

export const envName = (process.env.NEXT_PUBLIC_ENV || "devnet") as "devnet" | "rehearsal" | "mainnet";
export const apiUrl = process.env.NEXT_PUBLIC_API_URL || "/v1";
export const cluster = (process.env.NEXT_PUBLIC_CLUSTER || "devnet") as "devnet" | "mainnet-beta";
export const stageLabel =
  envName === "mainnet" ? "CAPPED MAINNET" : envName === "rehearsal" ? "REHEARSAL" : STAGE;
export const programId = process.env.NEXT_PUBLIC_PROGRAM_ID || PROGRAM_ID;
export const rpcUrl =
  process.env.NEXT_PUBLIC_RPC_URL ||
  (cluster === "devnet" ? "https://api.devnet.solana.com" : "https://api.mainnet-beta.solana.com");
