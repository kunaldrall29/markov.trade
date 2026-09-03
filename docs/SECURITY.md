# SECURITY — Markov Book (devnet)

Status: devnet. Unaudited. Marked PnL. Not a promised rate. **Stub written in P01; completed in P15 from `docs/14-SECURITY-AND-KEYS.md` §6 with real signatures.**

## Keys
| Key | Pubkey | Held by | Can | Cannot |
|---|---|---|---|---|
| Owner (demo) | `5RPxDN9hxG3YaBSZo6TWfDx1CUGpJKFsuP3hmjUkd1Hv` | tape recording only | fund, amend (tighten), pause, unpause, revoke, withdraw on the demo mandate | anything on another owner's mandate |
| Operator | `EU8a73vNg3Ti4DXtnXF41JLhc78um17er9LDZgsWCbNY` | `book-one` service | propose within policy | withdraw, unpause, amend up, pause, revoke, halt |
| Emergency | `A67Gw8VZbYx6qEvFeDdq4eGzpz33L3BkqyhpFrCN7JxB` | `bot` service | pause, revoke | unpause, withdraw, trade, amend |
| Mark poster | `2Sx9UFGkkHf1sQ2xmPQyHgNcP7HiVnxoQRg8btzhM5Bv` | mark-poster job | write price/slot to the allowlisted `MarkAccount` | anything else |
| Upgrade authority | `8wuYJD6bZjSA115mwXgguPoUzSqEP3dc3GxBpCu4M3mn` | single key, Kunal's workstation | deploy | — |

## Accepted risks
1. Single-key upgrade authority on devnet. Accepted. Mainnet plan: multisig, then freeze.
2. Unaudited program.
3. Delta / gross / daily-loss halt enforced off-chain in v0; on-chain in Phase 1.
4. Mark source on devnet is the Pyth sponsored feed account (`docs/FACTS.md` `MARK_SOURCE`), relayed by a house poster only as a fallback and labelled as such.
5. `demo_perps` is a mock venue with **no token custody**, and that claim is checked rather than asserted: `scripts/no-token-custody.sh` fails the build if the source references a token program or a transfer CPI, if the built binary contains the SPL Token program id, or if any instruction name suggests moving value. Collateral never reaches the venue, so a bug in the mock cannot touch a vault.
6. Its funding figure is a **fixed published devnet constant** (1e-6 of notional per slot, charged to the long), not a measured rate. Any surface showing it must say so.
7. **With a real venue, a venue-level refusal would not produce a receipt** (ADR-008): a failed CPI fails the whole transaction, so only the mandate's own gates 1–12 are guaranteed to leave a record. `demo_perps` cooperates by reporting refusals as data; no real venue will.
8. **A CPI carries the mandate's signature to the venue's own CPIs** (ADR-009).
   The venue is a callee of a CPI the mandate PDA signed, and inside its own
   instruction that PDA is a signer — so a venue handed the vault can spend it
   with the mandate's authority. This was demonstrated, not theorised:
   `programs/test-rogue-venue` moved 10 of 1,000 base units out of a vault and
   the transaction committed. Gate 15 now refuses to forward any account the
   mandate controls, and the vault snapshot is compared before any receipt
   commits. **The devnet program deployed before 2026-09-03 carries the
   vulnerability until it is upgraded** — no mandate on it holds value beyond
   test tokens, and the only registered venue takes no custody.
9. The predecessor program `5o8E…` and its mints are controlled by a key this project does not hold; nothing in Gate B depends on it.

## Invariants we test every build
- `owner_withdraw` succeeds in Active, Paused, Revoked, Expired.
- Only the owner can unpause.
- The operator cannot withdraw, unpause, or widen a cap.
- Every allow and every block that reaches the program emits a receipt.
- A venue is never handed an account the mandate controls, and a rogue venue
  that is handed one anyway cannot spend it (`a_rogue_venue_is_never_handed_the_vault`).
- A venue that reports filling more than it was asked for reverts the
  transaction rather than writing the receipt
  (`a_venue_that_reports_more_than_it_was_asked_for_reverts`).
- The vault, the owner, the operator and the policy hash are identical before
  and after every venue call.
- `BlockReason` discriminants are append-only: 0–10 as emitted by `5o8E…`, 11–17 appended.

## Reporting
Contact: PENDING (Kunal). No bug bounty on devnet. We will publish what we fix.
