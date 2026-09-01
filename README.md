# Markov Book — Engineering Pack
**Date:** 31 August 2026 · v0.2
**Scope:** everything needed to close **Gate B** (27 Sep 2026) and nothing that belongs to Gate C, mainnet, or Phase 3.
**Inputs:** `00-INDEX` … `06-PMF` planning pack, `GATE-B.md`, the live design at `markovhq.grok.me` (fetched 31 Aug 2026).
**Supersedes:** engineering pack v0.1. See `CORRECTIONS.md`.

**Rule:** claims here are design intent. Nothing is a live product claim. Every version number, program ID, and API shape is **verified in Week 0 and written into `docs/FACTS.md`** before code depends on it. Observed-as-of dates below are not pins.

**solana.new:** founder-mode Idea / Build / Launch / Raise is a side tool. It is not a source of IDs, not a substitute for P02, and not a Gate B dependency.

---

## Documents

| # | File | What it answers |
|---|---|---|
| — | `CORRECTIONS.md` | What v0.1 got wrong |
| 07 | `docs/07-TECH-ARCHITECTURE.md` | System, trust boundaries, **unified** gate order, topology |
| 08 | `docs/08-INTEGRATIONS-AND-TOOLS.md` | Every integration, required / optional / later, Week-0 verify |
| 09 | `docs/09-REPO-STRUCTURE.md` | Monorepo tree, package boundaries, env matrix |
| 10 | `docs/10-PROGRAM-SPEC.md` | On-chain: accounts, instructions, gates, BlockReason, adapter ABI |
| 11 | `docs/11-AGENT-SPEC.md` | `book-one`: sidecar / core / guard / submitter / redteam / paper |
| 12 | `docs/12-DATA-AND-API-SPEC.md` | Indexer, schema, endpoints, `chainReady`, parity job |
| 13 | `docs/13-FRONTEND-SPEC.md` | Design system from the live site, `/book` spec, copy rules |
| 14 | `docs/14-SECURITY-AND-KEYS.md` | Key custody, threat model, `SECURITY.md` template |
| 15 | `docs/15-TESTING-CI-OBSERVABILITY.md` | Test matrix, CI, metrics, alerts, runbooks |
| 16 | `docs/16-GATE-B-TRACEABILITY.md` | B1–B15 → component → test → artifact → FACTS key |
| 17 | `docs/17-FACTS-TEMPLATE.md` | `docs/FACTS.md` skeleton and verification log format |

## Build prompts (run in order)

| # | File | Seat | Gate B items |
|---|---|---|---|
| P00 | `prompts/P00-conventions.md` | all | preamble — read first |
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

`scripts/copy-grep.sh` — B15. Claim-shaped, not token-shaped.

## Landing page

`landing/index.html` — static, tokens and structure from `markovhq.grok.me`. `landing/DESIGN-NOTES.md` records what was lifted.

## Reading order for a new seat

1. `CORRECTIONS.md` → 2. `07` architecture → 3. the spec for your seat (`10`/`11`/`12`/`13`) → 4. `prompts/P00` → 5. your prompt.
Nobody writes code before `docs/FACTS.md` has the rows their prompt needs.
