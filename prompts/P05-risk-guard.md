# P05 — `markov-guard`: the pure veto
Seat: Agents · Window: 7–12 Sep · Inherits `P00-conventions.md`

## Goal
A function. No I/O, no clock, no network, no logging. Given an intent, a state snapshot, and a policy view, it returns Allow, Veto(reason), or Skip. It mirrors the on-chain ladder so a divergence is detectable.

## Pre-flight
1. Read the deployed `BlockReason` list from FACTS. The guard's reasons must be a subset plus the three off-chain-only ones (`DeltaBandExceeded`, `GrossExceeded`, `DailyLossHalt`), which must be clearly marked as off-chain in the type itself.
2. Confirm `deny.toml` forbids tokio/reqwest/solana-client in this crate. If it does not, fix that first.

## Deliverables
- `pub fn evaluate(intent: &Intent, state: &GuardState, policy: &PolicyView) -> Verdict`
- Rules in the exact order of `docs/11-AGENT-SPEC.md` §4.
- `GuardState` carries `now_ts`, `slot`, `mark`, `mark_slot`, positions, vault balance, day counters, session-start equity — all **passed in**, never read.
- Overflow-safe arithmetic (`checked_*`), and any `None`/overflow → `Veto(GuardInternal)`.
- `fixtures/*.json`: one per veto reason, one allow, one skip. Golden tests read them.
- A doc comment at the top of `lib.rs` that states the whole guard in under 20 lines, in plain words. If you cannot write that page without mentioning a model, the design is wrong.

## Hard constraints
- No `SystemTime::now()`, no `chrono::Utc::now()`, no RPC type in the signature.
- No path returns `Allow` when an input is missing.
- The off-chain-only reasons are tagged `enforcement: OffChainV0` in the type so the API and UI can label them without guessing.

## Acceptance
```
guard::fail_closed_on_missing_input
guard::mirrors_onchain_ladder_order
guard::fixture_<reason>        # one per reason
guard::allow_only_when_all_pass
```
Plus a compile-fail test (trybuild or a CI grep) proving the crate cannot import a network client.

## Evidence
Test output, the 20-line plain-words doc comment pasted into the session log, and the fixture list.
