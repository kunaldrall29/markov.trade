process.env.MARKOV_DATA_DIR = "memory";

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
    tickSize: null,
    lotSize: null,
    minOrderSize: null,
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
    tickSize: 0.01,
    lotSize: 0.01,
    minOrderSize: 10,
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
    tickSize: 0.01,
    lotSize: 0.01,
    minOrderSize: 10,
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

test("candles refuse to invent a Phoenix series", async () => {
  const { app } = await buildServer();
  const res = await app.inject({ method: "GET", url: "/markets/SOL-PERP/candles?venue=phoenix" });
  assert.equal(res.statusCode, 200);
  const body = res.json() as { candles: unknown[]; note: string };
  assert.equal(body.candles.length, 0);
  assert.match(body.note, /not invented/i);
  await app.close();
});

test("invest propose writes a receipt even without a session", async () => {
  const { app } = await buildServer();
  const res = await app.inject({
    method: "POST",
    url: "/invest/propose",
    payload: { mint: "Xsc9qvGR1efVDFGLrVsmkzv3qi45LTBjeUKSPmx9qEh", usd_per_period: 5, period_seconds: 604800 },
  });
  assert.equal(res.statusCode, 200);
  const body = res.json() as { decision: string; signable_for_model: boolean; receipt_id: string };
  assert.equal(body.decision, "REQUIRE_APPROVAL");
  assert.equal(body.signable_for_model, false);
  const receipts = await app.inject({ method: "GET", url: "/receipts" });
  assert.equal((receipts.json() as { receipts: unknown[] }).receipts.length >= 1, true);
  await app.close();
});

test("idempotent trade reject replays the same request_id", async () => {
  const { app, cache } = await buildServer();
  cache.pacifica.set("SOL-PERP", {
    venue: "pacifica",
    symbol: "SOL",
    mark: 100,
    index: 100,
    funding: 0,
    nextFunding: 0,
    openInterest: 1,
    volume24h: 1,
    change24h: 0,
    bids: [{ price: 100, size: 50 }],
    asks: [{ price: 100.1, size: 50 }],
    venueTs: Date.now(),
    fetchedAt: Date.now(),
    slot: null,
    takerFeeBps: 4,
    makerFeeBps: 1.5,
    maxLeverage: 50,
    isolatedOnly: false,
    tickSize: 0.01,
    lotSize: 0.01,
    minOrderSize: 10,
  });
  const payload = { market: "SOL-PERP", side: "long", notional_usd: 50, leverage: 3 };
  const a = await app.inject({ method: "POST", url: "/trades/request", headers: { "idempotency-key": "abc" }, payload });
  const b = await app.inject({ method: "POST", url: "/trades/request", headers: { "idempotency-key": "abc" }, payload });
  assert.equal(a.statusCode, 200);
  assert.equal((a.json() as { request_id: string }).request_id, (b.json() as { request_id: string }).request_id);
  await app.close();
});
