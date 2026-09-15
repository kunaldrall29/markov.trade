/**
 * The read API's wire shapes (docs/12 §3 rules: every money field carries
 * `raw`, `decimals`, `mint`; every derived field says `source`; bigints are
 * strings; there is no `apy`, `apr` or `projected_*` field anywhere).
 */
export type Money = { raw: string; decimals: number; mint: string };

export type ReceiptRow = {
  signature: string;
  eventIndex: number;
  slot: string;
  blockTime: number | null;
  txError: boolean;
  kind: "action" | "refusal" | "owner";
  mandate: string;
  actor: string;
  seq: string | null;
  action: string | null;
  side: string | null;
  market: string | null;
  notional: Money | null;
  fillPrice: string | null;
  fee: string | null;
  markPrice: string | null;
  markPublishTime: number | null;
  reason: string | null;
  reasonByte: number | null;
  gateIndex: number | null;
  forced: boolean | null;
  ownerKind: string | null;
  amount: Money | null;
  explorer: string;
};

export type Envelope = { env: "devnet"; source: "chain"; data_slot: string; fetched_at: number };

export type ReceiptsResponse = Envelope & {
  receipts: ReceiptRow[];
  before: string | null;
  signatures: number;
  address: string;
};

export type Circuit = "live" | "paused" | "revoked" | "expired" | "global_halt" | "stale_mark";

export type PolicyView = {
  venues: string[];
  tokens: string[];
  allowed_actions: string[];
  per_tx_cap: Money;
  daily_cap: Money;
  spend_per_call: Money;
  spend_daily: Money;
  max_slippage_bps: number;
  max_mark_age_secs: string;
  expiry_ts: string;
};

export type MandateSummary = {
  address: string;
  owner: string;
  operator: string;
  emergency: string;
  strategy: string;
  state: "Active" | "Paused" | "Revoked";
  nonce: string;
  created_at: string;
  action_seq: string;
  mint: string;
  mark_account: string;
  vault: Money & { address: string; exists: boolean };
  policy: PolicyView;
  day: { epoch: string; notional_used: Money; spend_used: Money };
  /** Literal `true`: `owner_withdraw` has no state check in the program (docs/13 §6). */
  withdraw_enabled: true;
};

export type BookStats = Envelope & {
  mandate: MandateSummary;
  position: {
    address: string;
    side: "long" | "short";
    notional: Money;
    entry_price_e6: string;
    funding_accrued: string;
    updated_slot: string;
  } | null;
  mark: {
    venue: { price_e6: string | null; source: "pyth" | "house"; slot: string; publish_time: string } | null;
    pyth: { price_e6: string | null; publish_time: string; posted_slot: string; verification: "Full" | "Partial"; age_secs: number } | null;
  };
  registry: { global_halt: boolean; adapters: string[] } | null;
  circuit: Circuit;
  window: { hours: 24; refusals: number; actions: number; owner_actions: number; truncated: boolean; signatures_walked: number };
  /** ADR-005: the delta band, gross ceiling and daily-loss halt are enforced by the guard, not the program. */
  enforcement: { delta: "offchain"; gross: "offchain"; daily_loss: "offchain" };
  offchain_limits: { delta_band: Money; max_gross: Money; daily_loss_bps: number };
};

export type Health = {
  ok: boolean;
  chainReady: boolean;
  failing: string[];
  env: "devnet";
  program: string;
  rpc: { host: string; slot: string | null; latency_ms: number | null; error: string | null };
  checked_at: number;
};

export type ApiError = { error: { code: string; message: string } };
