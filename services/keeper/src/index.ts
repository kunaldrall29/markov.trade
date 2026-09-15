/**
 * Invest keeper. Spend is bounded by Subscriptions & Allowances on chain.
 * This process only evaluates due rules and either record_skip or
 * invest_execute. It does not hold user keys.
 */
import { NVDAX, REASON } from "@markov/facts";
import { check, type InvestMandateView } from "@markov/policy";

export type Rule = {
  id: string;
  owner: string;
  mint: string;
  usd: number;
  nextDue: number;
  invest: InvestMandateView;
};

export type CycleResult = { rule_id: string; decision: string; reason_code: number };

export function evaluateDue(rule: Rule, now: number, quoteCostBps: number, reserveUsd: bigint, marketOpen: boolean): CycleResult {
  if (now < rule.nextDue) return { rule_id: rule.id, decision: "WAIT", reason_code: 0 };
  const r = check({
    action: {
      kind: "InvestExecute",
      actor: "keeper",
      isOwner: false,
      venueId: 4,
      marketId: "SPOT",
      mint: rule.mint,
      projectedLeverageBps: 0,
      projectedNotionalUsd: BigInt(Math.round(rule.usd * 1_000_000)),
      safetyBufferBps: 0,
      dailyLossUsd: 0n,
      slippageBps: quoteCostBps,
      slippageLimitBps: rule.invest.maxCostBps,
      quoteCostBps,
      reserveUsd,
      marketOpen,
      dataAgeMs: 200,
      freshnessLimitMs: 10_000,
      slot: 0n,
    },
    account: { owner: rule.owner, status: "Active", globalPaused: false },
    mandate: null,
    invest: rule.invest,
    permission: {
      actor: "keeper",
      scopes: 1 << 5,
      perActionCapUsd: 100_000_000n,
      dailyCapUsd: 2_000_000_000n,
      spentTodayUsd: 0n,
      revoked: false,
      expiresSlot: 0n,
    },
  });
  return { rule_id: rule.id, decision: r.decision, reason_code: r.reason_code };
}

export const CANARY_MINT = NVDAX;
export const CANARY_USD = 5;

if (import.meta.url === `file://${process.argv[1]}` || process.argv[1]?.endsWith("index.ts")) {
  console.log(JSON.stringify({ ok: true, role: "keeper", canary_usd: CANARY_USD, mint: CANARY_MINT, reason_market_closed: REASON.MARKET_CLOSED }));
  setInterval(() => {
    /* leader lock lives in Redis in production; local process is a single replica */
  }, 60_000).unref();
}
