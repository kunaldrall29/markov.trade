import { test } from "node:test";
import assert from "node:assert/strict";
import { normalize, simulateTrade } from "./index.ts";

test("missing equity yields null leverage — fail closed, not zero", () => {
  const s = normalize(
    [
      {
        venueId: 1,
        equityUsd: null,
        grossNotionalUsd: 50,
        maintenanceMarginUsd: null,
        dailyPnlUsd: 0,
        health: null,
        positions: [],
        dataSlot: 1,
        venueTs: Date.now(),
      },
    ],
    { maxLeverageBps: 20_000, maxNotionalUsd: 2000, maxDailyLossUsd: 50 },
    Date.now(),
    3000,
  );
  assert.equal(s.effectiveLeverage, null);
  assert.equal(s.safetyBufferBps, null);
});

test("simulateTrade increases gross", () => {
  const s = normalize(
    [
      {
        venueId: 1,
        equityUsd: 100,
        grossNotionalUsd: 50,
        maintenanceMarginUsd: 5,
        dailyPnlUsd: 0,
        health: 1,
        positions: [],
        dataSlot: 9,
        venueTs: Date.now(),
      },
    ],
    { maxLeverageBps: 20_000, maxNotionalUsd: 2000, maxDailyLossUsd: 50 },
    Date.now(),
    3000,
  );
  const p = simulateTrade(s, 50, 0);
  assert.equal(p.grossNotionalUsd, 100);
  assert.ok(p.effectiveLeverage && p.effectiveLeverage > 0.9);
});
