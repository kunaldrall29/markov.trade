/** Hand-written OpenAPI 3.1 for the account API. Terminal and MCP consume the same paths. */

export const OPENAPI = {
  openapi: "3.1.0",
  info: { title: "Markov control plane", version: "0.1.0" },
  paths: {
    "/health": { get: { summary: "Liveness, venue cache sizes, program deploy bit" } },
    "/markets": { get: { summary: "Canonical markets with live venue rows" } },
    "/markets/{id}": { get: { summary: "Venue rows for one market" } },
    "/markets/{id}/depth": { get: { summary: "Live books" } },
    "/markets/{id}/candles": { get: { summary: "Pacifica kline only" } },
    "/policy/check": { post: { summary: "Mandate checks, no side effects" } },
    "/routes/compare": { post: { summary: "Lifecycle cost ranking" } },
    "/trades/request": { post: { summary: "Receipt + owner signable" } },
    "/trades/submit": { post: { summary: "Relay signed Pacifica payload" } },
    "/receipts": { get: { summary: "Decision log" } },
    "/invest/propose": { post: { summary: "Pending invest rule; models never receive a signable" } },
    "/approvals": { get: { summary: "Pending owner approvals" } },
    "/auth/challenge": { post: { summary: "SIWS challenge" } },
    "/auth/verify": { post: { summary: "SIWS verify" } },
  },
};
