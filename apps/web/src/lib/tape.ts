export type TapeRow = {
  t: string;
  action: string;
  qty: string;
  result: "allowed" | "blocked" | "skip";
  reason?: string;
};

export const SAMPLE_TAPE: TapeRow[] = [
  { t: "14:02", action: "hedge SOL-PERP", qty: "+12.0", result: "allowed" },
  { t: "14:04", action: "increase SOL", qty: "+40.0", result: "blocked", reason: "OverTxCap" },
  { t: "14:11", action: "skip", qty: "—", result: "skip" },
  { t: "14:18", action: "flatten", qty: "0.0", result: "allowed" },
  { t: "14:26", action: "hedge SOL-PERP", qty: "+8.5", result: "allowed" },
];

export const BOOK_STATS = {
  netDelta: "$12.40",
  netBand: "±20",
  gross: "$61",
  cap: "cap 100",
  funding7d: "0",
  fundingUnit: "USDC-d",
  refusals: "3",
  refusalsWindow: "24h",
};

export function rowLabel(row: TapeRow) {
  return row.result === "blocked" ? (row.reason ?? "blocked") : row.result;
}

export const OWNER_VERBS = [
  {
    id: "fund" as const,
    label: "Fund",
    body: "USDC-d in the mandate PDA — never the operator.",
  },
  {
    id: "pause" as const,
    label: "Pause",
    body: "Owner-only unpause.",
  },
  {
    id: "revoke" as const,
    label: "Revoke",
    body: "Next intent refused. Receipt says Revoked.",
  },
  {
    id: "withdraw" as const,
    label: "Withdraw",
    body: "On in every state. Coins to you.",
  },
];

export const PAPER_LOG = [
  {
    date: "31 Aug 2026",
    title: "Book One is the only object we ship first.",
    body: "Hosted on devnet. demo_perps behind the same adapter trait as a future real venue. Marks are marked PnL, not a promised rate. Unaudited.",
  },
  {
    date: "30 Aug 2026",
    title: "A refusal is the system working.",
    body: "OverTxCap wrote a RefusalReceipt. The interesting row is the one that says no. Withdraw stayed on.",
  },
  {
    date: "28 Aug 2026",
    title: "Skip is the default.",
    body: "Circuit live. Inventory inside band. No fill. A skip is recorded, not hidden.",
  },
  {
    date: "22 Aug 2026",
    title: "Owner keeps the pile.",
    body: "Fund lands in the mandate PDA. The operator cannot withdraw. owner_withdraw works in Active, Paused, Revoked, Expired.",
  },
];
