# 12 — Data and API Spec
Markov Book · 31 August 2026 · v0.1

The read plane exists to make chain state legible in under ten seconds. It is a cache. It invents nothing. If Postgres is lost, the whole thing rebuilds from chain, and that rebuild is a tested runbook, not a hope.

---

## 1. Indexer

**Ingestion.** Two paths into one parser:

1. `logsSubscribe` on `PROGRAM_ID` at `confirmed` — the live path.
2. `getSignaturesForAddress` walk — startup backfill and gap repair, resuming from `last_indexed_signature`.

Both feed `parse(tx) -> Vec<Row>`, which decodes **Anchor CPI events by IDL**, never by string matching on log lines. A program upgrade that adds an event field must not break ingestion; an unknown discriminant is stored raw in `unparsed_events` and alerts, rather than being dropped.

**Commitment handling.** Rows land with `commitment='confirmed'`. A finalizer promotes them to `finalized` and stamps `finalized_at`. A row that disappears on reorg is marked `orphaned=true` and never deleted, so the feed's history is honest about what happened.

**Idempotency.** Primary key is `(signature, event_index)`. Re-ingesting a transaction is a no-op. The backfill can run any time without duplicating a receipt.

## 2. Schema

```sql
create table receipts (
  signature        text        not null,
  event_index      int         not null,
  kind             text        not null,          -- 'action' | 'refusal' | 'owner'
  seq              bigint,
  intent_id        bytea,
  mandate          text        not null,
  owner            text,
  operator         text,
  strategy_id      text        not null,          -- 'BOOK_ONE'
  venue            text,
  market           text,
  action           text,                          -- open|increase|reduce|close|flatten|fund|amend|pause|unpause|revoke|withdraw
  side             text,
  notional         numeric(38,0),
  fill_price       numeric(38,0),
  mark_price       numeric(38,0),
  mark_slot        bigint,
  spend            numeric(38,0),
  block_reason     text,                          -- null on allow
  gate_index       smallint,
  forced           boolean     not null default false,
  net_delta_e6     bigint,                        -- metadata, off-chain enforced (v0)
  gross_e6         bigint,
  slot             bigint      not null,
  block_time       timestamptz not null,
  commitment       text        not null default 'confirmed',
  finalized_at     timestamptz,
  orphaned         boolean     not null default false,
  ingested_at      timestamptz not null default now(),
  primary key (signature, event_index)
);
create index on receipts (strategy_id, block_time desc);
create index on receipts (mandate, block_time desc);
create index on receipts (block_reason) where block_reason is not null;

create table mandates (
  address text primary key, owner text, operator text, strategy_id text,
  state text, vault text, mint text, vault_balance numeric(38,0),
  policy jsonb, expiry_ts timestamptz, updated_slot bigint, updated_at timestamptz
);

create table ticks (                              -- agent-reported, clearly separated from chain truth
  tick_id text primary key, slot bigint, regime text, intent jsonb,
  verdict text, reason text, signature text, latency_ms int, created_at timestamptz
);

create table marks ( slot bigint primary key, price numeric(38,0), expo int, source text, observed_at timestamptz );

create table index_state (
  id int primary key default 1,
  last_indexed_slot bigint, last_indexed_signature text,
  last_ingest_ok timestamptz, parity_ok boolean, parity_checked_at timestamptz,
  parity_chain_count bigint, parity_db_count bigint
);

create table unparsed_events ( signature text, event_index int, raw bytea, seen_at timestamptz );
```

**Separation rule.** `receipts` and `mandates` are chain-derived. `ticks` and `marks` are house-reported. The API never mixes them in one response object without labelling the source, and the UI never renders a house-reported number in the same visual weight as a chain-derived one.

## 3. API

Base: `https://api.<domain>` · read-only · public · CORS open for GET · no auth · rate limited.

| Endpoint | Returns |
|---|---|
| `GET /health` | `{chainReady, last_indexed_slot, rpc_slot, lag_slots, last_ingest_ok, parity_ok, failing:[...]}` |
| `GET /v1/receipts?strategy=BOOK_ONE&mandate=&reason=&limit=50&cursor=` | receipts page, newest first, cursor-paginated on `(block_time, signature)` |
| `GET /v1/receipts/stats?window=24h` | `{actions, refusals, refusal_rate, by_reason:{...}, forced_share}` |
| `GET /v1/book/stats` | `{net_delta_usd, gross_usd, funding_7d, tvl_in_strategy, actions, refusals, circuit, mark:{price,slot,age,source}, enforcement:{delta:"offchain",gross:"offchain",daily_loss:"offchain"}}` |
| `GET /v1/mandates/:address` | mandate state, policy, vault balance, `withdraw_enabled: true` always |
| `GET /v1/paper` and `/v1/paper/:date` | the daily markdown, served as-is |
| `GET /v1/facts` | selected public rows from FACTS: program ids, cluster, paper start date |

**Response rules.**
- Every money field carries its `mint`, `decimals`, and raw base-unit integer alongside any formatted value. No float arithmetic in the API.
- Every derived field carries `source: "chain" | "house"`.
- `circuit` is one of `live | paused | revoked | daily_loss_halt | global_halt | stale_mark`.
- There is no `apy` field, no `apr` field, and no `projected_*` field in the schema. Absence is the enforcement.
- Errors are `{error: {code, message}}` with plain-language messages; the UI shows them verbatim.

## 4. `chainReady` and the parity job

```
chainReady = lag_ok && ingest_ok && parity_ok
lag_ok     = (rpc_slot_confirmed - last_indexed_slot) <= LAG_SLOTS      // default 150
ingest_ok  = now - last_ingest_ok < 30s
parity_ok  = last parity run matched within tolerance 0
```

**Parity job**, every 5 minutes and on demand in CI:

1. Independently walk `getSignaturesForAddress(PROGRAM_ID)` for the last 24h.
2. Count receipts by decoding events from those transactions — a second code path, not the indexer's.
3. Compare with `select count(*) from receipts where block_time > now() - 24h and not orphaned`.
4. Mismatch → `parity_ok=false`, alert, and `/health` names `parity` in `failing`.

This is the mechanism behind B9's "counts match an independent chain query". It is also the thing that stops the dashboard from confidently displaying a number that has quietly drifted.

## 5. Rebuild runbook (tested, not theoretical)

```
1. Stop indexer.  2. Truncate receipts, mandates, marks, index_state.
3. Start indexer with BACKFILL_FROM=<program deploy slot>.
4. Wait for lag_ok.  5. Run parity for the full range, not 24h.
6. Compare receipt count with the pre-drop count from the previous parity row.
```

Run this once before Gate B closes and record the wall-clock time in `docs/runbooks/indexer-rebuild.md`. A rebuild path nobody has executed is not a rebuild path.

## 6. Retention and privacy

- Receipts: kept forever. They are the product.
- `ticks`: 90 days. They are diagnostics.
- No IP logging, no wallet analytics, no cookies on the read API. A stranger auditing the book should not have to be tracked to do it.
- Raw instruction payloads are never stored (ADR-10). If a field is not on the event, the API does not know it.
