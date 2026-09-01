# P09 — Indexer
Seat: Protocol · Window: 14–20 Sep · Inherits `P00-conventions.md`

## Goal
Program logs become rows in under five seconds, by IDL decoding, with a second independent code path that checks the first. Half of **B9**.

## Pre-flight (STOP and report if any fails)
1. `logsSubscribe` works on your RPC plan for `PROGRAM_ID`, and you know the subscription limit.
2. You can decode an Anchor CPI event from a real devnet transaction produced by P02 — paste the decoded struct.
3. Postgres is reachable from the Railway private network with two roles: read-write for the indexer, read-only for the API.

## Deliverables
- Ingestion behind a trait with two implementations: `ws_logs` (live) and `signature_backfill` (startup + gap repair). A provider gRPC stream can be added later without touching the parser.
- Parser: IDL-based event decoding. Unknown discriminants go to `unparsed_events` and alert — never dropped, never guessed.
- Schema exactly as `docs/12-DATA-AND-API-SPEC.md` §2, with `(signature, event_index)` as the primary key so re-ingestion is a no-op.
- Finalizer promoting `confirmed` → `finalized`; reorged rows marked `orphaned`, never deleted.
- `index_state` maintained: `last_indexed_slot`, `last_indexed_signature`, `last_ingest_ok`.
- **Parity job** every 5 minutes: an independent signature walk + decode, counted against the DB, writing `parity_ok`, `parity_chain_count`, `parity_db_count`.
- `docs/runbooks/indexer-rebuild.md`, executed once for real, with the wall-clock time recorded.

## Hard constraints
- No raw instruction payloads stored (ADR-10).
- The parity job must not share code with the indexer's parser, or it is checking itself.
- `data/ledger.json` or any local file is never a source. If the DB is empty, the answer is "rebuild from chain", not "read the file".

## Acceptance
- A receipt appears in Postgres within 5s of confirmation, measured over 20 events.
- Killing the indexer for 10 minutes and restarting recovers every missed event via backfill.
- Parity job green; then delete one row by hand and prove it goes red.
- Rebuild runbook executed end-to-end.

## Evidence
Latency histogram, backfill recovery proof, deliberate-mismatch demonstration, rebuild timing.
