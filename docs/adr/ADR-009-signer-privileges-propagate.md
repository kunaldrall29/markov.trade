# ADR-009 — A CPI carries the mandate's signature to the venue's own CPIs (P04)

Status: **Accepted 2026-09-03** · Seat: Protocol · Blocks: P04, any Gate C venue adapter · Amends: `docs/10-PROGRAM-SPEC.md` §3 (adds gate 15), ADR-008

## What we found

The mandate program signs its CPI into a venue as the mandate PDA, and the
vault's authority *is* that PDA. The comfortable assumption — written into a
test's doc comment before it was checked — was that this is safe because "a PDA
signature does not propagate to the callee's own CPIs; the seeds belong to the
mandate program".

That is wrong. Only the *creation* of a PDA signature is restricted to the
owning program. Once created, the signer privilege travels down the CPI chain
like any other signer privilege: inside the venue's instruction the mandate PDA
is a signer, and the venue may pass it on to the token program.

`programs/test-rogue-venue` was written to try it. Given the vault among its
accounts, it issued an ordinary SPL `Transfer` with the mandate as authority:

```
Program markov_mandate invoke [1]
  Program 4hFUD…(rogue) invoke [2]
    Program Tokenkeg… invoke [3]
    Program Tokenkeg… success
  Program 4hFUD…(rogue) success
Program markov_mandate success
```

vault 1000 → 990, thief 999000 → 999010, transaction `Ok`.

Two separate defects let that commit:

1. **The vault was forwarded to the venue at all.** `execute_venue_action`
   passes `ctx.remaining_accounts` through to the CPI verbatim, and the
   operator chooses them. Any venue the policy allows could therefore be handed
   the collateral together with the authority over it.

2. **Gate 14 was skipped on the gate-13 refusal path.** The rogue reported
   nothing, so gate 13 took the "the venue refused" branch, emitted a clean
   `RefusalReceipt` and returned `Ok(())` — *before* the vault snapshot was
   compared. A theft was recorded on chain as a polite refusal.

A third weakness was latent: the post-check tolerated `|Δbalance| <= notional`,
so even on the success path a venue could take exactly the notional out of the
vault and pass. The tolerance existed for a future venue that collects
collateral. It bought nothing today and hid the attack.

## Decision

**Gate 15 — `ControlledAccountForwarded` (`BlockReason` 17).** Before the CPI,
every forwarded account is checked: if it is the vault, or any SPL token
account whose authority is the mandate PDA, the action is refused with a
committed receipt. An account whose data cannot be read is treated as
controlled. A venue needs the mandate's *signature* to authorise a position; it
never needs the collateral, so nothing legitimate is lost.

**The post-check runs before any receipt commits.** The vault snapshot is
compared immediately after the CPI returns and before gate 13 reads the venue's
report, so a venue that moves the vault cannot buy silence with a refusal.

**The post-check requires the vault to be unchanged**, not merely within the
notional. Gate B venues hold no token custody (`scripts/no-token-custody.sh`),
so any movement is a fault. A custody venue is Gate C work and needs real
accounting, not a tolerance band.

## Consequences

- Gate 14 gains an end-to-end trigger it never had: the rogue venue also
  reports twice the notional it was asked for, and the mandate reverts with
  `PostCheckFailed` rather than writing a receipt for a size nobody authorised.
  The BACKLOG item "gate 14 is unit-tested but never exercised end to end" is
  closed for the over-fill clause; the snapshot clause is now unreachable *by
  construction* (gate 15 prevents its precondition), which is the correct state
  for defence in depth.
- `BlockReason` grows to 18 variants. Append-only holds: 0–16 keep their
  numbers and 17 is new.
- The deployed devnet program `25CdYaZeB18QvUR7cTyZPgTZPNREb7t6xL8zmk1eXAU6`
  carries the vulnerability until it is upgraded. Recorded in FACTS and
  SECURITY; no mandate on it holds value, and the only venue in its registry is
  `demo_perps`, which takes no custody.
- **Gate C question, added to ADR-003's list**: a real venue *will* ask for a
  token account to take collateral from. The moment one does, gate 15 must be
  replaced by something stronger than "never forward the vault" — a delegate
  with an exact allowance, or a separate settlement account funded per action.
  Handing a custody venue the vault plus the vault's authority is the shape of
  this bug, and it must not be reintroduced under a deadline.

## Alternatives considered

- **Sign the venue CPI with a distinct `venue_auth` PDA** that has no authority
  over the vault. Structurally the strongest fix: the signature handed out
  simply cannot move money. Rejected for now only because the venue keys its
  position accounts to the mandate PDA, so this changes the adapter ABI and
  `demo_perps`'s account derivation. It is the right answer if a Gate C venue
  needs a collateral account, and is recorded in BACKLOG as such.
- **Deny-list the vault key only.** Cheaper, but a second mandate-owned token
  account would walk straight through. The authority check costs one
  comparison per forwarded account.
- **Post-check only, no gate 15.** Detection without prevention: the theft
  would revert, but only if every path that can commit a receipt remembers to
  check first — which is exactly the mistake that made this exploitable.

## How this was found

By writing the attacker. The property "a venue cannot touch the vault" had been
asserted in prose in three places and tested nowhere. The test that was supposed
to confirm it disproved it instead, which is the argument for keeping
`test-rogue-venue` in the tree and pointing it at every future adapter.
