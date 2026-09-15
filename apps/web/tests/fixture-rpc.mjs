#!/usr/bin/env node
/**
 * A Solana JSON-RPC stand-in for Playwright, serving `tests/fixtures/rpc.json`.
 *
 * It is a test double of the *transport*, not of the program: every account
 * byte it returns was encoded by the SDK's codecs, which are tested against
 * bytes the program crate itself serialised. It exists because this build
 * environment cannot reach devnet; the same specs run unchanged against a
 * real endpoint when `E2E_RPC_URL` is set.
 *
 * Control endpoints (test only):
 *   POST /__scenario {"state":"Active"|"Paused"|"Revoked","down":bool,"owner":"<address>"}
 *   GET  /__captured   → nothing; captured transactions live in the page
 */
import { createServer } from "node:http";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const fx = JSON.parse(readFileSync(join(here, "fixtures", "rpc.json"), "utf8"));
const PORT = Number(process.env.FIXTURE_RPC_PORT || 8899);

const STATE = { Active: 0, Paused: 1, Revoked: 2 };
const scenario = { state: "Active", down: false, owner: null, ownerAta: null, halt: false };
const started = Date.now();
const startSlot = fx.slot;

const b64 = {
  decode: (s) => Buffer.from(s, "base64"),
  encode: (b) => Buffer.from(b).toString("base64"),
};
const ALPHABET = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
function base58decode(str) {
  const bytes = [0];
  for (const ch of str) {
    let carry = ALPHABET.indexOf(ch);
    if (carry < 0) throw new Error("bad base58");
    for (let j = 0; j < bytes.length; j += 1) {
      carry += bytes[j] * 58;
      bytes[j] = carry & 0xff;
      carry >>= 8;
    }
    while (carry > 0) {
      bytes.push(carry & 0xff);
      carry >>= 8;
    }
  }
  for (const ch of str) {
    if (ch !== "1") break;
    bytes.push(0);
  }
  return Buffer.from(bytes.reverse());
}

function slotNow() {
  return startSlot + Math.floor((Date.now() - started) / 165);
}

/** Account bytes with the scenario applied: state byte, owner, fresh Pyth timestamps. */
function accountFor(address) {
  // The re-owned mandate's owner also gets the fixture's USDC-d token account,
  // relocated to the ATA the test derived for it (SPL layout: mint, owner, …).
  if (scenario.ownerAta && address === scenario.ownerAta) {
    const src = fx.accounts[fx.addresses.ownerAta];
    const data = b64.decode(src.data);
    base58decode(scenario.owner).copy(data, 32);
    return { data: [b64.encode(data), "base64"], executable: false, lamports: 2_039_280, owner: src.owner, rentEpoch: 0, space: data.length };
  }
  const a = fx.accounts[address];
  if (!a) return null;
  const data = b64.decode(a.data);
  if (address === fx.addresses.mandate) {
    data[fx.offsets.mandateState] = STATE[scenario.state];
    if (scenario.owner) base58decode(scenario.owner).copy(data, fx.offsets.mandateOwner);
  }
  if (address === fx.addresses.pyth) {
    data.writeBigInt64LE(BigInt(Math.floor(Date.now() / 1000) - 12), fx.offsets.pythPublishTime);
    data.writeBigUInt64LE(BigInt(slotNow() - 40), fx.offsets.pythPostedSlot);
  }
  if (address === fx.addresses.registry && scenario.halt) {
    data[8 + 32] = 1;
  }
  return { data: [b64.encode(data), "base64"], executable: false, lamports: a.lamports ?? 2_039_280, owner: a.owner, rentEpoch: 0, space: data.length };
}

function ctx() {
  return { context: { slot: slotNow(), apiVersion: "fixture" } };
}

function handle(req) {
  const { method, params = [] } = req;
  const slot = slotNow();
  switch (method) {
    case "getSlot":
      return slot;
    case "getBlockHeight":
      return slot - 40_000_000;
    case "getGenesisHash":
      return "EtWTRABZaYq6iMfeYKouRu166VU2xqa1wcaWoxPkrZBG";
    case "getLatestBlockhash":
      return { ...ctx(), value: { blockhash: "9sHcv6xwn9YkB8nxTUYMEm1MAj5yBcMMSc2zhpnXvNyD", lastValidBlockHeight: slot - 40_000_000 + 150 } };
    case "getAccountInfo": {
      const [address] = params;
      return { ...ctx(), value: accountFor(address) };
    }
    case "getMultipleAccounts": {
      const [addresses] = params;
      return { ...ctx(), value: addresses.map(accountFor) };
    }
    case "getBalance":
      return { ...ctx(), value: 1_500_000_000 };
    case "getProgramAccounts": {
      const [program, cfg = {}] = params;
      const filters = cfg.filters ?? [];
      const out = [];
      for (const address of Object.keys(fx.accounts)) {
        const acc = accountFor(address);
        if (!acc || acc.owner !== program) continue;
        const data = b64.decode(acc.data[0]);
        const ok = filters.every((f) => {
          if (f.memcmp) {
            const want = base58decode(f.memcmp.bytes);
            const off = Number(f.memcmp.offset);
            return data.subarray(off, off + want.length).equals(want);
          }
          if (f.dataSize != null) return data.length === Number(f.dataSize);
          return true;
        });
        if (ok) out.push({ pubkey: address, account: acc });
      }
      return out;
    }
    case "getSignaturesForAddress": {
      const [address, cfg = {}] = params;
      const limit = Number(cfg.limit ?? 1000);
      let txs = fx.transactions.filter((t) => t.accountKeys.includes(address));
      if (cfg.before) {
        const i = txs.findIndex((t) => t.signature === cfg.before);
        txs = i >= 0 ? txs.slice(i + 1) : [];
      }
      return txs.slice(0, limit).map((t) => ({ signature: t.signature, slot: t.slot, blockTime: t.blockTime, err: t.err, memo: null, confirmationStatus: "finalized" }));
    }
    case "getTransaction": {
      const [signature] = params;
      const t = fx.transactions.find((x) => x.signature === signature);
      if (!t) return null;
      const programIndex = t.accountKeys.indexOf(t.programIds[0]);
      return {
        slot: t.slot,
        blockTime: t.blockTime,
        version: "legacy",
        meta: {
          err: t.err,
          fee: 5000,
          status: t.err ? { Err: t.err } : { Ok: null },
          logMessages: [],
          preBalances: [],
          postBalances: [],
          innerInstructions: [{ index: 0, instructions: t.innerData.map((data) => ({ programIdIndex: programIndex, accounts: [], data, stackHeight: 2 })) }],
        },
        transaction: { message: { accountKeys: t.accountKeys, header: { numRequiredSignatures: 1, numReadonlySignedAccounts: 0, numReadonlyUnsignedAccounts: 1 }, instructions: [{ programIdIndex: programIndex, accounts: [], data: "" }], recentBlockhash: "9sHcv6xwn9YkB8nxTUYMEm1MAj5yBcMMSc2zhpnXvNyD" }, signatures: [t.signature] },
      };
    }
    case "getSignatureStatuses":
      return { ...ctx(), value: params[0].map(() => null) };
    case "sendTransaction":
      throw Object.assign(new Error("fixture rpc does not accept transactions; use E2E_RPC_URL for a real endpoint"), { code: -32003 });
    default:
      throw Object.assign(new Error(`fixture rpc: method ${method} not implemented`), { code: -32601 });
  }
}

const server = createServer((req, res) => {
  const cors = { "access-control-allow-origin": "*", "access-control-allow-headers": "content-type,solana-client", "access-control-allow-methods": "POST, GET, OPTIONS" };
  if (req.method === "OPTIONS") {
    res.writeHead(204, cors);
    return res.end();
  }
  let body = "";
  req.on("data", (c) => (body += c));
  req.on("end", () => {
    if (req.url === "/__scenario" && req.method === "POST") {
      Object.assign(scenario, JSON.parse(body || "{}"));
      res.writeHead(200, { ...cors, "content-type": "application/json" });
      return res.end(JSON.stringify(scenario));
    }
    if (req.url === "/__scenario") {
      res.writeHead(200, { ...cors, "content-type": "application/json" });
      return res.end(JSON.stringify(scenario));
    }
    if (scenario.down) {
      res.writeHead(503, cors);
      return res.end("fixture rpc: scenario down");
    }
    let parsed;
    try {
      parsed = JSON.parse(body);
    } catch {
      res.writeHead(400, cors);
      return res.end("bad json");
    }
    const reqs = Array.isArray(parsed) ? parsed : [parsed];
    const out = reqs.map((r) => {
      try {
        return { jsonrpc: "2.0", id: r.id, result: handle(r) };
      } catch (e) {
        return { jsonrpc: "2.0", id: r.id, error: { code: e.code ?? -32000, message: e.message } };
      }
    });
    res.writeHead(200, { ...cors, "content-type": "application/json" });
    res.end(JSON.stringify(Array.isArray(parsed) ? out : out[0]));
  });
});

server.listen(PORT, "127.0.0.1", () => console.log(`fixture rpc on http://127.0.0.1:${PORT} (${Object.keys(fx.accounts).length} accounts, ${fx.transactions.length} transactions)`));
