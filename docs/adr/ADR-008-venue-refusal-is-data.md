# ADR-008 — A venue refusal must be data, not a program error (P04)

Status: **Accepted 2026-09-02** · Seat: Protocol · Blocks: P04, P07 redteam, any Gate C venue adapter · Amends: `docs/10-PROGRAM-SPEC.md` §3 gate 13

## What we found

`docs/10` §3 specifies gate 13 as: "CPI returns error → `VenueRejected`", emitted as a `RefusalReceipt` that commits with `Ok(())`. That was implemented in P02 as

```rust
if venue_execute(...).is_err() { emit_cpi!(RefusalReceipt { .. }); return Ok(()); }
```

and it does not work. `demo_perps` was made to refuse a stale mark with a
program error, and the whole transaction failed:

```
Program demo_perps ... AnchorError ... Error Code: StaleMark. Error Number: 6001.
Program demo_perps failed: custom program error: 0x1771
Program markov_mandate failed: custom program error: 0x1771
```

`solana_invoke::invoke_signed` genuinely returns `Err` to the caller — the
source is unambiguous — but by then the runtime has already recorded the inner
instruction's failure and the transaction is doomed. Whatever the caller emits
afterwards is rolled back with it.

So a venue refusal signalled as an `Err` produces **no receipt at all**. That
contradicts the rule this project cannot bend: *every allow and every block
that reaches the program emits a receipt*, and a refusal must commit rather
than unwind. Found by a test, not by reading — the receipt count was zero.

## Decision

**The adapter ABI requires venue conditions to be reported as return data,
with `Ok`.** `markov-types::VenueReport` is either `Filled(VenueFill)` or
`Refused { code }`. `demo_perps` reports both through `set_return_data`; the
mandate program reads it and emits an `ActionReceipt` or a `RefusalReceipt` at
gate 13 accordingly.

Program errors are reserved for **structural faults** — a wrong account, a
position belonging to another mandate, an unknown action, arithmetic overflow.
Those are bugs rather than venue conditions, and they should revert. The
mandate maps such an `Err` to `PostCheckFailed`, which is already the one
refusal allowed to be an `Err`.

Silence is also a refusal: if the venue reports nothing, or reports from the
wrong program, gate 13 refuses rather than filling the receipt in with the
limit price (ADR-007).

## Consequences

- **Gate 13 works for `demo_perps`, and B5/B7-style proofs are reachable**
  through it: a venue-level refusal is now a committed, indexable receipt with
  `BlockReason::VenueRejected` and `gate_index = 13`.
- **For a real venue in Gate C, it does not.** A real venue signals refusals
  with program errors, and no ABI request of ours can change that. Against
  such a venue, a venue-level refusal is a **failed transaction with no
  receipt** — only the mandate's own gates 1–12 produce receipts. This is a
  material limit on "every refusal is a receipt", and it must be stated
  plainly wherever the claim appears rather than discovered by a stranger.
  Two honest mitigations, both Gate C work: pre-flight the venue's conditions
  by reading its accounts before the CPI (duplicating its logic, and racy), or
  record the failed attempt off chain and label it as agent-reported rather
  than chain-proven. Neither makes the on-chain claim true.
- **`docs/10` §3 gate 13 is amended** from "CPI returns error" to "the venue
  reports a refusal, or reports nothing".
- The `forced` redteam schedule in `docs/11` §6 can still force a venue
  refusal on `demo_perps`, because the mock cooperates by reporting. It could
  not force one on a real venue and get a receipt.

## What this does not change

`demo_perps` still holds no token custody, still fills deterministically at
`mark ± fee_bps`, and still enforces its own mark freshness independently of
the mandate program. `scripts/no-token-custody.sh` checks the custody claim
three ways — source, built binary, instruction names.
