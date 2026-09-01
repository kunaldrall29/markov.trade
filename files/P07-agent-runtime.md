# P07 — `book-one` runtime: scheduler, sidecar stub, submitter, redteam
Seat: Agents · Window: 10–20 Sep · Inherits `P00-conventions.md`

## Goal
The hosted agent that produces **B3, B4, B5, B7**: it ticks, it mostly skips, it lands one real action, and it deliberately gets told no on chain.

## Pre-flight (STOP and report if any fails)
1. Three distinct keypairs exist; their **public** keys are in FACTS; the operator key is the only one in this service's environment. Prove the emergency key is not readable here.
2. The demo mandate exists on devnet, is funded, and its policy matches the Gate B template.
3. `markov-guard` and `book-core` tests are green.
4. RPC primary and fallback both respond; you can send and confirm a transaction.

## Deliverables
- `scheduler`: tick every `TICK_SECONDS` (floor 60 — the process **refuses to boot** below it), jitter 0–5s, exponential backoff on error.
- `state`: chain reads with a hard timeout; any failure → `Skip` with a logged reason.
- `sidecar`: `RegimeSource` trait; Gate B implementation is a stub returning `Chop`. It is behind the trait so nothing downstream knows it is a stub.
- `submitter`: builds, signs with the operator key, sends, confirms at `confirmed`, records the signature; retries 3× with the same `intent_id`; treats `DuplicateIntent` as already-done; **never** widens a parameter to make a tx land.
- `redteam`: the schedule in `docs/11-AGENT-SPEC.md` §6. Forced intents carry `forced=true`, bypass only the *local* veto, and are refused by the *program*.
- `metrics` + one JSON log line per tick with `tick_id`.
- Kill switches: `MAX_ACTIONS_PER_HOUR` halts the agent; a `HALT` env or file stops submissions without stopping ticks (so the log keeps showing why).
- Railway service definition, health endpoint, restart policy.

## Hard constraints
- The agent may never call `unpause`, `amend_policy`, `owner_withdraw`, or `set_global_halt`. Assert this with a test that enumerates the instructions the binary can build.
- `guard_divergence_total` increments and pages if the program refuses something the guard allowed.
- Redteam is disabled when `VENUE=shadow`.

## Acceptance
- Agent up on Railway for ≥24h with ≥60s ticks → **B3**.
- ≥1 `ActionReceipt` with `strategy_id = BOOK_ONE` in the last hour → **B4** (`SIG-ACT`).
- `OverTxCap` refusal on chain → **B5** (`SIG-CAP`).
- One of `SlippageExceeded` / `OverSpendCap` / `OverSpendDailyCap` on chain → **B7** (`SIG-SLIP-OR-SPEND`).
- `agent::cannot_build_owner_instructions` passes.
- 24h of logs show `Skip` as the dominant verdict.

## Evidence
Four signatures, the tick log sample, the action histogram for 24h, and the divergence counter (must be 0).
