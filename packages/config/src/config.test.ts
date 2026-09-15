import { test } from "node:test";
import assert from "node:assert/strict";
import { loadEnv } from "./index.ts";

test("devnet is test stage and uses Pacifica testnet", () => {
  const e = loadEnv("devnet");
  assert.equal(e.stageLabel, "TEST STAGE");
  assert.equal(e.cluster, "devnet");
  assert.match(e.pacificaApi, /test-api\.pacifica/);
  assert.equal(e.phoenixExecutable, false);
  assert.equal(e.driftWrites, false);
  assert.equal(e.investLiveSwaps, false);
});

test("mainnet keeps Phoenix non-executable until gate P1", () => {
  const e = loadEnv("mainnet");
  assert.equal(e.cluster, "mainnet-beta");
  assert.equal(e.phoenixExecutable, false);
  assert.equal(e.investLiveSwaps, true);
  assert.equal(e.driftWrites, false);
});
