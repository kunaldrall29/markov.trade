# ADR-004 — Program delta vs. successor (decision D3) and spec reconciliation

Status: **Proposed — needs Kunal** · Date: 2026-09-01 · Seat: Protocol · Blocks: P01 (program row in FACTS), P02, P03, P09

## What is actually deployed (verified 2026-09-01, `docs/FACTS.md`)

The "existing `markov-mandate` program" the pack refers to is **`5o8EAwdHyQ31Nmt6tUDm1y6PNDt5STmVvA6CX3E6WJPm`** from `kunaldrall29/markov`, Anchor 0.31.1, upgrade authority `2fpQ…`, no on-chain IDL. It has emitted all eleven `BlockReason`s on devnet with signatures. A second, older program (`CT2n…`, `MarkovFyi/markov-program`) has never emitted a receipt and is not bound by the append-only rule.

Where the pack's `docs/10-PROGRAM-SPEC.md` and the deployed program disagree, the deployed program wins:

| Topic | `docs/10` says | Chain says |
|---|---|---|
| `BlockReason` numbering | `Paused`=0 … `SlippageExceeded`=10 | `OverTxCap`=0, `OverDailyCap`=1, `OverSpendCap`=2, `OverSpendDailyCap`=3, `ProgramNotAllowed`=4, `TokenNotAllowed`=5, `SlippageExceeded`=6, `Expired`=7, `Paused`=8, `Revoked`=9, `Unauthorized`=10 |
| Two names | `NotOperator`, `VenueNotAllowed` | `Unauthorized`, `ProgramNotAllowed` |
| Mandate seeds | `[mandate, owner, strategy_id, nonce]` | `[mandate, owner, seed_le_u64]` |
| State enum | Active / Paused / Revoked / Expired | `u8` 0/1/2, **no Expired state** (expiry is a gate on `expires_ts`) |
| Policy fields | venues[4], tokens[4], allowed_actions, per_tx_cap, daily_cap, max_slippage_bps, spend_per_call, spend_daily, max_mark_age_slots, expiry_ts | programs[4], tokens[4], per_tx_cap, daily_cap, spend_per_call_cap, spend_daily_cap, max_slippage_bps — **no** `allowed_actions`, `max_mark_age_slots`, or expiry in policy |
| Reserve bytes | required | **none** (`Mandate` = 8 + 673) |
| Registry / global halt | required | absent |
| `amend_policy` | tighten-only, hard error otherwise | any valid policy accepted |
| Replay protection | client `intent_id` → `DuplicateIntent` | program-incremented `nonce`; no client id |
| Mark / freshness gate | `StaleOracle` | absent |
| Post-CPI checks | `PostCheckFailed` reverts | absent |
| Events | `emit_cpi` (`ActionReceipt`, `RefusalReceipt`, `OwnerAction`) | `emit!` (`ActionExecuted`, `ActionRefused`, `MandateCreated`, `MandateFunded`, `Paused`, `Unpaused`, `Revoked`, `OwnerWithdrew`, `PolicyAmended`) |
| Instructions | `create_mandate, fund, amend_policy, pause, unpause, revoke, execute_venue_action, owner_withdraw, close_mandate, set_global_halt` | `register_operator, create_mandate, fund, amend_policy, pause, unpause, revoke, owner_withdraw, execute_swap, execute_deposit, execute_withdraw_venue, spend` |

What already matches the non-negotiables: refusals emit and return `Ok(())`; `owner_withdraw` has no state check; `unpause` is owner-only; the emergency key can only pause and revoke; the mandate PDA signs every venue CPI.

## The decision

The delta is not a few fields. Every account layout, seed, instruction and event shape changes. There are two honest routes:

**A. Successor program, new program ID.** Build the pack's spec on the D0 pin. Keep `BlockReason` 0–10 **verbatim in the deployed order and names**, append `StaleOracle`=11, `ActionNotAllowed`=12, `DuplicateIntent`=13, `GlobalHalt`=14, `VenueRejected`=15, `PostCheckFailed`=16 (final order fixed in P02, never reordered once emitted). `5o8E…` stays on chain as history; its receipts remain readable; FACTS records both IDs and the handover slot. The append-only promise is kept at the enum level, which is the level a stranger can check. No dependency on who holds `2fpQ…`.

**B. In-place upgrade of `5o8E…`.** Requires the `2fpQ…` keypair. Because the layout changes and there is no reserve, every existing `Mandate` account (20, all demo) either gets an Anchor v1 `Migration<From, To>` (only if D0 = Anchor 1.x) or is abandoned and recreated. Keeps one program ID in every explorer link and in the litepaper. Costs: a migration path to write and test inside Gate B for accounts nobody needs.

## Recommendation

**A.** It is the only route that does not depend on locating a private key this session could not find, it avoids writing a migration for throwaway demo accounts, and it lets D0 choose Anchor 1.1.2 freely. The cost is a second program ID in FACTS with a one-line explanation, which is cheaper than any of B's costs.

Engineering pack v0.2 (adopted 2026-09-01) does not change this: its `CORRECTIONS.md` still lists the "historical eleven" as `Paused, Revoked, Expired, NotOperator, VenueNotAllowed, …` and says "discriminants from the deployed IDL" — the deployed program has no on-chain IDL, and the checked-in IDL plus 20 on-chain refusal payloads give `OverTxCap`=0 … `Unauthorized`=10.

Regardless of A or B: `docs/10-PROGRAM-SPEC.md` §1, §2, §4 and `CORRECTIONS.md` must be amended to the chain's names and numbering **before P02 starts**, and `docs/11` §4 / the guard fixtures / `docs/12` schema must use `Unauthorized` and `ProgramNotAllowed`, not the pack's invented names.

## Open inputs from Kunal

1. Do you hold `keys/deployer.json` (`2fpQ…`)? If not, B is impossible and A is the decision by default.
2. Is `execute_swap` / `demo_swap` history worth keeping in the public feed, or does Gate B's feed start at the successor's first slot?
3. Confirm that the six appended reason names above are the ones you want; they cannot change after first emit.
