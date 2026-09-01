# P10 — `data-api`
Seat: Protocol · Window: 15–21 Sep · Inherits `P00-conventions.md`

## Goal
A narrow, honest read API whose `chainReady` is computed, not asserted. The other half of **B9**.

## Pre-flight
1. Indexer is writing and the parity job is green.
2. The API's Postgres role is read-only — prove it by attempting an insert and showing the failure.

## Deliverables
Endpoints exactly as `docs/12-DATA-AND-API-SPEC.md` §3: `/health`, `/v1/receipts`, `/v1/receipts/stats`, `/v1/book/stats`, `/v1/mandates/:address`, `/v1/paper[/:date]`, `/v1/facts`.

- `chainReady = lag_ok && ingest_ok && parity_ok`, with a `failing: []` array naming which term broke.
- Cursor pagination on `(block_time, signature)`. No offset pagination.
- Every money field returns `{raw, decimals, mint}`; no floats anywhere in the response path.
- Every derived field carries `source: "chain" | "house"`.
- `enforcement: {delta: "offchain", gross: "offchain", daily_loss: "offchain"}` on `/v1/book/stats` so the UI can label it without guessing.
- Rate limiting, CORS for GET, structured errors `{error:{code,message}}` in plain language.

## Hard constraints
- The schema has no `apy`, `apr`, or `projected_*` field. Add a test that greps the OpenAPI output for those strings and fails.
- The API never computes a receipt it did not read. No synthesised rows, no "estimated" fills.
- `withdraw_enabled` on the mandate endpoint is a literal `true`, with a comment explaining that it is an invariant, not a computed value.

## Acceptance
- `curl /health` shows real numbers; stopping the indexer flips `chainReady` to false within 60s and names `ingest`.
- `/v1/receipts/stats` count matches an independent `solana` CLI signature count for the same window → **B9**.
- p95 latency under 200ms for `/v1/receipts?limit=50`.

## Evidence
The two counts side by side, the degraded `/health` payload, and the latency measurement.
