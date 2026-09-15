import { REASON, VENUE_BIT } from "@markov/facts";

export type DecisionKind = "ALLOW" | "REJECT" | "REQUIRE_APPROVAL" | "SKIP";

export type Check = {
  rule: number;
  observed: bigint;
  limit: bigint;
  pass: boolean;
  reason: number;
};

export type MandateView = {
  version: number;
  maxLeverageBps: number;
  maxNotionalUsd: bigint;
  minSafetyBufferBps: number;
  maxDailyLossUsd: bigint;
  approvedMarkets: string[];
  allowedVenues: number;
};

export type InvestMandateView = {
  version: number;
  allowlist: string[];
  monthlyCeilingUsd: bigint;
  spentThisMonthUsd: bigint;
  reserveFloorUsd: bigint;
  maxCostBps: number;
  singleAssetCapBps: number;
  executionMode: "AlwaysOn" | "ReferenceSafe" | "MarketHoursOnly";
  paused: boolean;
};

export type PermissionView = {
  actor: string;
  scopes: number;
  perActionCapUsd: bigint;
  dailyCapUsd: bigint;
  spentTodayUsd: bigint;
  revoked: boolean;
  expiresSlot: bigint;
};

export type AccountView = {
  owner: string;
  status: "Active" | "Paused" | "Closed";
  globalPaused: boolean;
};

export type MarketView = {
  venueId: number;
  marketId: string;
  dataAgeMs: number;
  freshnessLimitMs: number;
};

export type TradeAction = {
  kind: "TradeOpen" | "TradeReduce" | "TradeClose" | "InvestExecute";
  actor: string;
  isOwner: boolean;
  venueId: number;
  marketId: string;
  mint?: string;
  projectedLeverageBps: number;
  projectedNotionalUsd: bigint;
  safetyBufferBps: number;
  dailyLossUsd: bigint;
  slippageBps: number;
  slippageLimitBps: number;
  quoteCostBps?: number;
  reserveUsd?: bigint;
  marketOpen?: boolean;
  dataAgeMs: number;
  freshnessLimitMs: number;
  slot: bigint;
};

const SCOPE_TRADE_REQUEST = 1 << 2;
const SCOPE_TRADE_REDUCE = 1 << 3;
const SCOPE_INVEST_EXECUTE = 1 << 5;

function bit(venueId: number): number | null {
  if (venueId === 1) return VENUE_BIT.PACIFICA;
  if (venueId === 2) return VENUE_BIT.DRIFT;
  if (venueId === 3) return VENUE_BIT.PHOENIX;
  if (venueId === 4) return VENUE_BIT.JUPITER;
  return null;
}

function push(
  checks: Check[],
  rule: number,
  reason: number,
  observed: bigint,
  limit: bigint,
  pass: boolean,
): void {
  checks.push({ rule, reason, observed, limit, pass });
}

export type PolicyResult = {
  decision: DecisionKind;
  reason_code: number;
  checks: Check[];
};

export function check(input: {
  action: TradeAction;
  account: AccountView;
  mandate: MandateView | null;
  invest?: InvestMandateView | null;
  permission?: PermissionView | null;
}): PolicyResult {
  const { action, account, mandate, invest, permission } = input;
  const checks: Check[] = [];

  const hard = (reason: number, decision: DecisionKind = "REJECT"): PolicyResult => ({
    decision,
    reason_code: reason,
    checks,
  });

  if (account.globalPaused && action.kind !== "TradeReduce" && action.kind !== "TradeClose") {
    push(checks, 12, REASON.GLOBAL_PAUSED, 1n, 0n, false);
    return hard(REASON.GLOBAL_PAUSED);
  }
  push(checks, 12, REASON.GLOBAL_PAUSED, 0n, 0n, true);

  if (account.status !== "Active") {
    push(checks, 11, REASON.ACCOUNT_PAUSED, 1n, 0n, false);
    return hard(REASON.ACCOUNT_PAUSED);
  }
  push(checks, 11, REASON.ACCOUNT_PAUSED, 0n, 0n, true);

  if (!action.isOwner) {
    if (!permission || permission.revoked) {
      push(checks, 8, REASON.ACTOR_SCOPE_DENIED, 0n, 1n, false);
      return hard(REASON.ACTOR_SCOPE_DENIED);
    }
    const need =
      action.kind === "InvestExecute"
        ? SCOPE_INVEST_EXECUTE
        : action.kind === "TradeOpen"
          ? SCOPE_TRADE_REQUEST
          : SCOPE_TRADE_REDUCE;
    if ((permission.scopes & need) === 0) {
      push(checks, 8, REASON.ACTOR_SCOPE_DENIED, BigInt(permission.scopes), BigInt(need), false);
      return hard(REASON.ACTOR_SCOPE_DENIED);
    }
    if (action.projectedNotionalUsd > permission.perActionCapUsd) {
      push(checks, 9, REASON.ACTOR_CAP_EXCEEDED, action.projectedNotionalUsd, permission.perActionCapUsd, false);
      return hard(REASON.ACTOR_CAP_EXCEEDED);
    }
  }

  if (action.kind === "InvestExecute") {
    if (!invest) return hard(REASON.ASSET_NOT_ALLOWLISTED);
    if (invest.paused) return hard(REASON.ACCOUNT_PAUSED);
    if (!action.mint || !invest.allowlist.includes(action.mint)) {
      push(checks, 20, REASON.ASSET_NOT_ALLOWLISTED, 0n, 1n, false);
      return hard(REASON.ASSET_NOT_ALLOWLISTED);
    }
    push(checks, 20, REASON.ASSET_NOT_ALLOWLISTED, 1n, 1n, true);
    const next = invest.spentThisMonthUsd + action.projectedNotionalUsd;
    if (next > invest.monthlyCeilingUsd) {
      push(checks, 22, REASON.MONTHLY_CEILING_EXCEEDED, next, invest.monthlyCeilingUsd, false);
      return hard(REASON.MONTHLY_CEILING_EXCEEDED, "SKIP");
    }
    if (action.reserveUsd !== undefined && action.reserveUsd < invest.reserveFloorUsd) {
      push(checks, 23, REASON.RESERVE_FLOOR, action.reserveUsd, invest.reserveFloorUsd, false);
      return hard(REASON.RESERVE_FLOOR, "SKIP");
    }
    if (action.quoteCostBps !== undefined && action.quoteCostBps > invest.maxCostBps) {
      push(checks, 24, REASON.COST_LIMIT, BigInt(action.quoteCostBps), BigInt(invest.maxCostBps), false);
      return hard(REASON.COST_LIMIT, "SKIP");
    }
    if (invest.executionMode === "MarketHoursOnly" && action.marketOpen === false) {
      push(checks, 25, REASON.MARKET_CLOSED, 0n, 1n, false);
      return hard(REASON.MARKET_CLOSED, "SKIP");
    }
    if (action.dataAgeMs > action.freshnessLimitMs) {
      push(checks, 29, REASON.STALE_QUOTE, BigInt(action.dataAgeMs), BigInt(action.freshnessLimitMs), false);
      return hard(REASON.STALE_QUOTE, "SKIP");
    }
    return { decision: "ALLOW", reason_code: REASON.EXECUTION_CONFIRMED, checks };
  }

  if (!mandate) return hard(REASON.MARKET_NOT_ALLOWED);

  const vb = bit(action.venueId);
  const venueOk = vb !== null && (mandate.allowedVenues & vb) !== 0;
  push(checks, 10, REASON.VENUE_NOT_ALLOWED, BigInt(action.venueId), BigInt(mandate.allowedVenues), venueOk);
  if (!venueOk) return hard(REASON.VENUE_NOT_ALLOWED);

  const marketOk = mandate.approvedMarkets.includes(action.marketId);
  push(checks, 1, REASON.MARKET_NOT_ALLOWED, 1n, BigInt(mandate.approvedMarkets.length), marketOk);
  if (!marketOk) return hard(REASON.MARKET_NOT_ALLOWED);

  const fresh = action.dataAgeMs <= action.freshnessLimitMs;
  push(checks, 6, REASON.STALE_MARKET_DATA, BigInt(action.dataAgeMs), BigInt(action.freshnessLimitMs), fresh);
  if (!fresh) return hard(REASON.STALE_MARKET_DATA);

  const levOk = action.projectedLeverageBps <= mandate.maxLeverageBps;
  push(checks, 2, REASON.MAX_LEVERAGE_EXCEEDED, BigInt(action.projectedLeverageBps), BigInt(mandate.maxLeverageBps), levOk);
  if (!levOk) return hard(REASON.MAX_LEVERAGE_EXCEEDED);

  const notionalOk = action.projectedNotionalUsd <= mandate.maxNotionalUsd;
  push(checks, 3, REASON.MAX_NOTIONAL_EXCEEDED, action.projectedNotionalUsd, mandate.maxNotionalUsd, notionalOk);
  if (!notionalOk) return hard(REASON.MAX_NOTIONAL_EXCEEDED);

  const bufOk = action.safetyBufferBps >= mandate.minSafetyBufferBps;
  push(checks, 4, REASON.MIN_SAFETY_BUFFER, BigInt(action.safetyBufferBps), BigInt(mandate.minSafetyBufferBps), bufOk);
  if (!bufOk) return hard(REASON.MIN_SAFETY_BUFFER);

  const lossOk = action.dailyLossUsd <= mandate.maxDailyLossUsd;
  push(checks, 5, REASON.DAILY_LOSS_BUDGET_EXCEEDED, action.dailyLossUsd, mandate.maxDailyLossUsd, lossOk);
  if (!lossOk) return hard(REASON.DAILY_LOSS_BUDGET_EXCEEDED);

  const slipOk = action.slippageBps <= action.slippageLimitBps;
  push(checks, 7, REASON.SLIPPAGE_LIMIT, BigInt(action.slippageBps), BigInt(action.slippageLimitBps), slipOk);
  if (!slipOk) return hard(REASON.SLIPPAGE_LIMIT);

  return { decision: "ALLOW", reason_code: REASON.EXECUTION_CONFIRMED, checks };
}
