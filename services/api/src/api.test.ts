process.env.MARKOV_DATA_DIR = "memory";

import { test } from "node:test";
import assert from "node:assert/strict";
import nacl from "tweetnacl";
import bs58 from "bs58";
import { createCache, policyPreview, routesFor, type Cache } from "./engine.ts";
import { loadEnv } from "@markov/config";
import { buildServer } from "./index.ts";
import type { MarketState } from "@markov/adapters";

function freshPacifica(cache: Cache, overrides: Partial<MarketState> = {}) {
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
    bids: [{ price: 99.99, size: 50 }],
    asks: [{ price: 100.01, size: 50 }],
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
    ...overrides,
  });
}

async function siws(app: { inject: (opts: object) => Promise<{ statusCode: number; json: () => unknown }> }) {
  const kp = nacl.sign.keyPair();
  const pubkey = bs58.encode(kp.publicKey);
  const ch = await app.inject({ method: "POST", url: "/auth/challenge", payload: { pubkey } });
  assert.equal(ch.statusCode, 200);
  const { nonce, message } = ch.json() as { nonce: string; message: string };
  const signature = bs58.encode(nacl.sign.detached(new TextEncoder().encode(message), kp.secretKey));
  const v = await app.inject({ method: "POST", url: "/auth/verify", payload: { pubkey, signature, nonce } });
  assert.equal(v.statusCode, 200);
  const { token } = v.json() as { token: string };
  return { kp, pubkey, token, headers: { authorization: `Bearer ${token}` } };
}

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

test("SIWS round-trip issues a session; a bad signature does not", async () => {
  const { app } = await buildServer();
  const ok = await siws(app);
  const me = await app.inject({ method: "GET", url: "/auth/me", headers: ok.headers });
  assert.equal(me.statusCode, 200);
  assert.equal((me.json() as { pubkey: string }).pubkey, ok.pubkey);
  const kp = nacl.sign.keyPair();
  const pubkey = bs58.encode(kp.publicKey);
  const ch = await app.inject({ method: "POST", url: "/auth/challenge", payload: { pubkey } });
  const { nonce, message } = ch.json() as { nonce: string; message: string };
  const other = nacl.sign.keyPair();
  const bad = bs58.encode(nacl.sign.detached(new TextEncoder().encode(message), other.secretKey));
  const v = await app.inject({ method: "POST", url: "/auth/verify", payload: { pubkey, signature: bad, nonce } });
  assert.equal(v.statusCode, 401);
  await app.close();
});

test("owner-signed ALLOW request builds compact JSON; submit is a venue ack not a fill", async () => {
  const { app, cache } = await buildServer();
  freshPacifica(cache);
  const s = await siws(app);
  const req = await app.inject({
    method: "POST",
    url: "/trades/request",
    headers: s.headers,
    payload: { market: "SOL-PERP", side: "long", notional_usd: 50, leverage: 1.5 },
  });
  assert.equal(req.statusCode, 200);
  const body = req.json() as {
    decision: string;
    request_id: string;
    signables: Array<{ compact_json: string; fields: { reduce_only: boolean; amount: string } }>;
  };
  assert.equal(body.decision, "REQUIRE_APPROVAL");
  assert.ok(body.signables[0]?.compact_json);
  assert.equal(body.signables[0].fields.reduce_only, false);
  assert.equal(body.signables[0].fields.amount, "0.50");
  const compact = body.signables[0].compact_json;
  const signature = bs58.encode(nacl.sign.detached(new TextEncoder().encode(compact), s.kp.secretKey));
  const sub = await app.inject({
    method: "POST",
    url: "/trades/submit",
    headers: s.headers,
    payload: { request_id: body.request_id, signature },
  });
  assert.equal(sub.statusCode, 200);
  const out = sub.json() as { ok: boolean; receipt: { reason: string }; venue_status: number };
  assert.equal(out.ok, false);
  assert.equal(out.receipt.reason, "EXECUTION_FAILED");
  assert.ok(typeof out.venue_status === "number");
  await app.close();
});

test("reduce requires SIWS and sets reduce_only", async () => {
  const { app, cache } = await buildServer();
  const denied = await app.inject({
    method: "POST",
    url: "/trades/reduce",
    payload: { market: "SOL-PERP", side: "short", notional_usd: 50 },
  });
  assert.equal(denied.statusCode, 401);
  freshPacifica(cache);
  const s = await siws(app);
  const res = await app.inject({
    method: "POST",
    url: "/trades/reduce",
    headers: s.headers,
    payload: { market: "SOL-PERP", side: "short", notional_usd: 50 },
  });
  assert.equal(res.statusCode, 200);
  const body = res.json() as { signables: Array<{ fields: { reduce_only: boolean } }> };
  assert.equal(body.signables[0].fields.reduce_only, true);
  await app.close();
});

test("risk and portfolio leave leverage null when equity is unknown", async () => {
  const { app } = await buildServer();
  const risk = await app.inject({ method: "GET", url: "/risk" });
  const port = await app.inject({ method: "GET", url: "/portfolio" });
  assert.equal(risk.statusCode, 200);
  assert.equal((risk.json() as { effective_leverage: null }).effective_leverage, null);
  assert.equal((port.json() as { equity: null }).equity, null);
  await app.close();
});
