import { mkdirSync, readFileSync, writeFileSync, existsSync } from "node:fs";
import { dirname, join } from "node:path";
import type { Receipt } from "@markov/sdk";
import type { PacificaCancelFields, PacificaOrderFields } from "@markov/adapters";

export type Proposal = {
  id: string;
  kind: "invest_rule" | "mandate" | "trade";
  actor: string;
  payload: Record<string, unknown>;
  status: "pending" | "signed" | "declined";
  created_at: string;
  receipt_id: string;
};

export type PendingTrade = {
  request_id: string;
  status: "REQUESTED" | "AWAITING_SIGNATURE" | "SUBMITTED" | "FAILED" | "REJECTED";
  actor: string;
  market: string;
  op?: "create_order" | "cancel_order";
  compact_json: string | null;
  timestamp: number | null;
  expiry_window: number | null;
  fields: PacificaOrderFields | PacificaCancelFields | null;
  venue_response?: unknown;
  created_at: string;
};

export type VenueLink = {
  venue: "pacifica";
  owner: string;
  linked_at: string;
  venue_account_found: boolean | null;
};

export type MandateDraft = {
  owner: string;
  version: number;
  max_leverage_bps: number;
  max_notional_usd: number;
  min_safety_buffer_bps: number;
  max_daily_loss_usd: number;
  approved_markets: string[];
  updated_at: string;
  on_chain: false;
};

export type Disk = {
  receipts: Receipt[];
  proposals: Proposal[];
  pending: PendingTrade[];
  links: VenueLink[];
  mandates: MandateDraft[];
  investRules: Array<Record<string, unknown>>;
};

export function persistEnabled(): boolean {
  return process.env.MARKOV_DATA_DIR !== "memory";
}

export function dataFile(): string {
  const dir = process.env.MARKOV_DATA_DIR || join(process.cwd(), "data");
  return join(dir, "control-plane.json");
}

export function loadDisk(): Disk {
  const empty: Disk = { receipts: [], proposals: [], pending: [], links: [], mandates: [], investRules: [] };
  if (!persistEnabled()) return empty;
  const p = dataFile();
  if (!existsSync(p)) return empty;
  try {
    return { ...empty, ...JSON.parse(readFileSync(p, "utf8")) };
  } catch {
    return empty;
  }
}

export function persist(cache: {
  receipts: Receipt[];
  proposals: Proposal[];
  pending: PendingTrade[];
  links: VenueLink[];
  mandates: MandateDraft[];
  investRules: Array<Record<string, unknown>>;
}): void {
  if (!persistEnabled()) return;
  const p = dataFile();
  mkdirSync(dirname(p), { recursive: true });
  const disk: Disk = {
    receipts: cache.receipts.slice(0, 500),
    proposals: cache.proposals.slice(0, 200),
    pending: cache.pending.slice(0, 200),
    links: cache.links,
    mandates: cache.mandates,
    investRules: cache.investRules,
  };
  writeFileSync(p, JSON.stringify(disk, null, 2));
}
