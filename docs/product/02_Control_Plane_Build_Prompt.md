# 02 — Control Plane (API, engines, adapters, keeper) — Build Prompt

Inherit `00_Build_Conventions.md`. Language: TypeScript (Node 22), Fastify, Postgres 16 (Supabase-hosted is acceptable; migrations in repo), Redis for queues, `@solana/kit` for RPC, `@drift-labs/sdk` stable, `@ellipsis-labs/rise` for Phoenix reads, Pacifica via signed REST/WS, Jupiter via REST. Every adapter has a mainnet implementation; test doubles are forbidden outside `packages/adapters/**/__tests__` and even there must replay recorded real responses.

## 1. Services

| Service | Responsibility |
| --- | --- |
| `services/api` | HTTP + WS control plane; auth; policy/risk/router orchestration; receipts; account API |
| `services/keeper` | Invest keeper (due-rule evaluation, pull + swap + deliver), reconciliation workers per venue, canary rule |
| `services/mcp` | separate prompt (`03`) |
| workers in `api` | market data ingest per venue, official-price ingest, canonical registry sync, alerting |

## 2. Data model (Postgres; every table has `created_at`, `updated_at`)

`accounts` (owner, markov_account_pda, execution_mode, status, active versions) · `mandates` / `invest_mandates` (mirrors of on-chain, plus soft preferences) · `permissions` · `actors` (MCP clients, API keys, keepers; hashed secrets) · `canonical_assets` (canonical_asset_id, category, risk_tier, tier_reasons) · `venue_markets` (canonical id ↔ venue symbol, contract multiplier, token program, extensions, max leverage, isolated_only, status, liquidity floor, last seen) · `market_state` (per venue/market: mark, index, funding, depth ±10/50 bps, freshness, session status) · `official_prices` (asset, source, price, market_status, ts) · `requests` (request_id, actor, kind, payload, state machine state) · `decisions` (policy result, checks) · `routes` (comparison snapshot per request) · `receipts` (mirror of chain receipts + off-chain-only skips, tx signature, explorer url, reconciled) · `positions` / `holdings` (normalized; raw + scaled amounts) · `invest_rules` (compiled from invest mandates: schedule, mode, window, next_due) · `keeper_runs` · `venue_accounts` (links, health, last reconciled slot) · `alerts` · `evidence` (rehearsal artefacts).

All money as `numeric(30,0)` micro-USD; all token amounts as raw `numeric(40,0)` with a separate `ui_multiplier` column for scaled-UI mints.

## 3. Auth

- Wallet sign-in: challenge (nonce, domain, expiry) → `signMessage` → session JWT (15 min) + refresh (7 days, rotating), bound to owner pubkey.
- Actors: MCP clients and API keys get bearer tokens scoped to a `permissions` row; every write checks scope and caps **before** the policy engine runs.
- Rate limits per actor; idempotency keys on every write (`Idempotency-Key` header = `request_id`).

## 4. Policy engine (`packages/policy`)

Pure function `check(action, accountState, mandate, permission, marketState, officialPrice, now) → Decision`. Evaluates in this order and short-circuits only on hard failures: global pause → account pause → actor scope → actor caps → venue allowed → market allowed → data freshness → projected post-trade leverage → projected notional → projected safety buffer → daily loss → slippage/cost → (Invest) allowlist, budget, ceiling, reserve, cost, execution mode/market status, single-asset cap. Output: `{decision: ALLOW|REJECT|REQUIRE_APPROVAL|SKIP, reason_code, checks[]}` with `checks[]` carrying observed vs limit for every rule evaluated, in the same encoding the program expects. The API never invents a decision; the program re-validates the same observed values on-chain.

## 5. Risk engine (`packages/risk`)

Normalizes each venue's account state into `NormalizedRiskState` (equity, gross notional, net delta, effective leverage, venue health, normalized safety buffer = distance to venue liquidation as % of notional using the venue's own liquidation math, daily loss, headroom per rule, data slot). Provides `simulateTrade` and `simulateScenario` (price shocks, funding spikes, venue halt). Venue liquidation math is taken from each venue's SDK/docs, never approximated: Drift via SDK margin calculations; Pacifica via account-info fields; Phoenix via Hawkeye view. If a venue does not expose a field the engine needs, the field is `null` and any rule depending on it fails closed with `STALE_MARKET_DATA`.

## 6. Router (`packages/router`)

Input: eligible venues (from adapters' capability + freshness + account link), notional, side, expected holding horizon. Per venue: entry slippage from live depth at the notional, taker fee, expected funding over horizon from the venue's current and recent funding, exit cost estimate, venue risk premium (health/freshness). Output: ranked routes with component breakdown in bps, `selected`, and a one-line reason. Manual venue pin re-runs policy; a REJECT stays REJECT. Never reuse a route older than the freshness window. No routing to a venue where the account has no linked, funded venue account.

## 7. Venue adapters (`packages/adapters`) — one interface, real implementations

```
interface PerpVenueAdapter {
  id, env, capabilities(): { execution_model, order_types, margin_modes, delegation_model, onchain_enforceable, freshness_ms }
  markets(): VenueMarket[]                 // dynamic; mapped to canonical ids via registry
  marketState(symbol): MarketState         // mark, index, funding, next funding, depth, ts
  accountState(account): VenueAccountState // balances, positions, margin, health, ts
  quote(req): Quote                        // executable estimate incl. fees, slippage at size
  buildOpen/Modify/Close/Cancel(req): Signable[]  // tx (Solana) or message (Pacifica) for the owner to sign
  submit(signed): SubmitResult             // lands/relays; returns venue id + tx signature
  reconcile(account): ReconcileResult      // authoritative post-state
  funding/marginState/liquidationState(...)
}
```

**Pacifica (mainnet + testnet):** REST `GET /info`, `/info/prices`, orderbook, funding history; WS for prices/book/account. Writes are Ed25519-signed deterministic JSON per Pacifica's signing spec; the adapter returns the exact bytes to sign and a human-readable `display`. Owner signs with `signMessage`; if the wallet cannot sign messages, refuse (no fallback). Agent keys only if the owner bound one in the Terminal; their scope is recorded from a real withdrawal attempt on testnet (gate P2) before any mainnet use.
**Drift (mainnet + devnet):** SDK `DriftClient` in read-only mode for state; transaction builders for deposit, place/cancel, close; delegate mode not used in this build. Subaccount 0 only.
**Phoenix (mainnet, read + rehearsal):** Rise TS client for markets, orderbook, mark, funding, trader state; Hawkeye views for margin/liquidation. Execution is enabled only for the **rehearsal wallet** after gate P1 (fresh-wallet onboarding via `build-register-ixs`) passes; the adapter's `capabilities().executable` is `false` for all other accounts until the capped mainnet stage, and the Terminal shows `INTEGRATED · read-only`.
**Jupiter spot (mainnet):** quote → policy cost check → swap instructions → transaction assembled by Markov with the Token-2022 output ATA (create idempotently with the Token-2022 program), transfer-hook accounts resolved, priority fee from recent fees, simulated before signing; post-fill balance read uses raw amount and `scaledUiAmountConfig` multiplier. Route labels recorded; `RFQ`/`AMM` tag only if the route data identifies it (gate I7), otherwise omitted.

Each adapter ships with a **recorded-response test suite** (real responses captured at a slot/timestamp, replayed) and a **live smoke test** (`pnpm smoke:<venue>`) that hits the real endpoint and asserts shape and freshness.

## 8. Canonical registry

`canonical_assets` + `venue_markets` populated from live venue market lists (Phoenix `/v1/view/exchange/markets`, Pacifica `/info`, Drift on-chain `PerpMarket` accounts) and, for spot, from a pinned allowlist verified against Jupiter Tokens API `isVerified` + tag and on-chain mint extensions (RPC `getAccountInfo` jsonParsed). Contract multipliers (kBONK / 1MBONK / 1KPUMP) and symbol aliases (GOLD/XAU/PAXG as distinct canonical ids) are explicit fields. Registry sync runs hourly; a market that disappears from a venue is marked `delisted` and blocked from new positions immediately.

## 9. Official price and calendar

Ingest Chainlink Data Streams US-equities (regular-hours feeds) and/or Pyth equity feeds on Solana for allowlisted names; store price + `market_status` + timestamp. Maintain the US exchange calendar (holidays, early closes). Expose `getReference(asset) → {price, status, age}`. Reference Safe and Market Hours Only modes read this; if the feed is absent for an asset, Reference Safe is unavailable for that asset (UI says so) — never approximated.

## 10. Invest keeper (`services/keeper`)

Loop every 60 s: for each due `invest_rule` → build action → `policy.check` → if ALLOW: build one transaction `[compute budget, markov.invest_execute, subscriptions.collect(pull USDC to keeper vault ATA), jupiter swap ixs, transfer output to user ATA (if the swap can't deliver directly), markov.finalize_decision]` → sign with KMS → send with retries and blockhash refresh → confirm → reconcile → receipt. If SKIP/REJECT: `markov.record_skip` on-chain (batched per run to save fees) + off-chain receipt. Handles `DELEGATION_INSUFFICIENT` by pre-checking the delegation state 24 h before due time and alerting the owner. Retry-within-window then skip. All amounts and mints from the compiled rule; nothing computed from symbols. Canary rule: one $5/week rule on the rehearsal wallet, alert if a cycle produces no receipt.

**Gate before mainnet keeper runs:** I2 (Subscriptions `collect` semantics and atomicity with the swap in one transaction) and I3 (Jupiter swap-instructions composition with Token-2022 output) proven on mainnet with the rehearsal wallet at $5.

## 11. Reconciliation workers

Per venue and per account: pull authoritative state every 30 s (or on WS event), diff against expected post-state, mark receipts `reconciled` or `mismatch`, raise `RECONCILE_MISMATCH` alert and set the venue account `degraded` (keeper automation paused for that account) on mismatch.

## 12. API surface (must match `docs/ui-contract.md` and `packages/sdk`)

REST: `/account`, `/portfolio`, `/portfolio/history`, `/positions`, `/holdings`, `/markets`, `/markets/{id}`, `/markets/{id}/candles|depth|funding-history`, `/venues/capabilities`, `/venues/accounts`, `/venues/{venue}/link`, `/risk`, `/risk/history`, `/risk/simulate-trade`, `/risk/simulate-scenario`, `/policy/check`, `/routes/compare`, `/routes/quote`, `/trades/request`, `/trades/submit`, `/trades/{id}/status`, `/positions/reduce`, `/orders/cancel`, `/mandate`, `/mandate/versions`, `/mandate/pause|unpause`, `/invest/rules`, `/invest/rules/{id}`, `/invest/simulate`, `/invest/propose`, `/invest/history`, `/receipts`, `/receipts/{id}`, `/mcp/tools|tokens|approvals`, `/alerts`, `/health`.
WS: `portfolio`, `positions`, `risk`, `market:{id}:{venue}`, `trade:{request_id}`, `invest:{rule_id}`, `receipts`, `venues:health`.
Every response carries `data_slot` or `venue_ts` and `env`. OpenAPI 3.1 generated from zod schemas; the Terminal and MCP import the same schemas.

## 13. Testing (real)

- Unit: policy/risk/router with slot-labelled real fixtures; property tests on policy.
- Adapters: recorded-response replay + live smoke against mainnet read endpoints (no writes) in CI.
- Integration nightly: Drift devnet open/close with a funded devnet wallet; Pacifica testnet open/close; Subscriptions devnet create/collect/revoke; keeper cycle on devnet with `devnet_swap_stub`.
- Rehearsal (mainnet, capped): every flow in `07_Mainnet_Test_Plan.md`, evidence saved.
- Load: 50 concurrent policy checks < 50 ms p95; WS fan-out 1,000 clients.

## 14. Out of scope

Delegated perp execution, Phoenix PDA gating, cross-user netting, solver interface implementation beyond the intent schema, Markov Markets.
