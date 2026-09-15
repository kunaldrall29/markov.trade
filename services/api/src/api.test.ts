import { test } from "node:test";
import assert from "node:assert/strict";
import { createCache, policyPreview, routesFor } from "./engine.ts";
import { loadEnv } from "@markov/config";
import { buildServer } from "./index.ts";

test("Phoenix pin stays rejected for execution even with a fresh cache", () => {
  const cache = createCache(loadEnv("devnet"));
  cache.phoenix.set("SOL-PERP", {
    venue: "phoenix",
    symbol: "SOL",
    mark: 100,
    index: 100,
    funding: 0,
    nextFunding: 0,
    openInterest: 1,
    volume24h: 1,
    change24h: 0,
    bids: [{ price: 99.9, size: 10 }],
    asks: [{ price: 100.1, size: 10 }],
    venueTs: Date.now(),
    fetchedAt: Date.now(),
    slot: 1,
    takerFeeBps: 3.5,
    makerFeeBps: 0.5,
    maxLeverage: 10,
    isolatedOnly: false,
  });
  const r = policyPreview(cache, { market: "SOL-PERP", venue: "phoenix", notional_usd: 50, leverage: 1.5 });
  assert.equal(r.decision, "REJECT");
});

test("leverage above 2x rejects", () => {
  const cache = createCache(loadEnv("devnet"));
  cache.pacifica.set("SOL-PERP", {
    venue: "pacifica",
    symbol: "SOL",
    mark: 100.324,
    index: 100.3,
    funding: 0,
    nextFunding: 0,
    openInterest: 1,
    volume24h: 1,
    change24h: 0,
    bids: [{ price: 100.32, size: 50 }],
    asks: [{ price: 100.33, size: 50 }],
    venueTs: Date.now(),
    fetchedAt: Date.now(),
    slot: null,
    takerFeeBps: 4,
    makerFeeBps: 1.5,
    maxLeverage: 50,
    isolatedOnly: false,
  });
  const r = policyPreview(cache, { market: "SOL-PERP", notional_usd: 50, leverage: 3 });
  assert.equal(r.reason_code, 2);
});

test("routes never invent a Drift mark from Pacifica or Phoenix", () => {
  const cache = createCache(loadEnv("devnet"));
  cache.pacifica.set("SOL-PERP", {
    venue: "pacifica",
    symbol: "SOL",
    mark: 100.324,
    index: 100.3,
    funding: 0,
    nextFunding: 0,
    openInterest: 1,
    volume24h: 1,
    change24h: 0,
    bids: [{ price: 100.32, size: 50 }],
    asks: [{ price: 100.33, size: 50 }],
    venueTs: Date.now(),
    fetchedAt: Date.now(),
    slot: null,
    takerFeeBps: 4,
    makerFeeBps: 1.5,
    maxLeverage: 50,
    isolatedOnly: false,
  });
  const r = routesFor(cache, "SOL-PERP", 50, 24);
  assert.equal(r.routes.some((x) => x.venue === "drift"), false);
  assert.equal(r.routes.find((x) => x.venue === "pacifica")?.mark, 100.324);
});

test("health injects without hanging the interval", async () => {
  const { app } = await buildServer();
  const res = await app.inject({ method: "GET", url: "/health" });
  assert.equal(res.statusCode, 200);
  const body = res.json() as { ok: boolean; stage: string };
  assert.equal(body.ok, true);
  assert.equal(body.stage, "TEST STAGE");
  await app.close();
});
