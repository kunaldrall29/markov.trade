# 14 — Security and Keys
Markov Book · 31 August 2026 · v0.2
This file is the source for `docs/SECURITY.md` (Gate B item **B13**).

---

## 1. The claim we are allowed to make

> On devnet, with an unaudited program and a single upgrade authority, the operator key cannot withdraw, cannot unpause, and cannot widen a cap. The owner can withdraw in every state.

That is the whole security claim. Everything stronger — "secure", "audited", "safe" — is a B15 failure. The wedge is a *runtime bound*, not a rest-time audit, and the copy has to keep saying so.

## 2. Key inventory

| Key | Held by | Can do | Cannot do | Rotation |
|---|---|---|---|---|
| **Owner** | the depositor's own wallet | fund, amend (tighten), pause, **unpause**, revoke, **withdraw** | operate the book | user-owned, not ours |
| **Owner (demo)** | a local file used only to record the tape | same as owner, on the demo mandate only | anything on a real owner's mandate | regenerate after the tape |
| **Operator** | `book-one` service env, Railway | propose actions inside policy | withdraw, unpause, amend up, pause, revoke, halt | documented, tested, ≤ 15 min |
| **Emergency** | `bot`/emergency service env, separate deployment | pause, revoke | unpause, withdraw, trade, amend | documented |
| **Mark poster** | mark-poster job | write price/slot to allowlisted `MarkAccount` | anything else | documented |
| **Upgrade authority** | one key on devnet, offline | deploy | — | mainnet plan: multisig, then freeze |

**Separation is deployment-level, not comment-level.** The operator key must not be readable from the process holding the emergency key, and neither may appear in the web app, CI logs, the indexer, or the API. A grep job in CI fails the build on any keypair-shaped string in the repo.

## 3. Threat model

| Adversary | Capability | What they get | Mitigation |
|---|---|---|---|
| Operator key theft | signs any proposal | nothing beyond policy — capped notional, allowlisted venue, no withdraw | gates; owner revokes; rotate key |
| Emergency key theft | pause/revoke spam | a stopped book; owners still withdraw | separate service; alert on unexpected state change |
| Mark-poster key theft | posts a wrong price | mock fills at a wrong mark on devnet; no vault drain (mock holds no custody) | freshness + bounds check; rotate; Gate C blocker for real venue |
| Malicious upgrade | replaces the program | total loss — the Drift-shaped criticism pointed at ourselves | **documented, not hidden**; devnet only; multisig plan written before any mainnet dollar |
| RPC provider lies about state | stale/forged reads | agent acts on wrong state | reads are advisory; the program re-checks everything on chain |
| Indexer compromise | fake receipts in the API | a lying dashboard | parity job against an independent chain query; explorer link on every row |
| Malicious depositor | funds a mandate and grieves | more SMA fan-out cost | allowlist owners in the guarded window; not a Gate B problem |
| Front-end supply chain | injected script steals a signature | one owner's transaction | pinned deps, `cargo deny` / `pnpm audit` in CI, no third-party analytics, CSP |

## 4. Accepted risks (written down, not buried)

1. **Single upgrade authority on devnet.** One key can replace the program. Accepted for Gate B because there is no real money and the alternative is theatre. Mainnet requires a multisig and a written freeze plan; that plan is a Phase-1 gate, not a Gate B one.
2. **No audit.** The program is unaudited. Every page says unaudited.
3. **Delta, gross, and daily-loss halt are enforced off-chain in v0** (ADR-05). The dashboard labels them. Anyone who believes they are on-chain has been misled, so the label is a correctness requirement, not a decoration.
4. **The mark is house-posted on devnet** if the oracle SDK does not build against the pinned Anchor. As of 31 Aug 2026, `pyth-solana-receiver-sdk` is documented only through Anchor 0.31.1, so on an Anchor 1.x pin this is the expected path, not a surprise. Labelled on the page as a house mark.
5. **`demo_perps` is a mock.** Fills are deterministic and generous compared to a real venue. No PnL number from it is evidence about a real book.
6. **The Telegram bot holds the emergency key**, so anyone who takes over the bot can stop the book. They cannot take anything. Allowlist the chat ids and treat bot takeover as a pause event, not a loss event.

## 5. Operational rules

- Keys are generated on a workstation, never in CI, never in a browser.
- Private keys live only in the runtime env of exactly one service. No key in a repo, an issue, a screenshot, a doc, or a chat message.
- `scripts/keys/derive-pubkeys.sh` prints **public** keys for FACTS. There is no script that prints a private key.
- Rotation drill runs once before Gate B closes: rotate the operator key, confirm the old key's next proposal is refused with `NotOperator`, and record both signatures. That drill is the proof the separation is real.
- Every owner-visible destructive action confirms in plain words and names what stays possible ("withdraw stays on").

## 6. `docs/SECURITY.md` template (B13)

```markdown
# SECURITY — Markov Book (devnet)

Status: devnet. Unaudited. Marked PnL. Not a promised rate.

## Keys
| Key | Pubkey | Held by | Can | Cannot |
| Owner (demo) | <FACTS:OWNER_DEMO_PUBKEY> | tape recording only | ... | ... |
| Operator | <FACTS:OPERATOR_PUBKEY> | book-one service | propose within policy | withdraw, unpause, amend up |
| Emergency | <FACTS:EMERGENCY_PUBKEY> | emergency/bot service | pause, revoke | unpause, withdraw, trade |
| Upgrade authority | <FACTS:UPGRADE_AUTHORITY> | single key, offline | deploy | — |

## Accepted risks
1. Single-key upgrade authority on devnet. Accepted. Mainnet plan: multisig, then freeze.
2. Unaudited program.
3. Delta / gross / daily-loss halt enforced off-chain in v0; on-chain in Phase 1.
4. Mark source is <FACTS:MARK_SOURCE> on devnet.
5. demo_perps is a mock venue with no token custody.

## Invariants we test every build
- owner_withdraw succeeds in Active, Paused, Revoked, Expired.
- Only the owner can unpause.
- The operator cannot withdraw, unpause, or widen a cap.
- Every allow and every block that reaches the program emits a receipt.
- BlockReason discriminants are append-only.

## Reporting
<contact>. No bug bounty on devnet. We will publish what we fix.
```

## 7. What would make us stop

- A single out-of-policy execution on any cluster.
- A refusal that does not produce a receipt.
- A `guard_divergence_total` above zero that we cannot explain.
- A withdraw that fails in any state for any reason other than an empty vault.

Any one of these halts the book (global halt), and the incident is written up before the next feature.
