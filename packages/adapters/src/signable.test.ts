import { test } from "node:test";
import assert from "node:assert/strict";
import { buildPacificaSignable } from "./pacifica.ts";
import { slippageBpsAtNotional } from "./types.ts";

test("Pacifica compact JSON matches the documented example shape", () => {
  const { compactJson } = buildPacificaSignable({
    type: "create_order",
    account: "6ETnufiec2CxVWTS4u5Wiq33Zh5Y3Qm6Pkdpi375fuxP",
    timestamp: 1748970123456,
    expiry_window: 5000,
    data: {
      symbol: "BTC",
      price: "100000",
      amount: "0.1",
      side: "bid",
      tif: "GTC",
      reduce_only: false,
      client_order_id: "12345678-1234-1234-1234-123456789abc",
    },
  });
  assert.ok(compactJson.startsWith("{"));
  assert.ok(compactJson.includes('"type":"create_order"'));
  assert.ok(!compactJson.includes(" "));
  const parsed = JSON.parse(compactJson);
  assert.deepEqual(Object.keys(parsed), ["data", "expiry_window", "timestamp", "type"]);
});

test("slippage is null when the book cannot fill", () => {
  const bps = slippageBpsAtNotional([{ price: 100, size: 0.01 }], 10_000);
  assert.equal(bps, null);
});

test("cancel_order compact JSON is typed cancel_order with client_order_id", async () => {
  const { buildPacificaCancelOrder } = await import("./pacifica.ts");
  const o = buildPacificaCancelOrder({
    account: "6ETnufiec2CxVWTS4u5Wiq33Zh5Y3Qm6Pkdpi375fuxP",
    symbol: "SOL",
    clientOrderId: "12345678-1234-1234-1234-123456789abc",
    timestamp: 1748970123456,
    expiryWindow: 5000,
  });
  const parsed = JSON.parse(o.compactJson);
  assert.equal(parsed.type, "cancel_order");
  assert.equal(parsed.data.symbol, "SOL");
  assert.equal(parsed.data.client_order_id, "12345678-1234-1234-1234-123456789abc");
  assert.deepEqual(Object.keys(parsed), ["data", "expiry_window", "timestamp", "type"]);
});

test("create_order amount is base size rounded to lot, not USD", async () => {
  const { buildPacificaCreateOrder } = await import("./pacifica.ts");
  const o = buildPacificaCreateOrder({
    account: "6ETnufiec2CxVWTS4u5Wiq33Zh5Y3Qm6Pkdpi375fuxP",
    symbol: "SOL",
    side: "long",
    notionalUsd: 50,
    mark: 100,
    tickSize: 0.01,
    lotSize: 0.01,
    bestBid: 99.99,
    bestAsk: 100.01,
    clientOrderId: "12345678-1234-1234-1234-123456789abc",
    timestamp: 1748970123456,
    expiryWindow: 5000,
  });
  assert.equal(o.fields.amount, "0.50");
  assert.equal(o.fields.side, "bid");
  assert.equal(o.fields.price, "100.01");
  const parsed = JSON.parse(o.compactJson);
  assert.equal(parsed.data.amount, "0.50");
  assert.deepEqual(Object.keys(parsed), ["data", "expiry_window", "timestamp", "type"]);
});
