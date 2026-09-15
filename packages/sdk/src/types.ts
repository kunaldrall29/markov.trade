export type EnvTag = "devnet" | "rehearsal" | "mainnet";

export type CheckRow = {
  rule: number;
  observed: string;
  limit: string;
  pass: boolean;
};

export type Receipt = {
  request_id: string;
  actor: string;
  kind: string;
  decision: "ALLOW" | "REJECT" | "REQUIRE_APPROVAL" | "SKIP";
  reason_code: number;
  reason: string;
  mandate_version: number;
  invest_mandate_version: number;
  data_slot: number;
  venue_id: number;
  market_id: string;
  checks: CheckRow[];
  route_snapshot_hash?: string;
  tx_signature?: string | null;
  explorer_url?: string | null;
  env: EnvTag;
  created_at: string;
};

export type MarketStateDto = {
  canonical_market_id: string;
  venue: string;
  venue_id: number;
  symbol: string;
  mark: number | null;
  index: number | null;
  funding: number | null;
  next_funding: number | null;
  change_24h: number | null;
  open_interest: number | null;
  depth_10bps: number | null;
  depth_50bps: number | null;
  spread_bps: number | null;
  max_leverage: number | null;
  isolated_only: boolean | null;
  taker_fee_bps: number | null;
  maker_fee_bps: number | null;
  health: string;
  freshness_ms: number;
  stale: boolean;
  executable: boolean;
  env: EnvTag;
  venue_ts: number;
  data_slot: number | null;
  bids?: Array<{ price: number; size: number }>;
  asks?: Array<{ price: number; size: number }>;
  tick_size?: number | null;
  lot_size?: number | null;
  min_order_size?: number | null;
};

export type VenueCapabilities = {
  id: string;
  env: string;
  execution_model: string;
  order_types: string[];
  margin_modes: string[];
  delegation_model: string;
  onchain_enforceable: boolean;
  executable: boolean;
  freshness_ms: number;
};

export type PolicyPreview = {
  decision: "ALLOW" | "REJECT" | "REQUIRE_APPROVAL" | "SKIP";
  reason_code: number;
  reason: string;
  checks: CheckRow[];
  data_slot: number;
  env: EnvTag;
};

export type Signable = {
  kind: "solana_tx" | "pacifica_message";
  display: string;
  bytes_b64: string;
  program_ids?: string[];
};

export type TradeRequest = {
  request_id: string;
  market: string;
  side: "long" | "short";
  notional_usd: number;
  leverage: number;
  venue: "AUTO" | "pacifica" | "drift" | "phoenix";
  horizon_hours: number;
  max_slippage_bps: number;
};
