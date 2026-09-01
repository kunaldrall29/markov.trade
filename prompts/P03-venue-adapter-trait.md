# P03 — Venue adapter trait (the Gate C seam)
Seat: Protocol · Window: 7–10 Sep · Inherits `P00-conventions.md`

## Goal
One interface that `demo_perps` implements today and a real venue implements later, with a conformance suite that runs against any implementation. This is **B11**.

## Pre-flight
1. Re-read `docs/10-PROGRAM-SPEC.md` §6 and confirm the six methods and the fixed error set.
2. Sanity-check the shape against how a real Solana perp venue would actually be called (request/fulfilment vs direct fill) — enough to be sure the trait can express both. Record what you checked. Do **not** integrate anything.
3. Confirm the CPI authority model: the **mandate PDA** signs venue writes, never the operator key.

## Deliverables
- `crates/markov-venue/src/lib.rs`: `VenueAdapter` trait with `mark`, `positions`, `open`, `increase`, `reduce`, `close`; `Fill { price, notional, fee }`; `VenueError` fixed set (`MarketUnknown`, `StaleMark`, `SlippageExceeded`, `InsufficientCollateral`, `PositionLimit`, `VenuePaused`).
- On-chain CPI helper in `programs/markov-mandate/src/cpi/venue.rs` that builds those calls with the mandate PDA as signer.
- `crates/markov-venue/tests/conformance.rs`: a suite parameterised over an adapter, asserting: writes require the mandate PDA signer; a stale mark returns `StaleMark`; a fill returns a price within the slippage bound; `positions` after `open` reflects the notional; unmapped errors surface as `VenueRejected` upstream.

## Hard constraints
- No method takes a caller-supplied price. Marks come from an account.
- No `&str` market names in the ABI; use a fixed-width id.
- The trait has no method that could move tokens without the mandate program authorising it.

## Acceptance
`cargo test -p markov-venue` green, and the conformance suite is written so that pointing it at a second adapter requires zero edits to the test bodies. Prove that by running it against a trivial second stub.

## Evidence
Trait source, conformance test names, and the note in FACTS that B11's trait is fixed.
