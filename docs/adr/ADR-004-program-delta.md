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

## Verification pass (2026-09-01)

- **The Mandate account has 154 bytes of unnamed slack** (allocated 8 + 673, serialized 519; bytes 527..681 zero on all three inspected accounts). "No reserve" was misleading: up to 154 bytes can be appended without realloc. That does not rescue B, because the spec's layout changes are far larger than 154 bytes and reorder existing fields.
- **B's real cost was understated.** A new layout means the new binary cannot deserialize the 20 existing mandates, so `owner_withdraw` on their vaults dies unless every owner path uses `Migration<From, To>` or a legacy withdraw instruction is kept forever. That is a non-negotiable #3 violation, not a migration for throwaway accounts — and it is the strongest argument for A.
- **A is not key-free either.** A successor deploy needs a funded deployer (~3 SOL of rent for a 400 KB program; the faucet refused this box twice; `2fpQ…` holds 6.75 SOL), and `2fpQ…` is also the **mint and freeze authority of USDC-d `6eDV…`**, so a fresh demo owner needs USDC-d from a new mint (labelled as such) or a transfer from a holder whose key custody is equally unknown. Input #1 therefore decides the mint and the funding, not just the program.
- **Continuity must be provable, not asserted.** With B15 banning "all eleven", the historical reasons are provable only if the feed carries a `program_id` per row and a per-row explorer link that resolves to the right program; index both program IDs (two IDLs, one receipts table with a program column). Publish the successor's IDL on chain (`anchor idl init`) so `ActionRefused` decodes on the explorer the tape links to. Surface the predecessor ID and handover slot on `/v1/facts` and a footer line on `/book`, since markovhq.com's litepaper names `5o8E…`.
- **Decide the event mechanism here.** `emit_cpi` puts the payload in a self-CPI's instruction data (survives log truncation) but forces `getTransaction`-per-signature ingestion on an RPC that already 429s; `emit!` keeps `logsSubscribe` + backfill. The successor should choose with P09 in the room.
- Verifiable build (see ADR-001) so the successor's on-chain hash ties to a commit.

## Open inputs from Kunal

1. Do you hold `keys/deployer.json` (`2fpQ…`)? If not, B is impossible and A is the decision by default — and a new USDC-d mint plus a funded deployer are required.
2. Is `execute_swap` / `demo_swap` history worth keeping in the public feed? Recommendation: yes, indexed under its own `program_id` (see above), rather than starting the feed at the successor's first slot.
3. Confirm that the six appended reason names above are the ones you want; they cannot change after first emit.
4. Funding source for the successor deploy and the P02–P04 redeploys (devnet SOL) given the faucet refusal.
