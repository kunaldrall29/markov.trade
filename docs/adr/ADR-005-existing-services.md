# ADR-005 — The existing TypeScript services vs. the pack's Rust workspace (decision D4)

Status: **Proposed — needs Kunal** · Date: 2026-09-01 · Seat: Protocol + Agents + Truth · Blocks: P07, P09, P10, P13 scoping

## What exists (verified 2026-09-01)

`kunaldrall29/markov` is a bun/TypeScript monorepo with `apps/{api, agents, bot, data-api, indexer, web, site}` and `packages/{engine, operator, rpc, sdk}`. Railway project `markov` runs `api`, `agents`, `bot`, `data-api`, `indexer` and Postgres; all report healthy. Live public feed: `https://data-api-production-5ac5.up.railway.app/v1/receipts` and `/v1/receipts/stats` (41 receipts, 11 reason keys). Telegram `@markov_float_bot` is attached to the `bot` service.

Against the pack's `docs/12` and `docs/07`:

| Pack requirement | Existing stack |
|---|---|
| Rust workspace, shared types with the program (`07` §4) | TypeScript; `BlockReason` list duplicated by hand in `packages/engine` and tested against the IDL |
| Decode events by IDL, `(signature, event_index)` PK | Decodes by IDL via `@coral-xyz/anchor` EventParser; unique index on `(sig, event_index)` — close |
| `chainReady = lag && ingest && parity` | `chainReady = lag` only; **no parity job, no finalizer, no orphan marking, no `unparsed_events`** |
| Indexer live via `logsSubscribe` + backfill | polls; WebSocket on public devnet 429s; **stalled since ≈ 29 Aug at lag 1.2M slots, `chainReady:false`** |
| Endpoints `/health /v1/receipts /v1/receipts/stats /v1/book/stats /v1/mandates/:a /v1/paper /v1/facts` | has `/health`, `/v1/receipts`, `/v1/receipts/stats`, `/price/:symbol`; **no** book stats, mandates, paper, facts |
| Money fields `{raw, decimals, mint}`, `source: chain\|house` on every derived field | flat `amount`, symbol strings (`USDC-d`), no `source` |
| Emergency key in its own service (`14` §2) | `EMERGENCY_KEY_JSON` is an env var of the **`api`** service; the bot calls the API |
| Operator key only in the agent service | `agents` has no key; the `api` signs house-operator actions from `keys/*.json` on disk |
| Receipts schema `docs/12` §2 (mandate, owner, operator, strategy_id, venue, market, action, side, notional, fill/mark price, mark_slot, spend, block_reason, gate_index, forced, …) | `receipts(mandate_id, kind, refused, reason, nonce, sig, ts, strategy_id, operator, venue, token, amount, action_type, event_index)` — no `gate_index`, `forced`, `mark_*`, `side`, `market`, `owner`, commitment/finalized/orphaned |

## Options

**A. Follow the pack: new Rust workspace; retire the TS services after Gate B closes.** Shared types with the program are a compile-time guarantee; the parity job, finalizer and fail-closed `chainReady` are built as specified; key separation is deployment-level from day one. Cost: `book-one`, `indexer`, `data-api`, `bot` are all new code inside 26 days, in parallel with the successor program (ADR-004). The existing Railway project can host them, but every service is replaced.

**B. Fork the TS services into this repo and bring them up to `docs/12`.** Faster start for indexer/data-api (hosting, Dockerfiles, Postgres already work). Cost: violates `07` §4 (a documented architectural decision), the guard/`BlockReason` mirror stays a hand-copied list, the key-custody layout must be rebuilt anyway, and the missing parity/finalizer/`source`/`{raw,decimals,mint}` work is most of the spec regardless. Also drags `@solana/web3.js` v1 and `@coral-xyz/anchor` 0.31 (npm stuck at 0.32.1) into the new tree.

**C. Rust for everything that touches a key or a `BlockReason` (`book-one`, `markov-guard`, `bot`); reuse and extend the TS `indexer` + `data-api` for Gate B only, replaced in Gate C.** Cuts the Rust surface roughly in half where shared types matter least. Cost: two languages in one repo and two package managers plus bun; `12` §2 schema still has to be rewritten; the parity job must be a second code path anyway.

## Recommendation

**A**, because the pack's reasons for Rust are about correctness of the proof surface (guard ↔ program types, fail-closed `chainReady`, key separation), which is exactly what Gate B grades, and because the existing indexer is stalled and lacks the parts that make B9 true. The existing Railway project and Postgres are reused as **infrastructure** (new services, separate env scopes), the existing `data-api` URL is kept alive as-is until the Rust one answers, and nothing from the TS tree is copied except facts.

If Kunal wants B or C, `docs/07` §4 needs a superseding ADR first, and the calendar in ADR-006 changes.

## Consequences

The existing services are treated as read-only evidence and as a source of devnet facts (mints, pools, operator pubkeys). No Gate B code imports from `kunaldrall29/markov`. The Telegram bot token and the `2fpQ…` deployer stay where they are until the new services are ready to take over, and the emergency key for Gate B is a **new** keypair generated per `docs/14` §5.
