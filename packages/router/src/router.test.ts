import { test } from "node:test";
import assert from "node:assert/strict";
import { compareRoutes } from "./index.ts";

test("Phoenix read-only is never selected", () => {
  const r = compareRoutes(
    [
      {
        venueId: 1,
        venue: "pacifica",
        executable: true,
        linked: true,
        stale: false,
        mark: 100,
        slippageBps: 4,
        takerFeeBps: 4,
        fundingBpsPerDay: 1,
        exitBps: 4,
        riskPremiumBps: 0,
        freshnessMs: 400,
      },
      {
        venueId: 3,
        venue: "phoenix",
        executable: false,
        linked: false,
        stale: false,
        mark: 100.4,
        slippageBps: 1,
        takerFeeBps: 3.5,
        fundingBpsPerDay: 0.5,
        exitBps: 1,
        riskPremiumBps: 0,
        freshnessMs: 200,
      },
    ],
    72,
  );
  assert.equal(r.find((x) => x.venue === "pacifica")?.selected, true);
  assert.equal(r.find((x) => x.venue === "phoenix")?.selected, false);
});

test("stale venue is not selected even if cheaper", () => {
  const r = compareRoutes(
    [
      {
        venueId: 1,
        venue: "pacifica",
        executable: true,
        linked: true,
        stale: true,
        mark: 100,
        slippageBps: 1,
        takerFeeBps: 1,
        fundingBpsPerDay: 0,
        exitBps: 1,
        riskPremiumBps: 0,
        freshnessMs: 20_000,
      },
      {
        venueId: 2,
        venue: "drift",
        executable: true,
        linked: true,
        stale: false,
        mark: 100,
        slippageBps: 8,
        takerFeeBps: 5,
        fundingBpsPerDay: 2,
        exitBps: 8,
        riskPremiumBps: 1,
        freshnessMs: 200,
      },
    ],
    24,
  );
  assert.equal(r.find((x) => x.venue === "drift")?.selected, true);
});
