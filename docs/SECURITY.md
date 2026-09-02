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
5. `demo_perps` is a mock venue with no token custody.
6. The predecessor program `5o8E…` and its mints are controlled by a key this project does not hold; nothing in Gate B depends on it.

## Invariants we test every build
- `owner_withdraw` succeeds in Active, Paused, Revoked, Expired.
- Only the owner can unpause.
- The operator cannot withdraw, unpause, or widen a cap.
- Every allow and every block that reaches the program emits a receipt.
- `BlockReason` discriminants are append-only: 0–10 as emitted by `5o8E…`, 11–16 appended.

## Reporting
Contact: PENDING (Kunal). No bug bounty on devnet. We will publish what we fix.
