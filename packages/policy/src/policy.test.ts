import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { REASON, VENUE_BIT } from "@markov/facts";
import { check, type MandateView, type TradeAction, type AccountView } from "./index.ts";

const fixture = JSON.parse(
  readFileSync(resolve(dirname(fileURLToPath(import.meta.url)), "fixtures/pacifica-sol-2026-09-15.json"), "utf8"),
);

const account: AccountView = { owner: "owner", status: "Active", globalPaused: false };
const mandate: MandateView = {
  version: 1,
  maxLeverageBps: 20_000,
  maxNotionalUsd: 500_000_000n,
  minSafetyBufferBps: 2_000,
  maxDailyLossUsd: 50_000_000n,
  approvedMarkets: ["SOL-PERP"],
  allowedVenues: VENUE_BIT.PACIFICA | VENUE_BIT.DRIFT,
};

function base(over: Partial<TradeAction> = {}): TradeAction {
  return {
    kind: "TradeOpen",
    actor: "owner",
    isOwner: true,
    venueId: 1,
    marketId: "SOL-PERP",
    projectedLeverageBps: 15_000,
    projectedNotionalUsd: 50_000_000n,
    safetyBufferBps: 5_000,
    dailyLossUsd: 0n,
    slippageBps: 8,
    slippageLimitBps: 30,
    dataAgeMs: 400,
    freshnessLimitMs: fixture.freshness_limit_ms,
    slot: BigInt(fixture.venue_ts),
    ...over,
  };
}

test("fixture is labelled with the live Pacifica timestamp", () => {
  assert.equal(fixture.source, "https://api.pacifica.fi/api/v1/info/prices");
  assert.equal(fixture.symbol, "SOL");
  assert.ok(Number(fixture.mark) > 0);
});

test("inside mandate allows", () => {
  const r = check({ action: base(), account, mandate });
  assert.equal(r.decision, "ALLOW");
  assert.equal(r.reason_code, REASON.EXECUTION_CONFIRMED);
  assert.ok(r.checks.every((c) => c.pass));
});

test("$5,000 at 3x is MAX_LEVERAGE_EXCEEDED", () => {
  const r = check({
    action: base({ projectedLeverageBps: 30_000, projectedNotionalUsd: 5_000_000_000n }),
    account,
    mandate,
  });
  assert.equal(r.decision, "REJECT");
  assert.equal(r.reason_code, REASON.MAX_LEVERAGE_EXCEEDED);
});

test("stale fails closed", () => {
  const r = check({ action: base({ dataAgeMs: 10_000 }), account, mandate });
  assert.equal(r.reason_code, REASON.STALE_MARKET_DATA);
});

test("Phoenix rejected until bitmap includes it", () => {
  const r = check({ action: base({ venueId: 3 }), account, mandate });
  assert.equal(r.reason_code, REASON.VENUE_NOT_ALLOWED);
});

test("paused account rejects opens", () => {
  const r = check({
    action: base(),
    account: { ...account, status: "Paused" },
    mandate,
  });
  assert.equal(r.reason_code, REASON.ACCOUNT_PAUSED);
});

test("invest skip on market closed", () => {
  const r = check({
    action: base({
      kind: "InvestExecute",
      mint: "Xsc9qvGR1efVDFGLrVsmkzv3qi45LTBjeUKSPmx9qEh",
      projectedNotionalUsd: 5_000_000n,
      quoteCostBps: 12,
      reserveUsd: 80_000_000n,
      marketOpen: false,
    }),
    account,
    mandate,
    invest: {
      version: 1,
      allowlist: ["Xsc9qvGR1efVDFGLrVsmkzv3qi45LTBjeUKSPmx9qEh"],
      monthlyCeilingUsd: 40_000_000n,
      spentThisMonthUsd: 0n,
      reserveFloorUsd: 50_000_000n,
      maxCostBps: 40,
      singleAssetCapBps: 10_000,
      executionMode: "MarketHoursOnly",
      paused: false,
    },
  });
  assert.equal(r.decision, "SKIP");
  assert.equal(r.reason_code, REASON.MARKET_CLOSED);
});
