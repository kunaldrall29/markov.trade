import { test } from "node:test";
import assert from "node:assert/strict";
import { NVDAX, REASON } from "@markov/facts";
import { evaluateDue } from "./index.ts";

const invest = {
  version: 1,
  allowlist: [NVDAX],
  monthlyCeilingUsd: 40_000_000n,
  spentThisMonthUsd: 0n,
  reserveFloorUsd: 50_000_000n,
  maxCostBps: 40,
  singleAssetCapBps: 10_000,
  executionMode: "MarketHoursOnly" as const,
  paused: false,
};

test("outside market hours skips", () => {
  const r = evaluateDue(
    { id: "r1", owner: "o", mint: NVDAX, usd: 5, nextDue: 0, invest },
    Date.now(),
    12,
    80_000_000n,
    false,
  );
  assert.equal(r.decision, "SKIP");
  assert.equal(r.reason_code, REASON.MARKET_CLOSED);
});

test("cost limit skips", () => {
  const r = evaluateDue(
    { id: "r1", owner: "o", mint: NVDAX, usd: 5, nextDue: 0, invest },
    Date.now(),
    90,
    80_000_000n,
    true,
  );
  assert.equal(r.reason_code, REASON.COST_LIMIT);
});
