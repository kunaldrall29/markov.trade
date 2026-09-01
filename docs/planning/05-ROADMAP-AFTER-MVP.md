# 05 — After the Devnet MVP
Markov Book · 30 August 2026 · v0.1

Phases are gated. A phase does not start because a calendar said so.

## Phase 0 — Devnet Book One (now → ~end Oct)

Done when `03-MVP-DEVNET.md` gate list is green and paper log exists.

**Public artifact:** hosted dashboard, receipt feed, 90s tape, paper folder.

## Phase 1 — Real venue, still fake or tiny money (~Nov)

**Goal:** Book One talks to one real perp venue without pooling.

- Venue adapter passes ADR-03 checklist.
- Shadow book and live-venue paper run in parallel for ≥14 days.
- Guarded mainnet *only if* all of these hold:
  - scoped review of the policy path (not a full audit yet)
  - per-mandate cap (start at $100–$500)
  - allowlist of owners (start at ≤20)
  - total ceiling (start at $5,000)
  - kill switch live
  - upgrade authority plan written (multisig)
  - zero out-of-policy executions in the window

**Not in Phase 1:** performance fees charged on-chain, open deposits, a second venue, a token.

## Phase 2 — A book that can lose and still be honest (~Dec–Q1)

**Goal:** enough history that a stranger can decide.

- 30–90 days of public marks (paper + guarded live).
- On-chain enforcement of net-delta / gross if a trusted mark exists.
- Batching design (not necessarily shipped) so SMA fan-out does not die at 50 owners.
- Decision gate: **pool or stay SMA.** Pooling requires its own review and a different ToS. Default remains SMA.
- **Equity book (design only):** allowed as a Phase-2 design if a venue lists a tokenized-equity name and passes ADR-03. Not a `/book` headline. Not Gate B. We do not issue the underlying.

**Kill criteria for the whole product (any one):**

- hedge error stays outside band for 14 days and cannot be explained
- a single out-of-policy fill on mainnet
- daily-loss halt fires so often the book is a paused museum
- nobody who is not the team will deposit even $100 after seeing the curve

## Phase 3 — Open the platform (only after Phase 2 is not embarrassing)

This is the old Float idea, earned.

- External operators publish a strategy template.
- Owners subscribe with tighten-only overrides.
- House Book One remains listed as one strategy among n.
- MCP / SDK become the B2B motion: “run your book under a mandate.”
- Score / reputation from receipts including refusals.

**Do not skip to Phase 3.** That is the failure mode we already lived in documentation form.

## Phase 4 — Credit and fees (venture layer, not this year unless forced)

- In-program fee switch: protocol share of operator performance, marketplace fee.
- Bonds / slashing only after refusal graphs exist.
- Token utility specified next to fee settlement, not before.

## Decision calendar (planning, not commitments)

| When | Decision |
|---|---|
| End of paper week 2 | Is there a book at all? If no, do not pitch Book One as yield. Stay a mandate toolkit. |
| End of Colosseum | Submit what is real. If only demo_perps exists, say that. |
| Dec | Raise / no-raise. Evidence: paper + guarded window + waitlist quality, not follower count. |
| After first real-venue month | Pool vs SMA. Default SMA. |

## What “move forward” looks like in operations

Weekly, 15 minutes, written in METRICS.md:

- paper / live return, DD, hedge error
- receipts and refusals (house vs anyone else)
- depositors (unique owners, Telegram-linked if we have them)
- one sentence: did the book do its job?

If the row is skipped two weeks in a row, the project is a slide deck again.
