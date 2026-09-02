# Corrections — Engineering Pack v0.1 → v0.2
Markov Book · 31 August 2026

What we are deciding: the architecture pack is the build contract for Gate B. v0.1 was directionally right and ADR-aligned. It is not a live product claim. A few rows were wrong enough to ship a lie or a stuck Week 0.

Constraint: ADR-05 (reuse the deployed program; dump IDL first), ADR-03 (no named venue), ADR-08 (devnet labels).

---

## Verdict

| Area | v0.1 | v0.2 |
|---|---|---|
| Product shape, ADRs, Gate B freeze, SMA, demo_perps, receipts, no LLM in signing, no marketplace | Correct | Unchanged |
| `execute_venue_action` ladder in `07` vs `10` | **Conflict** | Unified on `10` |
| Anchor “pin 1.0.0 / 1.0.1” | Stale guess | PENDING until `anchor --version`; latest *observed* 31 Aug 2026 is **1.1.2** (26 Jun 2026), recommended Agave **3.1.10** |
| LiteSVM vs Surfpool | Treated as near-duplicates | Different tiers. LiteSVM = in-process unit. Surfpool = default `anchor test` / `anchor localnet` runner |
| Pyth Core upgrade date | “18 Aug 2026” | **26 Aug 2026, 16:00 UTC** |
| `pyth-solana-receiver-sdk` vs Anchor 1.x | “known risk, try then fallback” | Documented compat stops at **Anchor 0.31.1**. On an Anchor 1.x pin, **do not fight the SDK**. ADR-013 + house `MarkAccount` is the expected Gate B path |
| Hermes | Endpoint + feed id only | **API key required** after the 26 Aug cutover |
| `LAG_SLOTS` 150 ≈ 60s | Assumed 400 ms slots | Wall-clock from measured **devnet** slot time in FACTS. Mainnet slot time moved in Aug 2026; do not quote 60s as a fact |
| `@solana/kit` | “verify current major” | Correct instinct. Latest *observed* 27 Aug 2026: **v8.1.0**. Still PENDING in FACTS |
| Landing blotter tile `funded` | Drifted from spec | `funding 7d` |
| solana.new founder mode | Not mentioned | Explicitly **not** a Gate B input. Idea/Build/Launch/Raise is a side tool. Do not generate the mandate program there |

Nothing in v0.1 invented a live venue, an APY, a pool, or an LLM signer. Those locks hold.

---

## Gate ladder (binding)

`07` omitted two gates that `10` already specified. The program spec wins.

```
1  GlobalHalt
2  state → Paused | Revoked | Expired
3  Unauthorized        (chain name; the pack said NotOperator)
4  DuplicateIntent
5  ProgramNotAllowed   (policy.venues AND registry.adapters; chain name; the pack said VenueNotAllowed)
6  TokenNotAllowed
7  ActionNotAllowed
8  OverTxCap
9  OverDailyCap
10 OverSpendCap / OverSpendDailyCap
11 SlippageExceeded
12 StaleOracle         (seconds since publish_time, ADR-003)
13 VenueRejected       (CPI error)
14 PostCheckFailed     (hard Err; transaction reverts)
```

Gates 1–13 emit a `RefusalReceipt` and return `Ok(())`. Gate 14 is the only refusal that is allowed to be an `Err`.

Historical eleven (append-only; **amended 2026-09-02 from the chain, `docs/FACTS.md`** — the deployed program has no on-chain IDL, so these come from 20 on-chain `ActionRefused` payloads and the checked-in IDL):

`0 OverTxCap, 1 OverDailyCap, 2 OverSpendCap, 3 OverSpendDailyCap, 4 ProgramNotAllowed, 5 TokenNotAllowed, 6 SlippageExceeded, 7 Expired, 8 Paused, 9 Revoked, 10 Unauthorized`

New in Gate B (append at end of deployed enum, never reorder):

`11 StaleOracle, 12 ActionNotAllowed, 13 DuplicateIntent, 14 GlobalHalt, 15 VenueRejected, 16 PostCheckFailed`

Plus off-chain-only guard reasons (not on-chain discriminants in v0): `DeltaBandExceeded`, `GrossExceeded`, `DailyLossHalt`, `GuardInternal`.

---

## What was already correct (do not reopen)

- House book first. Marketplace is Phase 3.
- SMA, not pooled NAV.
- `owner_withdraw` in every state. Unpause owner-only. Emergency = pause/revoke only.
- `demo_perps` holds zero token custody.
- Delta / gross / daily-loss off-chain in v0 (ADR-05).
- Redteam is the only sanctioned local-veto bypass; it cannot skip a program gate.
- Kit + Wallet Standard. No legacy wallet-adapter bundle.
- `copy-grep` is claim-shaped so “No APY on this page.” does not fail the build.
- Paper ≥7 consecutive days (B12). The older `03-MVP` “≥3 days” line is superseded by `GATE-B.md`.
- Colosseum 28 Sep – 2 Nov 2026. Gate B closes 27 Sep or is disclosed open.

---

## Smallest next action

P00 → P01 on a machine that can print `anchor --version`, `solana --version`, `rustc --version`. Dump the **deployed** `markov-mandate` IDL before P02. Do not invent a program ID.
