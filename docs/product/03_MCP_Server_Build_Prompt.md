# 03 — Markov MCP Server — Build Prompt

Inherit `00_Build_Conventions.md`. Stack: TypeScript, `@modelcontextprotocol/sdk` (Streamable HTTP transport), OAuth 2.1 with PKCE for client authorization, tokens bound to a `permissions` row in the control plane. The server is a thin, deterministic client of the control plane API: it has no model-specific logic, no prompts, no memory of its own.

## 1. Principle

Any model can read, simulate and propose. No model can execute a perp trade, execute an invest purchase, withdraw, change a mandate, or change an allowlist. Proposals land in the owner's approvals inbox; the owner signs in the Terminal or rejects; every proposal and its outcome is a receipt attributed to the client's subject.

## 2. Authorization

- Clients register (name, redirect URI, requested scopes); the owner approves in the Terminal, which creates a `permissions` row on-chain (`set_permission`) and off-chain with scopes ⊆ {`account:read`, `risk:simulate`, `trade:request`, `trade:reduce`, `invest:propose`}, per-action and daily caps, expiry ≤ 30 days.
- Token introspection on every call; revoked or expired → `401`. Scope missing → `403` with reason `ACTOR_SCOPE_DENIED` **and** a receipt.
- No `invest:execute`, no `collateral:withdraw` scope exists for MCP clients; the server rejects any request for them at registration.

## 3. Tools (exact names; inputs/outputs are the zod schemas from `packages/sdk`)

**Perps**
- `markov.get_account_state` (account:read) → mandate, permissions summary, positions, headroom, `data_slot`.
- `markov.get_risk` (account:read) → `NormalizedRiskState`.
- `markov.get_markets` (account:read; `{category?, venue?}`) → canonical markets with availability, tier, ceiling.
- `markov.compare_routes` (risk:simulate; `{market, side, notional_usd, horizon}`) → routes with bps components, selected, reason, freshness.
- `markov.simulate_trade` (risk:simulate; `{market, side, notional_usd, leverage?, venue?}`) → projected state, policy checks, liquidation estimate.
- `markov.request_trade` (trade:request; same input + `client_note`) → `{request_id, decision, checks, approval_required: true}`; on ALLOW creates a pending approval, on REJECT returns the reason and a receipt id. Never returns a signable payload to the model.
- `markov.reduce_position` (trade:reduce) → same shape, reduce-only.
- `markov.get_receipts` (account:read; filters) → receipts.
- `markov.explain_receipt` (account:read; `{request_id}`) → the checks table rendered as text + the raw receipt.

**Invest**
- `invest.get_rules` (account:read) · `invest.get_history` (account:read) · `invest.simulate_rule` (risk:simulate) · `invest.propose_rule` (invest:propose) → `{proposal_id, decision, checks, signable_for_owner: false}` · `invest.pause_rule` (invest:propose) → proposal · `invest.quote_swap` (risk:simulate) → route cost, impact, source tag if identifiable.

Tools that must not exist and are asserted absent by a test: `markov.withdraw`, `markov.execute`, `markov.set_mandate`, `invest.execute`, `invest.withdraw`, `invest.set_allowlist`.

## 4. Behaviour

- Every tool result includes `env`, `data_slot|venue_ts`, `freshness_ms`, and, for decisions, `receipt_id`.
- Determinism: same inputs at the same slot produce the same outputs; no randomness, no model calls inside the server.
- Tool descriptions follow the claims policy: no "best", no "optimal", no "safe".
- Idempotency: `request_id` supplied by the client or generated once per call and returned; repeats return the original result.
- Rate limits per client (per minute and per day); proposal budget per day.
- Approvals: `GET /mcp/approvals` in the control plane; the Terminal inbox subscribes via WS; the owner's decision (sign/decline) produces the final receipt, attributed to both owner and client.

## 5. Security

- OAuth tokens are opaque, short-lived, rotated; refresh requires the original client secret; PKCE required.
- Input validation with the shared zod schemas; reject unknown fields.
- Audit log of every call (client, tool, inputs hash, result decision, latency) retained 1 year.
- Prompt-injection posture: the server treats all tool inputs as untrusted data; nothing a model sends can widen its own scopes, extend expiry, or reference another account.

## 6. Tests

- Contract tests against the live control plane on **rehearsal** (mainnet) with a real Markov account and a real client token: read tools return live state; `request_trade` within mandate yields a pending approval and a receipt; `request_trade` exceeding leverage yields `REJECT · MAX_LEVERAGE_EXCEEDED` and a receipt; `invest.propose_rule` exceeding the monthly ceiling yields `REJECT · MONTHLY_CEILING_EXCEEDED`.
- Absent-tool test; scope-denial test; expiry test; idempotency test.
- Client walkthroughs with Claude Desktop, Claude Code and Cursor recorded as evidence (screen capture + receipt ids).

## 7. Deliverables

`services/mcp` with Dockerfile; `docs/mcp.md` (setup for Claude Desktop/Code, Cursor; scopes; approval flow; every tool with schema); registration UI hooks in the Terminal (`/integrations`).
