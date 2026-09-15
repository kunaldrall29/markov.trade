import { test } from "node:test";
import assert from "node:assert/strict";
import { PublicKey } from "@solana/web3.js";
import { PROGRAM_ID } from "@markov/facts";
import { deriveAccount, deriveConfig, explorerTx } from "./index.ts";

test("PDAs are stable for the facts program id", () => {
  const owner = new PublicKey("11111111111111111111111111111111");
  const a = deriveAccount(owner);
  const b = deriveAccount(owner, new PublicKey(PROGRAM_ID));
  assert.equal(a.toBase58(), b.toBase58());
  assert.notEqual(deriveConfig().toBase58(), a.toBase58());
});

test("explorer links carry the cluster", () => {
  assert.match(explorerTx("sig", "devnet"), /cluster=devnet/);
  assert.doesNotMatch(explorerTx("sig", "mainnet-beta"), /cluster=/);
});
