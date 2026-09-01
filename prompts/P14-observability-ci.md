# P14 — Observability, CI, and the truth scripts
Seat: Truth · Window: continuous, hardened 21–26 Sep · Inherits `P00-conventions.md`

## Goal
Make it impossible to close Gate B on a lie, and make it obvious within minutes when the system starts lying.

## Pre-flight
1. All services emit structured JSON logs with a `tick_id` or `request_id`.
2. The parity job is running and can be forced from CI.

## Deliverables
- Metrics and alerts exactly as `docs/15-TESTING-CI-OBSERVABILITY.md` §§4–5, including the two counter-intuitive ones: **guard divergence** and **refusal drought** (zero redteam refusals in 24h is an alert, because a silent proof surface is worse than a bad trade).
- `scripts/copy-grep.sh`: fails on `APY`, `APR`, `annualized`, `guaranteed`, `risk-free`, `audited`, `mainnet`, `all eleven`, and every venue brand, scanning **built HTML**, not source.
- `scripts/parity-check`: independent chain count vs API count, exit non-zero on mismatch.
- `scripts/gate-b-verify.sh`: takes `--host` and `--api`, runs B1–B15 as far as automatable, prints a red/green table, writes an artifact, exits non-zero if anything is red.
- CI workflows `rust.yml`, `program.yml` (with the **IDL append-only check**), `web.yml`, `truth.yml`, `devnet-smoke.yml`, `gate-b.yml`.
- `docs/runbooks/` — six runbooks, each executed once with the wall-clock time recorded.
- `docs/STATUS.md` daily six-line entry, starting today.

## Hard constraints
- The IDL check fails the build if any existing `BlockReason` variant is renamed, reordered, or removed.
- `gate-b-verify.sh` opens URLs with a clean profile and no cookies — a check that only passes while logged in is not a check.
- No alert may be silenced by editing its threshold in the same PR that made it fire.

## Acceptance
- Deliberately break each of: copy (add `12% APY`), parity (delete a row), IDL (rename a variant), and prove all three CI jobs go red, then revert.
- `gate-b-verify.sh` runs end to end and prints the table.

## Evidence
The three red runs, the reverts, and the first full `gate-b-verify` table.
