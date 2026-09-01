# P06 — `book-core`: the deterministic proposer
Seat: Agents · Window: 9–14 Sep · Inherits `P00-conventions.md`

## Goal
The book itself: a pure function that turns state plus features into at most one intent, and whose default answer is `Skip`.

## Pre-flight
1. `markov-guard` exists and its fixtures pass.
2. `MarkSource` returns a price and a slot you can prove is fresh.
3. Confirm the Gate B policy numbers from FACTS: `per_tx_cap 50`, `daily_cap 200`, `max_net_delta_usd 20`, `max_gross_usd 100`, `max_slippage_bps 50`.

## Deliverables
- `pub fn propose(state: &BookState, feats: &Features, policy: &PolicyView) -> Intent` implementing `docs/11-AGENT-SPEC.md` §3, including hysteresis (no repeat action if state moved < 25% of a clip) and the "Flatten always wins" rule.
- `Features { regime: Chop|Trend|Halt, funding_favourable: bool /*STUB in Gate B*/, mark, mark_age_slots }` with the stub flag visible in the type name or a comment that cannot be missed.
- Deterministic: same inputs → same intent, asserted by a test that runs the function 1,000 times.
- A simulation harness: feed a replayed price series, print the action histogram, assert `Skip` share > 90%.

## Hard constraints
- One action per tick. No batching.
- No randomness, no wall-clock reads, no network.
- `funding_favourable` is a constant until a real venue exposes funding. It must never appear on a public page as if it were measured.

## Acceptance
```
core::default_is_skip
core::deterministic_over_1000_runs
core::flatten_wins_on_halt
core::hysteresis_prevents_chatter
core::replay_histogram_skip_share_over_90pct
```

## Evidence
Histogram output from the replay harness, and the one-page description of the strategy pasted into the session log.
