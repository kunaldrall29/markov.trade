# P02 — `markov-mandate` core: gates, receipts, owner verbs
Seat: Protocol · Window: 3–9 Sep · Inherits `P00-conventions.md`

## Goal
The lock, working, with a receipt for every outcome and a withdraw that no state can disable.

## Pre-flight (STOP and report if any fails)
1. Dump the **deployed** IDL. Extract: account layouts, instruction names, event names, and the exact `BlockReason` variants with their discriminants. Write all of it into `docs/FACTS.md` under "BlockReason enum (APPEND ONLY)".
2. Confirm which of the eleven historical reasons already exist and which of the new ones in `docs/10-PROGRAM-SPEC.md` §4 must be appended.
3. Confirm the account has spare bytes for Phase-1 fields. If not, decide between a reserve-free `Migration<From, To>` (Anchor v1) and deferring — write the decision as an ADR before coding.
4. Confirm you can build and deploy to devnet with the pinned toolchain.

## Deliverables
Implement per `docs/10-PROGRAM-SPEC.md`:
- `state/`: `Mandate`, `Policy` (with the tighten-only diff), `Registry` (admin, `global_halt`, adapter allowlist).
- `gates.rs`: one function per gate, called in the documented order, returning `Option<BlockReason>` plus the `gate_index`.
- `receipts.rs`: `ActionReceipt`, `RefusalReceipt`, `OwnerAction`, emitted as Anchor CPI events so payloads survive log truncation.
- Instructions: `create_mandate`, `fund`, `amend_policy`, `pause`, `unpause`, `revoke`, `execute_venue_action`, `owner_withdraw`, `close_mandate`, `set_global_halt`.
- Day-epoch rollover for `day_notional_used` and `day_spend_used`.
- `intent_id` replay protection.

## Hard constraints
- A gate failure returns `Ok(())` **after emitting** the refusal. It must not unwind. The only exception is `PostCheckFailed`, which reverts on purpose.
- `unpause` checks `signer == mandate.owner` and nothing else can satisfy it.
- `owner_withdraw` has no state check at all. Write the test before the handler.
- `amend_policy` rejects any widening with a hard error.
- Never renumber an existing `BlockReason`.

## Acceptance (LiteSVM, all must be named exactly like this)
```
withdraw_succeeds_in_every_state
withdraw_rejects_non_owner
operator_cannot_withdraw
operator_cannot_unpause
emergency_cannot_unpause_or_withdraw
amend_tighten_ok / amend_widen_rejected
gate_order_matches_spec              # asserts reason AND gate_index for each gate
refusal_emits_receipt_and_commits    # log exists after a refusal
duplicate_intent_refused
daily_counter_rolls
```
Plus a `proptest` that `owner_withdraw` succeeds for every (state, balance ≤ vault) pair.

## Evidence
Deployed program ID, IDL sha, full test output, and one devnet signature each for `create_mandate`, `fund`, and `owner_withdraw`.
