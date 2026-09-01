# 15 — Testing, CI, Observability
Markov Book · 31 August 2026 · v0.2

Gate B is a claim about a running system, so the tests are mostly about states that must remain possible (withdraw) and states that must remain impossible (operator withdraw, silent refusal).

---

## 1. Test pyramid

| Layer | Tool | What it covers | Runs |
|---|---|---|---|
| Pure unit | `cargo test` | guard rules, core proposer, policy tighten-diff, day rollover math | every push |
| Fixture | `cargo test` + `crates/markov-guard/src/fixtures/*.json` | one file per veto reason + one allow | every push |
| Program | LiteSVM (in-process unit) | every gate, every state transition, receipt durability | every push |
| Property | `proptest` | tighten-only never widens; caps never overflow; `owner_withdraw` succeeds for any state and any balance ≤ vault | nightly + pre-merge |
| Integration | devnet script | fund → act → refuse → revoke → withdraw against real hosts | on `main`, nightly |
| E2E | Playwright | B10 withdraw-enabled matrix; the tape beats; explorer links resolve | on PR preview |
| Truth | `copy-grep`, `parity-check`, `gate-b-verify` | B15, B9, whole freeze list | every push + nightly |

## 2. Tests that are release blockers

```
program::withdraw_succeeds_in_every_state
program::operator_cannot_withdraw
program::operator_cannot_unpause
program::emergency_cannot_unpause_or_withdraw
program::refusal_emits_receipt_and_commits      # not a rollback
program::gate_order_matches_spec                # reason AND gate_index
program::amend_widen_rejected
program::duplicate_intent_refused
program::stale_mark_refused
guard::fail_closed_on_missing_input
guard::mirrors_onchain_ladder_order
agent::skip_is_default_when_state_unreadable
web::withdraw-enabled-in-every-state            # B10
truth::copy_grep_clean                          # B15
truth::api_parity_matches_chain                 # B9
```

If any of these are red, nothing ships and Gate B is open. There is no "temporarily skip" annotation allowed on this list; a skip is a red.

## 3. CI workflows

| Workflow | Trigger | Steps |
|---|---|---|
| `rust.yml` | push, PR | fmt → clippy `-D warnings` → build → test → `cargo deny` → `cargo audit` |
| `program.yml` | push touching `programs/**` | anchor build → IDL diff check → LiteSVM suite → **IDL/BlockReason append-only check** |
| `web.yml` | push touching `apps/web/**` | tsc → eslint → build → Playwright on preview → Lighthouse budget |
| `truth.yml` | push + nightly | `copy-grep.sh` on built HTML → `parity-check` against staging API → link-check with a logged-out fetch |
| `devnet-smoke.yml` | nightly + manual | full integration script on hosted services, posts signatures to the run summary |
| `gate-b.yml` | manual | `gate-b-verify.sh`, prints B1–B15 as red/green, writes an artifact |

**IDL append-only check.** CI decodes the previous IDL from `main`, compares `BlockReason` variants, and fails if any existing variant is renamed, reordered, or removed. New variants appended at the end pass. This is the mechanical enforcement of the append-only rule.

## 4. Metrics

Agent: `ticks_total`, `intents_total{action}`, `verdicts_total{kind}`, `vetoes_total{reason}`, `onchain_refusals_total{reason}`, `guard_divergence_total`, `submit_latency_ms`, `rpc_errors_total{endpoint}`, `mark_age_slots`, `net_delta_usd`, `gross_usd`, `hedge_error_usd`, `redteam_refusals_24h`, `actions_last_hour`.

Indexer/API: `ingest_lag_slots`, `events_indexed_total{kind}`, `unparsed_events_total`, `parity_ok`, `parity_delta`, `api_latency_ms{route}`, `api_5xx_total`.

Web: nothing that identifies a wallet. Error tracking only.

## 5. Alerts (and what each one means)

| Alert | Condition | Meaning |
|---|---|---|
| **Guard divergence** | `guard_divergence_total > 0` | the off-chain mirror is wrong; the page's explanations may be lying — page immediately |
| Agent silent | no tick for 3× interval | the book is not running; the dashboard's "live" chip must already have flipped |
| Overtrading | `actions_last_hour > MAX_ACTIONS_PER_HOUR` | halt the agent; a book that trades every minute is a bug |
| Refusal drought | `redteam_refusals_24h == 0` | the proof surface is broken, which is worse than a failing trade |
| chainReady false | > 5 min | the page is degraded; check lag, ingest, parity in that order |
| Parity mismatch | `parity_ok == false` | the API and the chain disagree — treat as a data incident, freeze claims |
| Stale mark | `mark_age_slots > policy` for > 10 min | every intent is being vetoed; the book is effectively paused |
| Daily-loss halt fired | event | expected occasionally; note it in the paper log |
| Fee balance low | operator SOL < 0.2 | run the faucet top-up runbook |

## 6. Runbooks to write before Gate B closes

`docs/runbooks/` — each one executed at least once, with the wall-clock time recorded:

1. `agent-halt.md` — stop the book, confirm no pending intents, announce.
2. `key-rotation.md` — rotate operator key, prove the old key is refused with `NotOperator`, record both signatures.
3. `indexer-rebuild.md` — full rebuild from the deploy slot, parity over the whole range.
4. `faucet-topup.md` — devnet SOL for operator, emergency, and mark-poster.
5. `rpc-failover.md` — switch primary, confirm lag recovers.
6. `incident-writeup.md` — the template: what happened, what the receipt says, what we changed.

## 7. The daily 15 minutes

Written into `docs/STATUS.md`, same six lines every day, skipped never:

```
date · paper day N · ticks · actions · refusals (organic/forced) · reasons
hedge error mean/max · daily-loss halt fired? · chainReady uptime
one sentence: did the book do its job?
```

Two skipped days in a row and, per `05-ROADMAP`, the project is a slide deck again. The status file is the cheapest instrument that prevents that.
