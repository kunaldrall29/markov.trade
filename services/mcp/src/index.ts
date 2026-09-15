/**
 * MCP server: read / simulate / propose. Never execute, withdraw, or set a mandate.
 * Thin client of the control plane.
 */
import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import { z } from "zod";

const API = process.env.MARKOV_API_URL || "http://127.0.0.1:4000";

const ALLOWED = [
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
] as const;

const ABSENT = [
  "markov.withdraw",
  "markov.execute",
  "markov.set_mandate",
  "invest.execute",
  "invest.withdraw",
  "invest.set_allowlist",
] as const;

async function api(path: string, init?: RequestInit) {
  const res = await fetch(`${API}${path}`, {
    ...init,
    headers: { accept: "application/json", "content-type": "application/json", ...(init?.headers || {}) },
  });
  const json = await res.json();
  return { status: res.status, json };
}

function text(obj: unknown) {
  return { content: [{ type: "text" as const, text: JSON.stringify(obj, null, 2) }] };
}

export function allowedTools(): readonly string[] {
  return ALLOWED;
}
export function absentTools(): readonly string[] {
  return ABSENT;
}

export function createMcp() {
  const server = new McpServer({ name: "markov", version: "0.1.0" });

  server.tool("markov.get_account_state", "Mandate, permissions summary, positions, headroom. Read only.", {}, async () =>
    text(await api("/account")),
  );
  server.tool("markov.get_risk", "Normalized risk state. Read only.", {}, async () => text(await api("/risk")));
  server.tool("markov.get_markets", "Canonical markets with venue availability.", { category: z.string().optional() }, async () =>
    text(await api("/markets")),
  );
  server.tool(
    "markov.compare_routes",
    "Rank eligible venues by lifecycle cost in bps.",
    { market: z.string(), side: z.enum(["long", "short"]), notional_usd: z.number(), horizon: z.number().optional() },
    async ({ market, notional_usd, horizon }) =>
      text(await api("/routes/compare", { method: "POST", body: JSON.stringify({ market, notional_usd, horizon_hours: horizon ?? 24 }) })),
  );
  server.tool(
    "markov.simulate_trade",
    "Projected state and policy checks. Does not place an order.",
    { market: z.string(), side: z.enum(["long", "short"]), notional_usd: z.number(), leverage: z.number().optional(), venue: z.string().optional() },
    async (args) => text(await api("/risk/simulate-trade", { method: "POST", body: JSON.stringify(args) })),
  );
  server.tool(
    "markov.request_trade",
    "Creates a pending owner approval or a rejection receipt. Never returns a signable to the model.",
    { market: z.string(), side: z.enum(["long", "short"]), notional_usd: z.number(), leverage: z.number(), client_note: z.string().optional() },
    async (args) => {
      const r = await api("/trades/request", { method: "POST", body: JSON.stringify(args) });
      const json = r.json as { signables?: unknown };
      delete json.signables;
      return text({ ...r, approval_required: true, signable_for_model: false });
    },
  );
  server.tool("markov.get_receipts", "Receipts with reason codes.", { limit: z.number().optional() }, async () =>
    text(await api("/receipts")),
  );
  server.tool("markov.explain_receipt", "Checks table as text plus raw receipt.", { request_id: z.string() }, async ({ request_id }) =>
    text(await api(`/receipts/${request_id}`)),
  );
  server.tool("markov.reduce_position", "Owner-signed reduce request. Models cannot sign.", {
    market: z.string(),
    notional_usd: z.number(),
  }, async (args) => text(await api("/trades/reduce", { method: "POST", body: JSON.stringify(args) })));
  server.tool("invest.get_rules", "Active invest mandate.", {}, async () => text(await api("/invest/rules")));
  server.tool("invest.get_history", "Invest receipts.", {}, async () => text(await api("/invest/history")));
  server.tool(
    "invest.simulate_rule",
    "Would this invest action pass. Does not spend.",
    { mint: z.string(), usd: z.number() },
    async (args) => text(await api("/invest/simulate", { method: "POST", body: JSON.stringify(args) })),
  );
  server.tool(
    "invest.propose_rule",
    "Creates a pending proposal. Owner must sign. Models cannot execute.",
    { mint: z.string(), usd_per_period: z.number(), period_seconds: z.number() },
    async (args) => {
      const r = await api("/invest/propose", { method: "POST", body: JSON.stringify(args) });
      const json = r.json as { signables?: unknown };
      delete json.signables;
      return text({ ...r, signable_for_model: false });
    },
  );
  server.tool("invest.pause_rule", "Owner must confirm pause.", { id: z.string() }, async () =>
    text(await api("/invest/pause", { method: "POST", body: JSON.stringify({}) })),
  );
  server.tool("invest.quote_swap", "Spot quote. Mainnet only; skips on devnet.", { mint: z.string(), usd: z.number() }, async (args) =>
    text(await api("/invest/quote", { method: "POST", body: JSON.stringify(args) })),
  );

  return server;
}

if (import.meta.url === `file://${process.argv[1]}` || process.argv[1]?.endsWith("index.ts")) {
  const server = createMcp();
  const transport = new StdioServerTransport();
  await server.connect(transport);
}
