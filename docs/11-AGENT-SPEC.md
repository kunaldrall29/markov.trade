# 11 — Agent Spec (`book-one`)
Markov Book · 31 August 2026 · v0.2

One binary. Two modes: `VENUE=devnet` (submits) and `VENUE=shadow` (paper, never touches a key). Same core, same guard, same reason vocabulary — that is the whole reason it is one binary.

---

## 1. Module boundaries

```
main.rs
 ├── scheduler      tick every TICK_SECONDS (>=60), jitter 0-5s, backoff on error
 ├── state          read chain: mandate, policy, positions, mark, vault balance
 ├── sidecar        RegimeSource -> Features            (stub in Gate B)
 ├── core           (BookState, Features, Policy) -> Intent     PURE
 ├── guard          (Intent, GuardState, PolicyView) -> Verdict PURE
 ├── submitter      Intent -> tx -> signature -> receipt         I/O
 ├── redteam        scheduled forced intents (devnet only)
 ├── paper          shadow writer, daily markdown
 └── metrics        counters, gauges, JSON logs keyed by tick_id
```

**Purity is enforced, not encouraged.** `core` and `guard` take every input as an argument — including `now`, `slot`, and the mark. They return a value. They cannot read a clock, open a socket, or log to a file. That is what makes them testable against fixtures and identical in paper and live.

## 2. Tick contract

```
tick(id) :=
  1. state  = read_chain()            // fail closed on any error -> Skip, log, return
  2. mark   = mark_source.get()       // includes slot + age
  3. feats  = sidecar.features(state, mark)
  4. intent = core::propose(state, feats, policy)     // default: Skip
  5. verdict= guard::evaluate(intent, guard_state, policy_view)
  6. match verdict:
       Skip            -> record tick, no tx
       Veto(reason)    -> record veto (off-chain), no tx unless intent.forced
       Allow(intent)   -> submitter::send(intent)
  7. record tick row: {tick_id, slot, regime, intent, verdict, sig?, latency}
```

Every tick produces exactly one row, including the boring ones. **`Skip` is the expected outcome.** A day where the agent skipped 1,400 of 1,440 ticks is a working day; a day where it acted 400 times is an incident.

## 3. `core` — the deterministic book

Gate B strategy, stated so a reviewer can hold it in one page:

```
target_delta      = 0
delta_band        = ±max_net_delta_usd            (20)
gross_cap         = max_gross_usd                 (100)
clip              = min(per_tx_cap, gross_cap/4)  (<= 50)

propose(state, feats, policy):
  if feats.regime == halt            -> Flatten if gross>0 else Skip
  if state.daily_loss_halt_active    -> Flatten if gross>0 else Skip
  if mark.age > policy.max_mark_age  -> Skip            // guard would veto anyway
  d = state.net_delta_usd
  if |d| > delta_band                -> Reduce/Increase the hedge leg by min(|d|, clip)
  if feats.regime == trend and gross > gross_cap/2 -> Reduce by clip
  if feats.regime == chop and gross < gross_cap and funding_favourable
                                     -> Increase by clip
  else                               -> Skip
```

Rules attached to it:
- **Hysteresis.** Never act if the same action was taken in the previous tick and the state moved less than 25% of a clip. Prevents the dashboard from looking like a slot machine.
- **One action per tick.** No batching, no multi-leg atomic cleverness in Gate B.
- **Flatten always wins.** If two rules fire and one is `Flatten`, the answer is `Flatten`.
- `funding_favourable` is a **stub constant** in Gate B until a real venue exposes funding. It is labelled as a stub in the code and on the paper log. It is not a claim.

## 4. `guard` — the pure veto

```rust
pub enum Verdict { Allow(Intent), Veto(BlockReason), Skip }

pub fn evaluate(intent: &Intent, s: &GuardState, p: &PolicyView) -> Verdict
```

Checks, in this order, mirroring the on-chain ladder so that a divergence is visible:

| # | Check | Veto reason |
|---|---|---|
| 1 | mandate state not Active | `Paused`/`Revoked`/`Expired` |
| 2 | mark age > `max_mark_age_slots` | `StaleOracle` |
| 3 | action not in `allowed_actions` | `ActionNotAllowed` |
| 4 | `notional > per_tx_cap` | `OverTxCap` |
| 5 | `day_used + notional > daily_cap` | `OverDailyCap` |
| 6 | spend budgets | `OverSpendCap` / `OverSpendDailyCap` |
| 7 | projected `|net_delta|` after fill > band | `DeltaBandExceeded` *(off-chain only in v0)* |
| 8 | projected gross > `max_gross_usd` | `GrossExceeded` *(off-chain only in v0)* |
| 9 | slippage bound | `SlippageExceeded` |
| 10 | daily loss ≥ 5% of session start equity | `DailyLossHalt` *(off-chain only in v0)* |

Rows 7, 8 and 10 have **no on-chain counterpart in v0** (ADR-05). They are recorded on the receipt as metadata and on the tick row, and the dashboard must label them as off-chain-enforced. Saying "delta is enforced" without that qualifier is a B15 failure.

**Fail-closed default:** any `None`, parse error, or arithmetic overflow inside the guard returns `Veto(GuardInternal)`. There is no path that returns `Allow` on missing data.

**Fixture tests:** `crates/markov-guard/src/fixtures/*.json` — one file per veto reason plus one allow, run in CI. A guard change that does not touch fixtures is suspicious.

## 5. `submitter`

- Builds one transaction per intent. Compute-budget instruction with a measured limit, not a guess.
- Signs with the operator key only. The key is loaded once at boot; if it is missing, the process exits rather than running unauthenticated.
- `intent_id` is the idempotency key: on retry after a timeout, re-send the **same** transaction (same blockhash if still valid) rather than building a new one, and treat `DuplicateIntent` as success-already-happened.
- Confirms at `confirmed`, records the signature, and writes it to the tick row.
- Retries: 3 attempts, exponential backoff, then give up and skip the tick. Never retry a refusal — a refusal is a result, not an error.
- Never widens a parameter to make a transaction land. If the tx fails because the cap is too low, the answer is the refusal, not a bigger cap.

## 6. `redteam`

The only component allowed to submit an intent the local guard vetoed, and it is still the program that refuses.

| Schedule (devnet only) | Forced intent | Expected receipt |
|---|---|---|
| every 6h | notional = `per_tx_cap + 1` | `OverTxCap` |
| every 12h | `max_slippage_bps` beyond bound | `SlippageExceeded` |
| every 12h | spend above per-call budget | `OverSpendCap` |
| after any revoke | any valid action | `Revoked` |
| daily | stale mark (replayed old slot) | `StaleOracle` |

Constraints: `forced=true` on the receipt, so nobody can later claim these were organic. Redteam is disabled in `shadow`. Redteam has its own metric — **if the redteam has produced zero refusals in 24h, that is an alert**, because it means the proof surface is silently broken.

## 7. `paper` mode (`VENUE=shadow`)

Same tick, same core, same guard, no key, no chain. Writes one file per day:

```
paper/2026-09-03.md

date: 2026-09-03
mark_source: hermes SOL/USD (feed id in FACTS)
started: 09:02Z   ended: 23:59Z   ticks: 812
regime counts: chop 780 / trend 30 / halt 2
proposed: 41   skipped: 771   would_send: 33   vetoed: 8
veto reasons: OverTxCap 3, DeltaBandExceeded 4, StaleOracle 1
hedge error (mean |target-actual| delta, USD): 3.10   max: 11.40
daily loss halt: no
marked return: -0.42%          <- marked, devnet-shaped, not a rate
notes: mark feed dropped 09:41-09:44, 3 ticks skipped on StaleOracle
```

Rules: one file per calendar day, written even when the day is boring or bad. A missing day is written as `no run — reason`. **Never backfill a day that did not run.** `PAPER_START_DATE` goes in FACTS on the first run and is never edited. No APY field exists in the schema, so nobody can add one by accident.

## 8. Configuration

| Var | Default | Constraint |
|---|---|---|
| `TICK_SECONDS` | 60 | hard floor 60 in Gate B; a lower value refuses to boot |
| `MAX_ACTIONS_PER_HOUR` | 6 | exceeding it halts the agent and pages |
| `DAILY_LOSS_HALT_BPS` | 500 | 5% |
| `MARK_MAX_AGE_SECS` | 150 | seconds since the mark's `publish_time` (ADR-003, 2026-09-02: seconds, not slots — devnet pacing is ≈165 ms/slot); must be ≤ the on-chain policy value |
| `REDTEAM_ENABLED` | false | true only on the devnet agent |
| `SIDECAR` | `stub` | `stub` returns `chop`; any other value must be declared in FACTS |

## 9. Observability contract

Every tick logs one JSON line with `tick_id`, `slot`, `regime`, `intent`, `verdict`, `reason`, `sig`, `latency_ms`. Metrics exported:

`ticks_total`, `intents_total{action}`, `verdicts_total{allow|veto|skip}`, `vetoes_total{reason}`, `onchain_refusals_total{reason}`, `submit_latency_ms`, `rpc_errors_total`, `mark_age_slots`, `net_delta_usd`, `gross_usd`, `hedge_error_usd`, `redteam_refusals_24h`.

**Divergence alarm.** If the program refuses an intent the local guard allowed, emit `guard_divergence_total` and page. That counter should be zero forever; a non-zero value means the off-chain mirror is wrong and the dashboard's "why" text is lying.

## 10. What the agent is not allowed to do

- Hold or read the emergency key or an owner key.
- Call `unpause`, `amend_policy`, `owner_withdraw`, or `set_global_halt`.
- Write to the indexer database.
- Read a price from anything other than a `MarkSource`.
- Take an LLM output as anything but a field inside `Features`.
- Run faster than 60 seconds, or act more than `MAX_ACTIONS_PER_HOUR` times.
