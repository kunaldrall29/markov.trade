# P04 — `demo_perps` mock venue
Seat: Protocol · Window: 8–13 Sep · Inherits `P00-conventions.md`

## Goal
A mock venue that is honest: it exercises the real trait, enforces mark freshness on chain, holds **zero token custody**, and never flatters the book.

## Pre-flight (STOP and report if any fails)
1. Decide the mark path and prove it: try `pyth-solana-receiver-sdk` against the pinned Anchor version. Published compatibility has historically trailed Anchor releases, and Pyth's Solana contracts had an upgrade dated 18 Aug 2026 — so confirm the **current devnet** receiver/price-feed addresses before depending on them.
2. If the SDK does not build, STOP, write ADR-013 choosing the house `MarkAccount` fallback, and record `MARK_SOURCE=house` in FACTS. Do not silently invent a price.
3. Confirm `demo_perps` can be added to the `Registry` adapter allowlist and to a mandate policy.

## Deliverables
- `programs/demo-perps`: `Market`, `MarkAccount`, `Position` PDA `[b"pos", mandate, market_id]`.
- Instructions implementing the trait: `open`, `increase`, `reduce`, `close`, `positions` (view), `mark` (view), plus `post_mark` for the allowlisted poster and `init_market`.
- Deterministic fills at `mark ± fee_bps`. No randomness. No simulated slippage that happens to be favourable.
- `funding_accrued` advanced by a fixed, published devnet rate per elapsed slot, clearly labelled as a devnet constant.
- On-chain `StaleMark` rejection when `slot - mark.slot > market.max_age`.
- `crates/markov-marks`: `MarkSource` trait with `hermes`, `onchain`, and `replay` implementations (replay is for tests and the stale-mark redteam tick).
- A `mark-poster` binary or job that writes `MarkAccount` and can only write price/slot/source.

## Hard constraints
- `demo_perps` never holds or moves tokens. It is an accounting mock. Write that in `SECURITY.md`.
- The mock's error set is exactly the trait's error set.
- The mark's `source` field (`pyth` | `house`) is on-chain and surfaces to the API, because the page must be able to say where the number came from.

## Acceptance
- `markov-venue` conformance suite passes against `demo_perps` unchanged → **B11**.
- `program::stale_mark_refused` passes with a replayed old slot.
- One devnet signature for `init_market`, one for `post_mark`, one for an `open` routed through the mandate program's CPI.
- Grep proves `demo_perps` has no token transfer instruction.

## Evidence
`DEMO_PERPS_ID`, market id, mark source decision + ADR link, three signatures, conformance output.
