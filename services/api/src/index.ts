import Fastify from "fastify";
import cors from "@fastify/cors";
import websocket from "@fastify/websocket";
import { randomBytes, randomUUID } from "node:crypto";
import nacl from "tweetnacl";
import bs58 from "bs58";
import { CANONICAL_MARKETS, REASON, REASON_NAME, STAGE, XSTOCKS } from "@markov/facts";
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
  type Cache,
} from "./engine.ts";
import { persist } from "./store.ts";
import { getAccountExecutable } from "./chain.ts";
import {
  buildPacificaCreateOrder,
  jupiterQuote,
  pacificaAccount,
  pacificaKlines,
  pacificaPositions,
  submitPacificaOrder,
  type PacificaOrderFields,
} from "@markov/adapters";
import { OPENAPI } from "./openapi.ts";

const PORT = Number(process.env.PORT || 4000);
const HOST = process.env.HOST || "0.0.0.0";

export async function buildServer() {
  const env = loadEnv(readName());
  const cache = createCache(env);
  const app = Fastify({ logger: true });
  await app.register(cors, { origin: true });
  await app.register(websocket);

  app.addHook("onRequest", async (req, reply) => {
    if (req.method === "GET" || req.method === "HEAD") return;
    const ip = req.ip || "local";
    if (!rateOk(ip)) {
      return reply.code(429).send({ error: "rate limited" });
    }
  });

  app.get("/health", async () => {
    await maybeProbeProgram(cache);
    return {
      ok: true,
      env: env.name,
      stage: env.stageLabel,
      cluster: env.cluster,
      program_id: env.programId,
      program_deployed: cache.program.deployed,
      slot: cache.program.slot,
      pacifica_markets: cache.pacifica.size,
      phoenix_markets: cache.phoenix.size,
      last_pacifica_ok: cache.lastPacificaOk || null,
      last_phoenix_ok: cache.lastPhoenixOk || null,
      receipts: cache.receipts.length,
    };
  });

  app.get("/openapi.json", async () => OPENAPI);

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

  app.get("/markets/:id/depth", async (req, reply) => {
    const id = (req.params as { id: string }).id;
    const venues = listMarkets(cache).filter((m) => m.canonical_market_id === id);
    if (!venues.length) return reply.code(404).send({ error: "unknown market", id });
    return {
      env: env.name,
      market: id,
      books: venues.map((v) => ({
        venue: v.venue,
        stale: v.stale,
        venue_ts: v.venue_ts,
        bids: v.bids ?? [],
        asks: v.asks ?? [],
      })),
    };
  });

  app.get("/markets/:id/candles", async (req, reply) => {
    const id = (req.params as { id: string }).id;
    const meta = CANONICAL_MARKETS.find((m) => m.id === id);
    if (!meta) return reply.code(404).send({ error: "unknown market", id });
    const q = req.query as { venue?: string; interval?: string; hours?: string };
    if (q.venue && q.venue !== "pacifica") {
      return {
        env: env.name,
        market: id,
        venue: q.venue,
        candles: [],
        note: "Only Pacifica exposes a verified kline in this build. Phoenix candles are not invented.",
      };
    }
    try {
      const pacEnv = env.name === "devnet" ? "testnet" : "mainnet";
      const candles = await pacificaKlines(meta.pacifica, pacEnv, q.interval || "1h", Number(q.hours) || 48);
      return { env: env.name, market: id, venue: "pacifica", interval: q.interval || "1h", candles };
    } catch (err) {
      return reply.code(502).send({ error: "kline fetch failed", detail: String(err) });
    }
  });

  app.get("/markets/:id/funding-history", async (req, reply) => {
    const id = (req.params as { id: string }).id;
    const venues = listMarkets(cache).filter((m) => m.canonical_market_id === id);
    if (!venues.length) return reply.code(404).send({ error: "unknown market", id });
    return {
      env: env.name,
      market: id,
      note: "Pacifica has no public funding-history path in this FACTS pass. One live print per venue, not a fabricated series.",
      points: venues.map((v) => ({
        venue: v.venue,
        funding: v.funding,
        next_funding: v.next_funding,
        venue_ts: v.venue_ts,
      })),
    };
  });

  app.get("/venues/capabilities", async () => ({
    env: env.name,
    stage: env.stageLabel,
    venues: capabilities(cache),
  }));

  app.get("/venues/accounts", async (req) => {
    const session = sessionOf(req, cache);
    const pacEnv = env.name === "devnet" ? "testnet" : "mainnet";
    const link = session ? cache.links.find((l) => l.owner === session.pubkey && l.venue === "pacifica") : undefined;
    let pacificaFound: boolean | null = link?.venue_account_found ?? null;
    let positions: unknown[] = [];
    if (session) {
      try {
        const acct = await pacificaAccount(session.pubkey, pacEnv);
        pacificaFound = acct.found;
        positions = await pacificaPositions(session.pubkey, pacEnv);
      } catch {
        /* leave found as previously stored */
      }
    }
    return {
      env: env.name,
      accounts: [
        {
          venue: "pacifica",
          linked: Boolean(link),
          venue_account_found: pacificaFound,
          env: pacEnv,
          positions,
          note: pacificaFound
            ? "Pacifica /account returned this wallet."
            : "Link requires an owner signature. Create the venue account on Pacifica if /account is 404.",
        },
        {
          venue: "drift",
          linked: false,
          env: env.cluster,
          note: "Initialize + deposit is owner-signed; SDK path pending RPC",
        },
        {
          venue: "phoenix",
          linked: false,
          env: "mainnet",
          note: "INTEGRATED · read-only until gate P1",
        },
      ],
    };
  });

  app.post("/venues/pacifica/bind", async (req, reply) => {
    const session = sessionOf(req, cache);
    if (!session) return reply.code(401).send({ error: "SIWS required" });
    const pacEnv = env.name === "devnet" ? "testnet" : "mainnet";
    let found: boolean | null = null;
    try {
      found = (await pacificaAccount(session.pubkey, pacEnv)).found;
    } catch {
      found = null;
    }
    cache.links = cache.links.filter((l) => !(l.owner === session.pubkey && l.venue === "pacifica"));
    cache.links.push({
      venue: "pacifica",
      owner: session.pubkey,
      linked_at: new Date().toISOString(),
      venue_account_found: found,
    });
    persist(cache);
    const receipt = recordReceipt(cache, {
      request_id: randomUUID(),
      actor: session.pubkey,
      kind: "VenueLink",
      decision: "REQUIRE_APPROVAL",
      reason_code: REASON.EXECUTION_CONFIRMED,
      reason: found ? "PACIFICA_ACCOUNT_FOUND" : "PACIFICA_ACCOUNT_ABSENT",
      mandate_version: activeMandate(cache, session.pubkey).version,
      invest_mandate_version: 0,
      data_slot: 0,
      venue_id: 1,
      market_id: "LINK",
      checks: [],
      tx_signature: null,
    });
    return {
      ok: true,
      linked: true,
      venue_account_found: found,
      receipt_id: receipt.request_id,
      note: found
        ? "Wallet exists on Pacifica. Balances still come from the venue; Markov does not custody."
        : "Bind stored. Pacifica /account is 404 for this wallet — open Pacifica testnet and create the account before an order can land.",
    };
  });

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

  app.post("/risk/simulate-scenario", async (req) => {
    const body = req.body as { price_shock_bps?: number };
    return {
      env: env.name,
      equityUsd: null,
      leverage: null,
      price_shock_bps: body.price_shock_bps ?? 0,
      note: "No linked venue equity. Shock is not applied to invented numbers.",
    };
  });

  app.post("/trades/request", async (req, reply) => {
    const dup = idempotent(req, cache);
    if (dup) return reply.code(dup.status).send(dup.body);
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
    const request_id = (req.headers["idempotency-key"] as string) || randomUUID();
    const actor = session?.pubkey ?? "anonymous";
    if (preview.decision !== "ALLOW") {
      const receipt = recordReceipt(cache, {
        request_id,
        actor,
        kind: "TradeOpen",
        decision: preview.decision,
        reason_code: preview.reason_code,
        reason: preview.reason,
        mandate_version: activeMandate(cache, actor).version,
        invest_mandate_version: 0,
        data_slot: preview.data_slot,
        venue_id: body.venue === "phoenix" ? 3 : body.venue === "drift" ? 2 : 1,
        market_id: body.market,
        checks: preview.checks,
        tx_signature: null,
      });
      cache.pending.unshift({
        request_id,
        status: "REJECTED",
        actor,
        market: body.market,
        compact_json: null,
        timestamp: null,
        expiry_window: null,
        fields: null,
        created_at: receipt.created_at,
      });
      persist(cache);
      const out = { request_id, approval_required: false, ...preview, receipt_id: receipt.request_id, state: "REJECTED" };
      remember(req, cache, 200, out);
      return out;
    }
    if ((body.venue ?? "AUTO") === "phoenix") {
      return reply.code(400).send({ error: "Phoenix is INTEGRATED · read-only", reason_code: REASON.VENUE_NOT_ALLOWED });
    }
    if ((body.venue ?? "AUTO") === "drift") {
      return reply.code(400).send({ error: "Drift is not executable until RPC+SDK subscribe", reason_code: REASON.STALE_MARKET_DATA });
    }
    const state = cache.pacifica.get(body.market);
    let fields: PacificaOrderFields | null = null;
    let compact_json: string | null = null;
    let timestamp: number | null = null;
    let expiry_window: number | null = null;
    let display: string | null = null;
    if (session && state && state.mark > 0) {
      try {
        const built = buildPacificaCreateOrder({
          account: session.pubkey,
          symbol: body.market.replace("-PERP", ""),
          side: body.side,
          notionalUsd: body.notional_usd,
          mark: state.mark,
          tickSize: state.tickSize ?? 0.01,
          lotSize: state.lotSize ?? 0.01,
          bestBid: state.bids[0]?.price ?? null,
          bestAsk: state.asks[0]?.price ?? null,
          tif: "IOC",
          clientOrderId: request_id,
        });
        fields = built.fields;
        compact_json = built.compactJson;
        timestamp = built.timestamp;
        expiry_window = built.expiry_window;
        display = `create_order ${body.side} ${body.market} ${built.fields.amount} @ ${built.fields.price} (notional $${body.notional_usd}, ${body.leverage}x) on Pacifica ${env.name === "devnet" ? "testnet" : "mainnet"}`;
      } catch (err) {
        return reply.code(400).send({ error: "cannot build order", detail: String(err) });
      }
    }
    const receipt = recordReceipt(cache, {
      request_id,
      actor,
      kind: "TradeOpen",
      decision: "REQUIRE_APPROVAL",
      reason_code: REASON.EXECUTION_CONFIRMED,
      reason: "REQUIRE_APPROVAL",
      mandate_version: activeMandate(cache, actor).version,
      invest_mandate_version: 0,
      data_slot: preview.data_slot,
      venue_id: 1,
      market_id: body.market,
      checks: preview.checks,
      tx_signature: null,
    });
    cache.pending.unshift({
      request_id,
      status: "AWAITING_SIGNATURE",
      actor,
      market: body.market,
      compact_json,
      timestamp,
      expiry_window,
      fields,
      created_at: receipt.created_at,
    });
    persist(cache);
    const out = {
      request_id,
      approval_required: true,
      ...preview,
      decision: "REQUIRE_APPROVAL" as const,
      receipt_id: receipt.request_id,
      state: "AWAITING_SIGNATURE",
      signables: [
        {
          kind: "pacifica_message",
          display:
            display ??
            `create_order ${body.side} ${body.market} $${body.notional_usd} @ ${body.leverage}x on Pacifica ${env.name === "devnet" ? "testnet" : "mainnet"}`,
          compact_json,
          fields,
          note: session
            ? "Owner signs this compact JSON with the connected wallet. Markov relays it to Pacifica only after that signature is attached. A venue reject is a receipt, not a fill."
            : "Connect and complete SIWS to receive a wallet-bound Pacifica payload.",
        },
      ],
    };
    remember(req, cache, 200, out);
    return out;
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
      mandate_version: activeMandate(cache, session?.pubkey ?? "anonymous").version,
      invest_mandate_version: 0,
      data_slot: 0,
      venue_id: 1,
      market_id: body.market,
      checks: [],
      tx_signature: null,
    });
    return { request_id, receipt_id: receipt.request_id, note: "Reduce is owner-signed. No position exists until a venue account is linked." };
  });

  app.post("/trades/confirm-signature", async (req, reply) => confirmAndSubmit(req, reply, cache, env.name));
  app.post("/trades/submit", async (req, reply) => confirmAndSubmit(req, reply, cache, env.name));

  app.get("/trades/:id", async (req, reply) => {
    const id = (req.params as { id: string }).id;
    const pending = cache.pending.find((p) => p.request_id === id);
    const receipt = cache.receipts.find((r) => r.request_id === id);
    if (!pending && !receipt) return reply.code(404).send({ error: "not found" });
    return { env: env.name, pending: pending ?? null, receipt: receipt ?? null };
  });

  app.get("/receipts", async () => ({ env: env.name, receipts: cache.receipts }));
  app.get("/receipts/:id", async (req, reply) => {
    const id = (req.params as { id: string }).id;
    const receipt = cache.receipts.find((r) => r.request_id === id);
    if (!receipt) return reply.code(404).send({ error: "not found" });
    const pending = cache.pending.find((p) => p.request_id === id);
    return { env: env.name, receipt, pending: pending ?? null };
  });

  app.get("/account", async (req) => {
    const session = sessionOf(req, cache);
    const mandate = session ? activeMandate(cache, session.pubkey) : null;
    return {
      env: env.name,
      stage: env.stageLabel,
      connected: Boolean(session),
      pubkey: session?.pubkey ?? null,
      note: session ? "Wallet verified via SIWS. Venue balances are empty until Pacifica /account exists." : "Connect a wallet in the Terminal. Empty means empty.",
      program_id: env.programId,
      program_deployed: cache.program.deployed,
      mandate: mandate
        ? {
            version: mandate.version,
            status: mandate.on_chain ? "on-chain" : "draft",
            max_leverage: `${(mandate.max_leverage_bps / 10_000).toFixed(1)}x`,
            max_notional_usd: mandate.max_notional_usd,
            on_chain: mandate.on_chain,
          }
        : {
            version: 0,
            status: "none",
            max_leverage: "2.0x cap (global)",
            max_notional_usd: env.caps.perAccountGrossNotionalUsd,
            on_chain: false,
          },
    };
  });

  app.get("/portfolio", async (req) => {
    const session = sessionOf(req, cache);
    return {
      env: env.name,
      equity: null,
      gross_notional: null,
      pubkey: session?.pubkey ?? null,
      note: "Empty until a venue account is linked and returns balances.",
    };
  });
  app.get("/portfolio/history", async () => ({
    env: env.name,
    points: [],
    note: "No equity series until a venue account exists.",
  }));
  app.get("/positions", async (req) => {
    const session = sessionOf(req, cache);
    if (!session) return { env: env.name, positions: [] };
    const pacEnv = env.name === "devnet" ? "testnet" : "mainnet";
    try {
      const positions = await pacificaPositions(session.pubkey, pacEnv);
      return { env: env.name, positions, source: "pacifica /positions" };
    } catch (err) {
      return { env: env.name, positions: [], error: String(err) };
    }
  });
  app.get("/holdings", async () => ({ env: env.name, holdings: [] }));
  app.get("/risk", async () => ({
    env: env.name,
    equity: null,
    effective_leverage: null,
    note: "No linked venue equity; leverage is null, not zero.",
  }));

  app.get("/mandate", async (req) => {
    const session = sessionOf(req, cache);
    const m = session ? cache.mandates.find((x) => x.owner === session.pubkey) : undefined;
    return {
      env: env.name,
      version: m?.version ?? 0,
      hash: null,
      on_chain: false,
      program_deployed: cache.program.deployed,
      rules: m
        ? {
            max_leverage_bps: m.max_leverage_bps,
            max_notional_usd: m.max_notional_usd,
            min_safety_buffer_bps: m.min_safety_buffer_bps,
            max_daily_loss_usd: m.max_daily_loss_usd,
            approved_markets: m.approved_markets,
          }
        : {
            max_leverage_bps: 20_000,
            max_notional_usd: env.caps.perAccountGrossNotionalUsd,
            min_safety_buffer_bps: 2_000,
            max_daily_loss_usd: 50,
            approved_markets: ["SOL-PERP", "BTC-PERP", "ETH-PERP"],
          },
      note: cache.program.deployed
        ? "Draft until set_mandate lands."
        : "Program undeployed. This draft is off-chain until FACTS says the program is live.",
    };
  });

  app.post("/mandate", async (req, reply) => {
    const session = sessionOf(req, cache);
    if (!session) return reply.code(401).send({ error: "SIWS required" });
    const body = req.body as {
      max_leverage_bps?: number;
      max_notional_usd?: number;
      min_safety_buffer_bps?: number;
      max_daily_loss_usd?: number;
      approved_markets?: string[];
    };
    const prev = cache.mandates.find((x) => x.owner === session.pubkey);
    const draft = {
      owner: session.pubkey,
      version: (prev?.version ?? 0) + 1,
      max_leverage_bps: Math.min(body.max_leverage_bps ?? 20_000, 20_000),
      max_notional_usd: Math.min(body.max_notional_usd ?? env.caps.perAccountGrossNotionalUsd, env.caps.perAccountGrossNotionalUsd),
      min_safety_buffer_bps: body.min_safety_buffer_bps ?? 2_000,
      max_daily_loss_usd: body.max_daily_loss_usd ?? 50,
      approved_markets: body.approved_markets ?? ["SOL-PERP", "BTC-PERP", "ETH-PERP"],
      updated_at: new Date().toISOString(),
      on_chain: false as const,
    };
    cache.mandates = cache.mandates.filter((x) => x.owner !== session.pubkey);
    cache.mandates.push(draft);
    const receipt = recordReceipt(cache, {
      request_id: randomUUID(),
      actor: session.pubkey,
      kind: "MandateSet",
      decision: "REQUIRE_APPROVAL",
      reason_code: REASON.EXECUTION_CONFIRMED,
      reason: "REQUIRE_APPROVAL",
      mandate_version: draft.version,
      invest_mandate_version: 0,
      data_slot: 0,
      venue_id: 0,
      market_id: "MANDATE",
      checks: [],
      tx_signature: null,
    });
    cache.proposals.unshift({
      id: receipt.request_id,
      kind: "mandate",
      actor: session.pubkey,
      payload: draft,
      status: "pending",
      created_at: receipt.created_at,
      receipt_id: receipt.request_id,
    });
    persist(cache);
    return { ok: true, draft, receipt_id: receipt.request_id, on_chain: false };
  });

  app.post("/mandate/pause", async (req, reply) => {
    const session = sessionOf(req, cache);
    if (!session) return reply.code(401).send({ error: "SIWS required" });
    const receipt = recordReceipt(cache, {
      request_id: randomUUID(),
      actor: session.pubkey,
      kind: "MandatePause",
      decision: "REQUIRE_APPROVAL",
      reason_code: REASON.EXECUTION_CONFIRMED,
      reason: "REQUIRE_APPROVAL",
      mandate_version: activeMandate(cache, session.pubkey).version,
      invest_mandate_version: 0,
      data_slot: 0,
      venue_id: 0,
      market_id: "PAUSE",
      checks: [],
      tx_signature: null,
    });
    return { ok: true, receipt_id: receipt.request_id, note: "Pause is owner-signed on chain once the program is deployed." };
  });

  app.get("/invest/rules", async (req) => {
    const session = sessionOf(req, cache);
    const rules = session ? cache.investRules.filter((r) => r.owner === session.pubkey) : [];
    return {
      env: env.name,
      rules,
      assets: XSTOCKS,
      note: "No on-chain invest mandate until the owner signs one. xStocks exist on mainnet only.",
    };
  });
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
  app.post("/invest/propose", async (req, reply) => {
    const session = sessionOf(req, cache);
    const body = req.body as { mint: string; usd_per_period: number; period_seconds: number; mode?: string };
    if (!body?.mint || !(body.usd_per_period > 0)) return reply.code(400).send({ error: "mint and usd_per_period required" });
    const actor = session?.pubkey ?? "mcp";
    const request_id = randomUUID();
    const receipt = recordReceipt(cache, {
      request_id,
      actor,
      kind: "InvestPropose",
      decision: "REQUIRE_APPROVAL",
      reason_code: REASON.EXECUTION_CONFIRMED,
      reason: "REQUIRE_APPROVAL",
      mandate_version: 0,
      invest_mandate_version: 0,
      data_slot: 0,
      venue_id: 4,
      market_id: body.mint,
      checks: [],
      tx_signature: null,
    });
    const rule = {
      id: request_id,
      owner: actor,
      mint: body.mint,
      usd_per_period: body.usd_per_period,
      period_seconds: body.period_seconds ?? 604800,
      mode: body.mode ?? "AlwaysOn",
      status: "pending_signatures",
      on_chain: false,
      note: env.investLiveSwaps
        ? "Owner must sign invest mandate + Subscriptions delegation."
        : "Draft only on devnet. xStocks mints are mainnet; keeper will record_skip until then.",
    };
    cache.investRules.unshift(rule);
    cache.proposals.unshift({
      id: request_id,
      kind: "invest_rule",
      actor,
      payload: rule,
      status: "pending",
      created_at: receipt.created_at,
      receipt_id: request_id,
    });
    persist(cache);
    return { decision: "REQUIRE_APPROVAL", signable_for_owner: Boolean(session), signable_for_model: false, rule, receipt_id: request_id };
  });
  app.post("/invest/pause", async (req, reply) => {
    const session = sessionOf(req, cache);
    const body = req.body as { id?: string };
    const receipt = recordReceipt(cache, {
      request_id: randomUUID(),
      actor: session?.pubkey ?? "anonymous",
      kind: "InvestPause",
      decision: "REQUIRE_APPROVAL",
      reason_code: REASON.EXECUTION_CONFIRMED,
      reason: "REQUIRE_APPROVAL",
      mandate_version: 0,
      invest_mandate_version: 0,
      data_slot: 0,
      venue_id: 4,
      market_id: body.id ?? "pause",
      checks: [],
      tx_signature: null,
    });
    if (body.id) {
      const rule = cache.investRules.find((r) => r.id === body.id);
      if (rule) rule.status = "pause_requested";
      persist(cache);
    }
    return { env: env.name, decision: "REQUIRE_APPROVAL", receipt_id: receipt.request_id, note: "Pause is owner-signed." };
  });

  app.get("/approvals", async (req) => {
    const session = sessionOf(req, cache);
    const rows = cache.proposals.filter((p) => p.status === "pending" && (!session || p.actor === session.pubkey || p.actor === "mcp"));
    return { env: env.name, proposals: rows };
  });
  app.post("/approvals/:id/sign", async (req, reply) => {
    const session = sessionOf(req, cache);
    if (!session) return reply.code(401).send({ error: "SIWS required" });
    const id = (req.params as { id: string }).id;
    const p = cache.proposals.find((x) => x.id === id);
    if (!p) return reply.code(404).send({ error: "not found" });
    p.status = "signed";
    persist(cache);
    return { ok: true, id, status: "signed", note: "Marked signed in the control plane. On-chain set_mandate/invest_execute waits on program deploy." };
  });
  app.post("/approvals/:id/decline", async (req, reply) => {
    const session = sessionOf(req, cache);
    if (!session) return reply.code(401).send({ error: "SIWS required" });
    const id = (req.params as { id: string }).id;
    const p = cache.proposals.find((x) => x.id === id);
    if (!p) return reply.code(404).send({ error: "not found" });
    p.status = "declined";
    recordReceipt(cache, {
      request_id: randomUUID(),
      actor: session.pubkey,
      kind: p.kind === "invest_rule" ? "InvestPropose" : "MandateSet",
      decision: "REJECT",
      reason_code: REASON.ACTOR_SCOPE_DENIED,
      reason: "DECLINED",
      mandate_version: 0,
      invest_mandate_version: 0,
      data_slot: 0,
      venue_id: 0,
      market_id: "DECLINE",
      checks: [],
      tx_signature: null,
    });
    persist(cache);
    return { ok: true, id, status: "declined" };
  });

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
    maybeProbeProgram(cache).catch((err) => app.log.error(err));
  });
  const timer = setInterval(() => {
    refreshMarkets(cache).catch((err) => app.log.error(err));
    maybeProbeProgram(cache).catch((err) => app.log.error(err));
  }, 2500);
  app.addHook("onClose", async () => {
    clearInterval(timer);
  });

  return { app, cache };
}

async function confirmAndSubmit(
  req: { body: unknown; headers: { authorization?: string } },
  reply: { code: (n: number) => { send: (b: unknown) => unknown } },
  cache: Cache,
  envName: string,
) {
  const body = req.body as { request_id: string; signature: string };
  const session = sessionOf(req, cache);
  if (!session) return reply.code(401).send({ error: "SIWS required" });
  const pending = cache.pending.find((p) => p.request_id === body.request_id);
  const receipt = cache.receipts.find((r) => r.request_id === body.request_id);
  if (!pending || !receipt) return reply.code(404).send({ error: "not found" });
  if (pending.actor !== session.pubkey) return reply.code(403).send({ error: "actor mismatch" });
  if (!pending.compact_json || !pending.fields || pending.timestamp == null || pending.expiry_window == null) {
    return reply.code(400).send({ error: "no signable on this request" });
  }
  const sig = decodeSig(body.signature);
  const pk = bs58.decode(session.pubkey);
  const msg = new TextEncoder().encode(pending.compact_json);
  if (pk.length !== 32 || sig.length !== 64 || !nacl.sign.detached.verify(msg, sig, pk)) {
    return reply.code(401).send({ error: "bad signature" });
  }
  receipt.tx_signature = body.signature;
  const pacEnv = envName === "devnet" ? "testnet" : "mainnet";
  const submitted = await submitPacificaOrder(pacEnv, {
    account: session.pubkey,
    signature: body.signature,
    timestamp: pending.timestamp,
    expiry_window: pending.expiry_window,
    fields: pending.fields,
  });
  pending.venue_response = submitted.body;
  const ok = submitted.status >= 200 && submitted.status < 300 && (submitted.body as { success?: boolean }).success !== false;
  pending.status = ok ? "SUBMITTED" : "FAILED";
  receipt.reason = ok ? "SUBMITTED_TO_VENUE" : "EXECUTION_FAILED";
  receipt.reason_code = ok ? REASON.EXECUTION_CONFIRMED : REASON.EXECUTION_FAILED;
  receipt.decision = ok ? "ALLOW" : "REJECT";
  persist(cache);
  return {
    ok,
    receipt,
    pending,
    venue_status: submitted.status,
    venue_body: submitted.body,
    note: ok
      ? "Pacifica accepted the signed payload. This is a venue ack, not a guaranteed fill."
      : "Signature stored. Pacifica rejected or the account does not exist. Not a fill.",
  };
}

function activeMandate(cache: Cache, owner: string) {
  return (
    cache.mandates.find((m) => m.owner === owner) ?? {
      owner,
      version: 1,
      max_leverage_bps: 20_000,
      max_notional_usd: cache.env.caps.perAccountGrossNotionalUsd,
      min_safety_buffer_bps: 2_000,
      max_daily_loss_usd: 50,
      approved_markets: CANONICAL_MARKETS.map((m) => m.id),
      updated_at: "",
      on_chain: false as const,
    }
  );
}

async function maybeProbeProgram(cache: Cache) {
  if (Date.now() - cache.program.at < 15_000) return;
  cache.program.at = Date.now();
  const probe = await getAccountExecutable(cache.env.rpcHttp, cache.env.programId);
  cache.program.deployed = probe.deployed;
  cache.program.slot = probe.slot;
}

const hits = new Map<string, number[]>();
function rateOk(ip: string): boolean {
  const now = Date.now();
  const window = hits.get(ip)?.filter((t) => now - t < 60_000) ?? [];
  window.push(now);
  hits.set(ip, window);
  return window.length <= 120;
}

function idempotent(req: { headers: Record<string, unknown> }, cache: Cache) {
  const key = String(req.headers["idempotency-key"] ?? "");
  if (!key) return null;
  const hit = cache.idem.get(key);
  if (hit && Date.now() - hit.at < 24 * 3600_000) return hit;
  return null;
}
function remember(req: { headers: Record<string, unknown> }, cache: Cache, status: number, body: unknown) {
  const key = String(req.headers["idempotency-key"] ?? "");
  if (!key) return;
  cache.idem.set(key, { at: Date.now(), status, body });
}

function sessionOf(req: { headers: { authorization?: string } }, cache: Cache) {
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

if (import.meta.url === `file://${process.argv[1]}` || process.argv[1]?.endsWith("index.ts")) {
  const { app } = await buildServer();
  await app.listen({ port: PORT, host: HOST });
  console.log(`markov api ${STAGE} on ${HOST}:${PORT}`);
}
