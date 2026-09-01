# P08 — Paper runner (`VENUE=shadow`)
Seat: Agents · Window: starts 31 Aug, runs to the gate · Inherits `P00-conventions.md`

## Goal
Seven consecutive dated daily files, ugly days kept. This is **B12**, and it starts before anything else is built because days cannot be manufactured later.

## Pre-flight
1. A price source responds and returns a slot/timestamp with each price. Record the endpoint and feed id in FACTS.
2. `book-core` and `markov-guard` compile (they can be minimal at first — a paper day with a simple core is worth more than a perfect core with no days).
3. `PAPER_START_DATE` is empty in FACTS. Once written, it is never edited.

## Deliverables
- Same binary, `VENUE=shadow`: no keypair loaded, no chain writes, redteam disabled.
- One file per calendar day at `paper/YYYY-MM-DD.md` in the exact schema of `docs/11-AGENT-SPEC.md` §7.
- A day with no run is written as `no run — <reason>`; it is never omitted and never backfilled.
- `/paper` API route serves the folder; `/paper` page renders it.
- Tests: `paper::one_file_per_day`, `paper::no_backfill` (attempting to write a past date fails), `paper::schema_has_no_apy_field`.

## Hard constraints
- The schema literally has no APY/APR/annualised field, so it cannot be added by accident.
- Ugly days are kept verbatim. If the marked return is negative for a week, that week ships.
- The file says which mark source it used and flags any gap in the feed.

## Acceptance
Seven consecutive dated files exist, at least one of them boring or negative, and `PAPER_START_DATE` is in FACTS → **B12**.

## Evidence
`ls paper/`, one full file pasted, and the start date row.
