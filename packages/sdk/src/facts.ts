/**
 * Constants every surface must read from here and nowhere else.
 *
 * Each value is copied from a named row in `docs/FACTS.md`; `pnpm facts:check`
 * fails if any value below is absent from that file, so a change on chain
 * has to go through FACTS first (conventions §1 "Facts come from FACTS").
 * Nothing in this file is typed from memory.
 */
import { address, type Address } from "@solana/kit";

/** FACTS `PROGRAM_ID` (P02, upgraded at slot 492602088 with gate 15). */
export const MANDATE_PROGRAM_ID: Address = address("25CdYaZeB18QvUR7cTyZPgTZPNREb7t6xL8zmk1eXAU6");
/** FACTS `DEMO_PERPS_ID` (P04, the mock venue: zero token custody, deterministic fills). */
export const DEMO_PERPS_PROGRAM_ID: Address = address("3Zcd8XsFWBTVku5GxQjwEBC7sLrJhF8vadyTnTr56hxB");
/** FACTS `USDC_D_MINT` — the Gate B settlement mint, decimals 6. */
export const USDC_D_MINT: Address = address("7ajorFYMrE9Mi3yZkwWaZp6ahzkK6RotZ75qAdtTV9Rj");
export const USDC_D_DECIMALS = 6;
/** FACTS `SOL_D_MINT` — decimals 9, in the policy allowlist, never minted. */
export const SOL_D_MINT: Address = address("73V1Vhs3A8j8NrXKCbGmRek2dR92x9MkwUk4WEdYYRfQ");
/** FACTS `PYTH_DEVNET_FEED_ACCOUNT` — SOL/USD `PriceUpdateV2`, shard 0, owned by the receiver. */
export const PYTH_SOL_USD_PRICE_UPDATE: Address = address("7UVimffxr9ow1uXYxsr4LHAcV58mLzhmwaeKvJ1pjLiE");
/** FACTS `PYTH_RECEIVER_PROGRAM` — owner of every `PriceUpdateV2` the program will read. */
export const PYTH_RECEIVER_PROGRAM_ID: Address = address("rec5EKMGg6MxZYaMdyBfgwp4d5rB9T1VQH5pJv5LtFJ");
/** FACTS `SOL_USD_FEED_ID` (`Crypto.SOL/USD`). */
export const SOL_USD_FEED_ID_HEX = "ef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d";
/** FACTS `GATE_B_MANDATE` — the house book's mandate on devnet (owner `5RPxDN9h…`, nonce 10). */
export const GATE_B_MANDATE: Address = address("2ivTE7hwgW9nzCQzRXm2ED1htArg37BgZXhLyAh2TMTo");
export const GATE_B_MANDATE_VAULT: Address = address("Am8xa8dQxVUKp7CYwKnXkFRYn6G3NeAVWmKaSF9mR6Yu");
export const GATE_B_MANDATE_OWNER: Address = address("5RPxDN9hxG3YaBSZo6TWfDx1CUGpJKFsuP3hmjUkd1Hv");
export const GATE_B_MANDATE_NONCE = 10n;
export const GATE_B_VENUE_POSITION: Address = address("GJj2vgMrkfPxBTTc4zRrV7yxpaBPkRYhJ7igH86ZpQUq");
/** FACTS `OPERATOR_PUBKEY` — the house agent's propose-only key. */
export const BOOK_ONE_OPERATOR: Address = address("EU8a73vNg3Ti4DXtnXF41JLhc78um17er9LDZgsWCbNY");
/** FACTS `EMERGENCY_PUBKEY` — pause and revoke only, never unpause, never withdraw. */
export const BOOK_ONE_EMERGENCY: Address = address("A67Gw8VZbYx6qEvFeDdq4eGzpz33L3BkqyhpFrCN7JxB");
/** FACTS `DEPLOYER_PUBKEY` — registry admin and demo_perps market authority on devnet. */
export const DEVNET_ADMIN: Address = address("8wuYJD6bZjSA115mwXgguPoUzSqEP3dc3GxBpCu4M3mn");

/** `markov_mandate::BOOK_ONE`: `b"BOOK_ONE"` padded to 16 bytes. */
export const BOOK_ONE_STRATEGY_ID = "BOOK_ONE";
/** The one market the Gate B book trades on the mock venue (`gate_b_setup.rs`). */
export const SOL_PERP_MARKET_ID = "SOL-PERP";

/** FACTS `RPC endpoints (keyless, 2026-09-02)`; the Terminal may override with `VITE_RPC_URL`. */
export const DEVNET_RPC_PRIMARY = "https://rpc.magicblock.app/devnet";
export const DEVNET_RPC_FALLBACK = "https://api.devnet.solana.com";
export const DEVNET_RPC_WS = "wss://api.devnet.solana.com";

/** FACTS `GATE_B_POLICY`, in mint base units (6 decimals). */
export const GATE_B_POLICY = {
  perTxCap: 50_000_000n,
  dailyCap: 200_000_000n,
  spendPerCall: 1_000_000n,
  spendDaily: 5_000_000n,
  maxSlippageBps: 50,
  maxMarkAgeSecs: 150n,
  /** Off-chain in v0 (ADR-005): the guard enforces these, the program does not. */
  offchain: { deltaBand: 20_000_000n, maxGross: 100_000_000n, dailyLossBps: 500 },
} as const;

/** FACTS `DEVNET_SLOT_MS` ≈ 165 ms; display only, never a freshness rule. */
export const DEVNET_SLOT_MS = 165;

export const CLUSTER = "devnet" as const;
export const SOLANA_CHAIN = "solana:devnet" as const;
