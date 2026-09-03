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

## Evidence from real venues (research completed 2026-09-02, after the decision)

A three-agent study of how Solana perp venues are actually called came back
*after* this ADR was written, from IDLs, endorsed parsing repos and official
docs. It confirms the decision on stronger grounds than the reasoning above,
and finds three further things this ADR did not anticipate.

**Confirmed — a `-> Fill` return is impossible on a request/fulfilment venue.**
Jupiter Perpetuals (`PERPHjGBqRHArX4DySjwM6UJHiR3sWAatqfdBS2qQJu`): not one
write instruction in its IDL carries a `returns` type — only the three
read-only `get*` instructions do — so there is no return data a CPI caller
could read. The fill fields exist **only** in an `IncreasePositionEvent`
emitted by the *keeper's* transaction (`increasePosition4`), not by the
owner's `createIncreasePositionMarketRequest`. Official docs: "two
transactions required to complete a trade request", and a request unexecuted
after 45 seconds "is considered stale and will be rejected".

**Confirmed — a synchronous fill still may not be returnable.** Drift v2
`place_and_take_perp_order` genuinely fills inside one instruction but returns
nothing to the caller. So `Filled(Fill)` is only obtainable when the venue
*chooses* to report it. `demo_perps` reports its fill with `set_return_data`;
the mandate program refuses to emit an `ActionReceipt` when no fill is
reported, rather than substituting the limit price.

**New: a fixed six-error enum cannot express "accepted, outcome pending".**
Jupiter's `ExceedExecutionPeriod` (the 45-second staleness) maps to none of the
six, and — decisively — *these errors surface in the keeper's transaction, not
the caller's*. A CPI that files a request returns `Ok`, and the trade can still
be rejected seconds later with nothing propagated back. Gate 13 therefore
cannot see such a refusal at all.

**New: the "nothing moves tokens without the mandate program authorising it"
rule is dented by escrow.** On Jupiter, collateral moves owner ATA →
`positionRequestAta` (a PDA of the perps program) → collateral custody at
*request* time. Between request and fulfilment the funds sit where the mandate
program cannot reach them, and only a keeper can spend them or return them
(`closePositionRequest2` is keeper-signed, owner not a signer) — so the mandate
could not abort its own escrowed order. The mandate authorises the escrow, not
the fill. That is a materially weaker guarantee than Gate B's, and it must be
stated plainly before any such venue is adopted, not discovered afterwards.

**New: "one market id" is too narrow.** Jupiter has no market-id field; a
market is the tuple (pool, custody, collateralCustody), and long and short use
*different* collateral custody accounts. The fixed-width-bytes rule survives
(they are Pubkeys), but a single `market: [u8;16]` argument does not — an
adapter must map a local id to that tuple.

**Unestablished, and a hard pre-flight for Gate C.** Jupiter's IDL contains
error `CPINotAllowed`. Whether it guards the owner-signed request
instructions could not be determined — the program is closed-source and the
keeper code is not published. If it does, there is no on-chain adapter path at
all. **Do not design a Jupiter adapter before probing this by simulation on a
live cluster.**

### What changes because of this

Nothing in Gate B. `demo_perps` is synchronous and reports its fill, and the
mandate program now refuses rather than inventing one. For Gate C, the venue
checklist in ADR-003/`docs/FACTS.md` gains three questions that the existing
five do not ask:

6. Does a write report its fill to a CPI caller, or only to an off-chain log?
7. Between request and fulfilment, who can move or return the collateral?
8. Is CPI into the write path permitted at all?

Also noted: the Drift documentation now redirects to Velocity
(`docs.drift.trade` → `docs.velocity.exchange`), so the copy ban in `docs/08`
§8 needs `velocity` alongside `drift` — it already has both.
