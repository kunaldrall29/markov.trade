# ADR-007 — A venue write returns an outcome, not always a fill (P03)

Status: **Accepted 2026-09-02** · Seat: Protocol · Blocks: P04, and any Gate C venue adapter

## What the pack says

`docs/10-PROGRAM-SPEC.md` §6: "Every write returns a `Fill { price, notional, fee }` the program can post-check."

## Why that is not quite right

That sentence assumes a venue fills synchronously inside the CPI. Not every Solana perp venue does. A well-known shape is **request/fulfilment**: the caller creates a position *request*, and a keeper or crank settles it in a later transaction. At CPI time there is no price, no fill and no position — only an accepted request.

An adapter for such a venue, forced to return `Fill`, has exactly three options:

1. **Invent a fill** — return the limit price as though it had been filled. The mandate program would then emit an `ActionReceipt` carrying `fill_price`, the indexer would store it, and `/book` would show a stranger a fill that never happened. A receipt with a signature on it is the product; a fabricated one is the worst failure mode this project has.
2. **Return an error** — refuse every asynchronous venue at the type level, which turns the "Gate C seam" into a seam that only fits mocks.
3. **Say what happened.**

## Decision

The trait's writes return:

```rust
pub enum VenueOutcome {
    Filled(Fill),
    Requested { request_id: [u8; 32] },
}
```

`demo_perps` is synchronous and returns `Filled` only; every Gate B path sees `Filled`, and gate 14's post-check is unchanged. `Requested` costs one enum variant and no code, and it exists so that a Gate C adapter cannot quietly choose option 1.

## Consequences, and what is deliberately not built

- **No Gate B code handles `Requested`.** The mandate program's `execute_venue_action` is not changed by this ADR. If a request-based venue is ever adopted, P02's receipt set needs a third variant — a receipt that says *requested*, carrying no price — and gate 14's post-check does not apply to it, because nothing has moved yet. That is Gate C work and is recorded here rather than pre-built (`docs/07` §13: generality with no second implementation is decoration).
- **The conformance suite reflects this.** Three checks — fill-within-bound, positions-after-open, and unhonourable-limit — accept `Requested` as a legitimate outcome, because a venue that has not filled cannot be judged on its fill. `crates/markov-venue/tests/conformance.rs` includes a `RequestVenue` adapter precisely so that path is exercised, and the suite was **found to be wrong here first**: it initially failed `RequestVenue` on the limit check, which is what surfaced the distinction between request-time and fill-time enforcement.
- **An unauthorised caller is not a venue condition.** The six-error set describes venue states (`MarketUnknown`, `StaleMark`, `SlippageExceeded`, `InsufficientCollateral`, `PositionLimit`, `VenuePaused`). A wrong signer is a rejection, not a state, and every venue error reaches the chain as one `BlockReason::VenueRejected` at gate 13 regardless — the mandate program records *that* the venue refused and does not re-interpret why.

## Amendment to the spec

`docs/10-PROGRAM-SPEC.md` §6's bullet is amended to: "Every write reports what happened — a `Fill` the program can post-check, or an accepted request carrying no price. It may never invent a fill."
