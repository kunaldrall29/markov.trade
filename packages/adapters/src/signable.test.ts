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
