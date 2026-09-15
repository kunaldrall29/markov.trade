import Fastify from "fastify";
import cors from "@fastify/cors";
import websocket from "@fastify/websocket";
import { randomBytes, createHash, randomUUID } from "node:crypto";
import nacl from "tweetnacl";
import bs58 from "bs58";
import { REASON, REASON_NAME, STAGE } from "@markov/facts";
import { loadEnv, readName } from "@markov/config";
import {
  bestMark,
  capabilities,
  createCache,
  listMarkets,
  policyPreview,
  recordReceipt,
  refreshMarkets,
  routesFor,
} from "./engine.ts";
import { buildPacificaSignable, jupiterQuote } from "@markov/adapters";

const PORT = Number(process.env.PORT || 4000);
const HOST = process.env.HOST || "0.0.0.0";

export async function buildServer() {
  const env = loadEnv(readName());
  const cache = createCache(env);
  const app = Fastify({ logger: true });
  await app.register(cors, { origin: true });
  await app.register(websocket);

  app.get("/health", async () => ({
    ok: true,
    env: env.name,
    stage: env.stageLabel,
    cluster: env.cluster,
    program_id: env.programId,
    pacifica_markets: cache.pacifica.size,
    phoenix_markets: cache.phoenix.size,
    last_pacifica_ok: cache.lastPacificaOk || null,
    last_phoenix_ok: cache.lastPhoenixOk || null,
  }));

  app.get("/markets", async () => ({
    env: env.name,
    data_slot: Math.max(0, ...listMarkets(cache).map((m) => m.data_slot ?? 0)),
    markets: listMarkets(cache),
  }));

  app.get("/markets/:id", async (req, reply) => {
    const id = (req.params as { id: string }).id;
    const venues = listMarkets(cache).filter((m) => m.canonical_market_id === id);
    const market = bestMark(cache, id);
    if (!venues.length) return reply.code(404).send({ error: "unknown market", id });
    return { env: env.name, market, venues };
  });

  app.get("/venues/capabilities", async () => ({
    env: env.name,
    stage: env.stageLabel,
    venues: capabilities(cache),
  }));

  app.get("/venues/accounts", async () => ({
    env: env.name,
    accounts: [
      { venue: "pacifica", linked: false, env: env.name === "devnet" ? "testnet" : "mainnet", note: "Link requires owner signature in /venues" },
      { venue: "drift", linked: false, env: env.cluster, note: "Initialize + deposit is owner-signed; SDK path pending RPC" },
      { venue: "phoenix", linked: false, env: "mainnet", note: "INTEGRATED · read-only until gate P1" },
    ],
  }));

  app.post("/policy/check", async (req) => {
    const body = req.body as { market: string; venue?: string; notional_usd: number; leverage: number; max_slippage_bps?: number };
    return policyPreview(cache, body);
  });

  app.post("/routes/compare", async (req) => {
    const body = req.body as { market: string; notional_usd: number; horizon_hours?: number };
    return routesFor(cache, body.market, body.notional_usd, body.horizon_hours ?? 24);
  });

  app.post("/risk/simulate-trade", async (req) => {
    const body = req.body as { market: string; notional_usd: number; leverage: number; venue?: string };
    const preview = policyPreview(cache, body);
    return { env: env.name, policy: preview, projected: { notional_usd: body.notional_usd, leverage: body.leverage } };
  });

  app.post("/trades/request", async (req, reply) => {
    const body = req.body as {
      market: string;
      side: "long" | "short";
      notional_usd: number;
      leverage: number;
      venue?: string;
      horizon_hours?: number;
      max_slippage_bps?: number;
    };
    const session = sessionOf(req, cache);
    const preview = policyPreview(cache, body);
    const request_id = randomUUID();
    const actor = session?.pubkey ?? "anonymous";
    if (preview.decision !== "ALLOW") {
      const receipt = recordReceipt(cache, {
        request_id,
        actor,
        kind: "TradeOpen",
        decision: preview.decision,
        reason_code: preview.reason_code,
        reason: preview.reason,
        mandate_version: 1,
        invest_mandate_version: 0,
        data_slot: preview.data_slot,
        venue_id: body.venue === "phoenix" ? 3 : body.venue === "drift" ? 2 : 1,
        market_id: body.market,
        checks: preview.checks,
        tx_signature: null,
      });
      return { request_id, approval_required: false, ...preview, receipt_id: receipt.request_id };
    }
    if ((body.venue ?? "AUTO") === "phoenix") {
      return reply.code(400).send({ error: "Phoenix is INTEGRATED · read-only", reason_code: REASON.VENUE_NOT_ALLOWED });
    }
    if ((body.venue ?? "AUTO") === "drift") {
      return reply.code(400).send({ error: "Drift is not executable until RPC+SDK subscribe", reason_code: REASON.STALE_MARKET_DATA });
    }
    const receipt = recordReceipt(cache, {
      request_id,
      actor,
      kind: "TradeOpen",
      decision: "REQUIRE_APPROVAL",
      reason_code: REASON.EXECUTION_CONFIRMED,
      reason: "REQUIRE_APPROVAL",
      mandate_version: 1,
      invest_mandate_version: 0,
      data_slot: preview.data_slot,
      venue_id: 1,
      market_id: body.market,
      checks: preview.checks,
      tx_signature: null,
    });
    const timestamp = Date.now();
    const signable = session
      ? buildPacificaSignable({
          type: "create_order",
          account: session.pubkey,
          timestamp,
          expiry_window: 30_000,
          data: {
            symbol: body.market.replace("-PERP", ""),
            side: body.side === "long" ? "bid" : "ask",
            amount: String(body.notional_usd),
            reduce_only: false,
            tif: "IOC",
            client_order_id: request_id,
          },
        })
      : null;
    return {
      request_id,
      approval_required: true,
      ...preview,
      decision: "REQUIRE_APPROVAL",
      receipt_id: receipt.request_id,
      signables: [
        {
          kind: "pacifica_message",
          display: `create_order ${body.side} ${body.market} $${body.notional_usd} @ ${body.leverage}x on Pacifica ${env.name === "devnet" ? "testnet" : "mainnet"}`,
          compact_json: signable?.compactJson ?? null,
          note: session
            ? "Owner signs this compact JSON with the connected wallet. Markov does not submit the order until that signature is attached."
            : "Connect and complete SIWS to receive a wallet-bound Pacifica payload.",
        },
      ],
    };
  });

  app.post("/trades/reduce", async (req) => {
    const body = req.body as { market: string; notional_usd: number };
    const session = sessionOf(req, cache);
    const request_id = randomUUID();
    const receipt = recordReceipt(cache, {
      request_id,
      actor: session?.pubkey ?? "anonymous",
      kind: "TradeReduce",
      decision: "REQUIRE_APPROVAL",
      reason_code: REASON.EXECUTION_CONFIRMED,
      reason: "REQUIRE_APPROVAL",
      mandate_version: 1,
      invest_mandate_version: 0,
      data_slot: 0,
      venue_id: 1,
      market_id: body.market,
      checks: [],
      tx_signature: null,
    });
    return { request_id, receipt_id: receipt.request_id, note: "Reduce is owner-signed. No position exists until a venue account is linked." };
  });

  app.post("/trades/confirm-signature", async (req, reply) => {
    const body = req.body as { request_id: string; signature: string };
    const receipt = cache.receipts.find((r) => r.request_id === body.request_id);
    if (!receipt) return reply.code(404).send({ error: "not found" });
    receipt.tx_signature = body.signature;
    return { ok: true, receipt, note: "Signature stored on the in-process receipt. Venue submission is a later step; this is not a fill." };
  });

  app.get("/receipts", async () => ({ env: env.name, receipts: cache.receipts }));
  app.get("/receipts/:id", async (req, reply) => {
    const id = (req.params as { id: string }).id;
    const receipt = cache.receipts.find((r) => r.request_id === id);
    if (!receipt) return reply.code(404).send({ error: "not found" });
    return { env: env.name, receipt };
  });

  app.get("/account", async (req) => {
    const session = sessionOf(req, cache);
    return {
      env: env.name,
      stage: env.stageLabel,
      connected: Boolean(session),
      pubkey: session?.pubkey ?? null,
      note: session ? "Wallet verified via SIWS. No venue account is linked yet." : "Connect a wallet in the Terminal. Empty means empty.",
      program_id: env.programId,
      mandate: {
        version: 0,
        status: "none",
        max_leverage: "2.0x cap (global)",
        max_notional_usd: env.caps.perAccountGrossNotionalUsd,
      },
    };
  });

  app.get("/portfolio", async () => ({
    env: env.name,
    equity: null,
    gross_notional: null,
    note: "Empty until a venue account is linked.",
  }));
  app.get("/positions", async () => ({ env: env.name, positions: [] }));
  app.get("/holdings", async () => ({ env: env.name, holdings: [] }));
  app.get("/risk", async () => ({
    env: env.name,
    equity: null,
    effective_leverage: null,
    note: "No linked venue equity; leverage is null, not zero.",
  }));
  app.get("/mandate", async () => ({
    env: env.name,
    version: 0,
    hash: null,
    rules: {
      max_leverage_bps: 20_000,
      max_notional_usd: env.caps.perAccountGrossNotionalUsd,
      min_safety_buffer_bps: 2_000,
      max_daily_loss_usd: 50,
      approved_markets: ["SOL-PERP", "BTC-PERP", "ETH-PERP"],
    },
  }));
  app.get("/invest/rules", async () => ({
    env: env.name,
    rules: [],
    note: "No invest mandate until the owner signs one. xStocks exist on mainnet only.",
  }));
  app.get("/invest/history", async () => ({ env: env.name, receipts: cache.receipts.filter((r) => r.kind.startsWith("Invest")) }));
  app.post("/invest/quote", async (req) => {
    const body = req.body as { mint: string; usd: number };
    if (!env.investLiveSwaps) {
      return {
        env: env.name,
        executable: false,
        note: "xStocks are absent on devnet. A live Jupiter quote is mainnet-only.",
      };
    }
    const q = await jupiterQuote({ outputMint: body.mint, amount: Math.round((body.usd ?? 5) * 1_000_000) });
    return { env: env.name, executable: true, quote: q };
  });
  app.post("/invest/simulate", async (req) => {
    const body = req.body as { mint: string; usd: number };
    return {
      env: env.name,
      decision: env.investLiveSwaps ? "REQUIRE_APPROVAL" : "SKIP",
      reason: env.investLiveSwaps ? "Owner must sign the invest mandate and the Subscriptions delegation." : "MARKET_CLOSED",
      mint: body.mint,
      usd: body.usd,
    };
  });
  app.post("/invest/pause", async () => ({
    env: env.name,
    decision: "REQUIRE_APPROVAL",
    note: "Pause is owner-signed. No rule exists yet.",
  }));
  app.get("/alerts", async () => ({ env: env.name, alerts: [] }));
  app.get("/mcp/tools", async () => ({
    env: env.name,
    tools: [
      "markov.get_account_state",
      "markov.get_risk",
      "markov.get_markets",
      "markov.compare_routes",
      "markov.simulate_trade",
      "markov.request_trade",
      "markov.reduce_position",
      "markov.get_receipts",
      "markov.explain_receipt",
      "invest.get_rules",
      "invest.get_history",
      "invest.simulate_rule",
      "invest.propose_rule",
      "invest.pause_rule",
      "invest.quote_swap",
    ],
    absent: ["markov.withdraw", "markov.execute", "markov.set_mandate", "invest.execute", "invest.withdraw", "invest.set_allowlist"],
  }));

  app.post("/auth/challenge", async (req) => {
    const { pubkey } = req.body as { pubkey: string };
    const nonce = randomBytes(16).toString("hex");
    const domain = process.env.SIWS_DOMAIN || "localhost";
    const expires_at = new Date(Date.now() + 5 * 60_000).toISOString();
    const message = `Markov ${STAGE}\nDomain: ${domain}\nAddress: ${pubkey}\nNonce: ${nonce}\nExpires: ${expires_at}\nCluster: ${env.cluster}`;
    cache.nonces.set(nonce, { pubkey, exp: Date.now() + 5 * 60_000, message });
    return { nonce, message, expires_at, env: env.name };
  });

  app.post("/auth/verify", async (req, reply) => {
    const { pubkey, signature, nonce } = req.body as { pubkey: string; signature: string; nonce: string };
    const row = cache.nonces.get(nonce);
    if (!row || row.pubkey !== pubkey || row.exp < Date.now()) {
      return reply.code(401).send({ error: "bad nonce" });
    }
    cache.nonces.delete(nonce);
    const msg = new TextEncoder().encode(row.message);
    const sig = decodeSig(signature);
    const pk = bs58.decode(pubkey);
    if (pk.length !== 32 || !nacl.sign.detached.verify(msg, sig, pk)) {
      return reply.code(401).send({ error: "bad signature" });
    }
    const token = randomBytes(24).toString("hex");
    const refresh = randomBytes(24).toString("hex");
    cache.sessions.set(token, { pubkey, exp: Date.now() + 15 * 60_000 });
    cache.sessions.set(refresh, { pubkey, exp: Date.now() + 7 * 24 * 3600_000 });
    return { token, refresh, pubkey, env: env.name };
  });

  app.get("/auth/me", async (req, reply) => {
    const session = sessionOf(req, cache);
    if (!session) return reply.code(401).send({ error: "no session" });
    return { pubkey: session.pubkey, env: env.name };
  });

  app.get("/ws", { websocket: true }, (socket) => {
    const tick = setInterval(() => {
      socket.send(JSON.stringify({ type: "markets", env: env.name, markets: listMarkets(cache).slice(0, 12) }));
    }, 2000);
    socket.on("close", () => clearInterval(tick));
  });

  setImmediate(() => {
    refreshMarkets(cache).catch((err) => app.log.error(err));
  });
  const timer = setInterval(() => {
    refreshMarkets(cache).catch((err) => app.log.error(err));
  }, 2500);
  app.addHook("onClose", async () => {
    clearInterval(timer);
  });

  return { app, cache };
}

function sessionOf(req: { headers: { authorization?: string } }, cache: ReturnType<typeof createCache>) {
  const h = req.headers.authorization;
  if (!h?.startsWith("Bearer ")) return null;
  const token = h.slice(7);
  const row = cache.sessions.get(token);
  if (!row || row.exp < Date.now()) return null;
  return row;
}

function decodeSig(signature: string): Uint8Array {
  try {
    return bs58.decode(signature);
  } catch {
    return Buffer.from(signature, "base64");
  }
}

void createHash; // keep crypto import used if tree-shaken oddly

if (import.meta.url === `file://${process.argv[1]}` || process.argv[1]?.endsWith("index.ts")) {
  const { app } = await buildServer();
  await app.listen({ port: PORT, host: HOST });
  console.log(`markov api ${STAGE} on ${HOST}:${PORT}`);
}
