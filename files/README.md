# Markov Book — Engineering Pack
**Date:** 31 August 2026 · v0.1
**Scope:** everything needed to close **Gate B** (27 Sep 2026) and nothing that belongs to Gate C, mainnet, or Phase 3.
**Inputs:** `00-INDEX` … `06-PMF` planning pack, the Gate B freeze list, and the live design at `markovhq.grok.me`.

**Rule inherited from the planning pack:** claims here are design intent. Nothing is a live product claim. Every version number, program ID, and API shape below is **to be verified in Week 0 and written into `docs/FACTS.md`** before it is used in code. Where this pack states a fact from the outside world, it is dated and sourced. Where it states a decision, it is labelled a decision.

---

## Documents

| # | File | What it answers |
|---|---|---|
| 07 | `docs/07-TECH-ARCHITECTURE.md` | What the system is, trust boundaries, gate order, every diagram |
| 08 | `docs/08-INTEGRATIONS-AND-TOOLS.md` | Every integration and tool, with required / optional / later |
| 09 | `docs/09-REPO-STRUCTURE.md` | Monorepo tree, package boundaries, env matrix, conventions |
| 10 | `docs/10-PROGRAM-SPEC.md` | On-chain: accounts, instructions, gates, BlockReason, adapter ABI |
| 11 | `docs/11-AGENT-SPEC.md` | `book-one`: sidecar / core / guard / submitter / redteam / paper |
| 12 | `docs/12-DATA-AND-API-SPEC.md` | Indexer, schema, endpoints, `chainReady`, parity job |
| 13 | `docs/13-FRONTEND-SPEC.md` | Design system lifted from the live site, `/book` spec, copy rules |
| 14 | `docs/14-SECURITY-AND-KEYS.md` | Key custody, threat model, `SECURITY.md` template |
| 15 | `docs/15-TESTING-CI-OBSERVABILITY.md` | Test matrix, CI, metrics, alerts, runbooks |
| 16 | `docs/16-GATE-B-TRACEABILITY.md` | B1–B15 → component → test → artifact → FACTS key |
| 17 | `docs/17-FACTS-TEMPLATE.md` | `docs/FACTS.md` skeleton and the verification log format |

## Build prompts (one file each, run in order)

| # | File | Seat | Gate B items |
|---|---|---|---|
| P00 | `prompts/P00-conventions.md` | all | preamble every other prompt inherits — read first |
| P01 | `prompts/P01-repo-bootstrap.md` | Protocol | scaffold, CI, FACTS |
| P02 | `prompts/P02-program-core.md` | Protocol | B2 B8 B10-support |
| P03 | `prompts/P03-venue-adapter-trait.md` | Protocol | B11 |
| P04 | `prompts/P04-demo-perps.md` | Protocol | B11 |
| P05 | `prompts/P05-risk-guard.md` | Agents | B3 |
| P06 | `prompts/P06-book-core.md` | Agents | B3 B4 |
| P07 | `prompts/P07-agent-runtime.md` | Agents | B3 B4 B5 B7 |
| P08 | `prompts/P08-paper-runner.md` | Agents | B12 |
| P09 | `prompts/P09-indexer.md` | Protocol | B9 |
| P10 | `prompts/P10-data-api.md` | Protocol | B9 |
| P11 | `prompts/P11-book-page.md` | Surfaces | B1 B2 B6 B8 B10 |
| P12 | `prompts/P12-landing-page.md` | Surfaces | B1 B15 |
| P13 | `prompts/P13-telegram-bot.md` | Surfaces | B6 |
| P14 | `prompts/P14-observability-ci.md` | Truth | B9 B15 |
| P15 | `prompts/P15-gate-b-close.md` | Truth | B13 B14 B15 + close ritual |

## Working script

`scripts/copy-grep.sh` — B15, written and self-tested against the real page. It is claim-shaped, not token-shaped: `No APY on this page.`, `What's the APY?`, `unaudited`, and `April 2026` all contain banned substrings and are all correct copy, so a naive word list fails the build on the page's own honesty. Verified clean on the landing page, and verified red when `12% APY, guaranteed` or a venue brand is injected.

## Landing page

`landing/index.html` — single-file, no build step, production-deployable. Design tokens, type, motion, and structure taken from `markovhq.grok.me` (fetched 31 Aug 2026). `landing/DESIGN-NOTES.md` records exactly what was lifted and what changed.

## Reading order for a new seat

1. `07` architecture → 2. the spec for your seat (`10`/`11`/`12`/`13`) → 3. `prompts/P00` → 4. your prompt file.
Nobody writes code before `docs/FACTS.md` has the rows their prompt needs.
