# 09 — Repo Structure
Markov Book · 31 August 2026 · v0.2

One repository. One Cargo workspace for everything that touches a key or a log, one pnpm workspace for everything that renders. Two package managers, no third.

---

## 1. Tree

```
markov-book/
├─ README.md
├─ Anchor.toml                     # [toolchain] pins anchor + solana versions
├─ Cargo.toml                      # rust workspace root
├─ rust-toolchain.toml
├─ package.json                    # pnpm workspace root
├─ pnpm-workspace.yaml
├─ turbo.json
├─ .env.example                    # every var, no values
├─ deny.toml                       # cargo-deny policy
│
├─ programs/
│  ├─ markov-mandate/              # the lock
│  │  ├─ Cargo.toml
│  │  └─ src/
│  │     ├─ lib.rs                 # program entry, instruction handlers
│  │     ├─ state/
│  │     │  ├─ mandate.rs          # Mandate account + state enum
│  │     │  ├─ policy.rs           # Policy struct + tighten-only diff
│  │     │  └─ registry.rs         # AdapterRegistry, global halt flag
│  │     ├─ gates.rs               # the ladder, in order, one fn per gate
│  │     ├─ receipts.rs            # ActionReceipt / RefusalReceipt emit
│  │     ├─ cpi/
│  │     │  └─ venue.rs            # adapter ABI caller
│  │     └─ errors.rs              # hard errors (NOT BlockReason)
│  └─ demo-perps/                  # the mock venue behind the real trait
│     └─ src/
│        ├─ lib.rs                 # open/close/increase/reduce/positions/mark
│        ├─ state.rs               # Position, Market, MarkAccount
│        └─ mark.rs                # freshness rules
│
├─ crates/
│  ├─ markov-types/                # shared with programs AND services
│  │  └─ src/{block_reason.rs, action.rs, policy_view.rs, ids.rs}
│  ├─ markov-guard/                # PURE. no I/O, no clock, no network.
│  │  └─ src/{lib.rs, rules/*.rs, fixtures/*.json}
│  ├─ markov-venue/                # VenueAdapter trait + demo_perps client
│  ├─ markov-chain/                # rpc, tx build, sign, send, confirm, retry
│  ├─ markov-marks/                # MarkSource trait: hermes | onchain | replay
│  ├─ book-one/                    # THE AGENT (binary)
│  │  └─ src/
│  │     ├─ main.rs                # mode: devnet | shadow
│  │     ├─ scheduler.rs           # 60s tick, jitter, backoff
│  │     ├─ sidecar.rs             # RegimeSource; Gate B stub returns chop
│  │     ├─ core.rs                # deterministic proposer -> Intent
│  │     ├─ submitter.rs           # idempotent send, intent_id, retries
│  │     ├─ redteam.rs             # forced intents that must be refused on chain
│  │     ├─ paper.rs               # shadow writer -> paper/YYYY-MM-DD.md
│  │     └─ metrics.rs
│  ├─ indexer/                     # logs -> Postgres
│  ├─ data-api/                    # axum read API
│  └─ bot/                         # telegram, emergency key only
│
├─ apps/
│  └─ web/                         # TanStack Start + Tailwind v4
│     ├─ app/routes/{index,book,receipts,paper}.tsx
│     ├─ app/components/{blotter,receipt-row,stat,circuit-chip,owner-verbs}.tsx
│     ├─ app/lib/{client.ts,program.ts,api.ts}
│     └─ app/styles/tokens.css     # the tokens in 13-FRONTEND-SPEC
│
├─ packages/
│  └─ sdk/                         # Codama-generated client from the IDL, checked in
│
├─ tests/
│  ├─ program/                     # LiteSVM: one file per gate
│  ├─ integration/                 # devnet smoke: fund -> act -> refuse -> revoke -> withdraw
│  └─ e2e/                         # Playwright: B10 and the tape beats
│
├─ paper/                          # YYYY-MM-DD.md, one per day, ugly days kept
│
├─ docs/
│  ├─ FACTS.md                     # single source of truth, dated, sourced
│  ├─ SECURITY.md                  # B13
│  ├─ STATUS.md
│  ├─ SESSION_LOG.md
│  ├─ BACKLOG.md
│  ├─ adr/ADR-0xx-*.md
│  ├─ runbooks/{key-rotation,indexer-rebuild,agent-halt,faucet-topup}.md
│  └─ demo/GATE-B.md               # B14: the eight proofs
│
└─ scripts/
   ├─ copy-grep.sh                 # B15
   ├─ parity-check.rs              # chain count vs API count
   ├─ gate-b-verify.sh             # runs B1..B15 against hosted URLs
   ├─ redteam-tick.sh              # manual force of one refusal
   └─ keys/derive-pubkeys.sh       # prints pubkeys for FACTS, never privates
```

## 2. Package boundaries (enforced, not suggested)

| Rule | Enforced by |
|---|---|
| `markov-guard` has **no** dependency on tokio, reqwest, solana-client, or `std::time::SystemTime` | `cargo deny` ban list + a compile test |
| `markov-types` compiles for SBF and for host | it is a dependency of both `programs/*` and `crates/*` |
| `book-one` never imports `indexer` or `data-api` | workspace dependency graph review in CI |
| `data-api` is read-only: no writer role on its DB user | separate Postgres role, checked in the deploy runbook |
| `apps/web` never imports a Rust crate; it talks HTTP + IDL | obvious, but it stops "just one helper" |
| Nothing outside `crates/bot` may reference the emergency key | grep in CI + separate env scope |

## 3. Branching and commits

- `main` is deployable and protected. No direct pushes.
- Feature branches named `seat/short-slug` — `agents/guard-daily-loss`, `protocol/demo-perps-mark`.
- PR labels: `gate-b`, `out-of-scope`, `docs`, `truth`. An `out-of-scope` label means it merges only after Gate B closes, or it goes to `BACKLOG.md`.
- Conventional-ish commits with the Gate B item in the subject when relevant: `feat(program): OverTxCap refusal receipt (B5)`.
- Every PR that changes a public number updates `docs/FACTS.md` in the same PR. A PR that adds a claim without a FACTS row does not merge.

## 4. Environment matrix

`.env.example` lists all of these with empty values. No secret is ever in the repo, in CI logs, or in an issue.

| Var | Used by | Notes |
|---|---|---|
| `CLUSTER` | all | `devnet` only in Gate B |
| `RPC_HTTP_URL`, `RPC_WS_URL` | agent, indexer, bot | primary provider |
| `RPC_HTTP_FALLBACK` | agent, indexer | public devnet |
| `PROGRAM_ID` | all | from FACTS |
| `DEMO_PERPS_ID` | agent, indexer | from FACTS |
| `USDC_D_MINT`, `SOL_D_MINT` | agent, web | from FACTS |
| `OPERATOR_KEYPAIR` | agent only | base58 or file path; never in bot/api |
| `EMERGENCY_KEYPAIR` | bot service only | never in agent |
| `OWNER_DEMO_KEYPAIR` | demo scripts only | for the recorded tape |
| `MARK_SOURCE` | agent, paper | `hermes` \| `onchain` \| `replay` |
| `HERMES_URL`, `HERMES_API_KEY`, `SOL_USD_FEED_ID` | agent, paper | from FACTS. Key required after 26 Aug 2026 Pyth cutover. Empty key + `MARK_SOURCE=hermes` must refuse to boot |
| `TICK_SECONDS` | agent | default 60, floor 60 in Gate B |
| `REDTEAM_ENABLED`, `REDTEAM_CRON` | agent | off in `shadow` |
| `DATABASE_URL` | indexer (rw), data-api (ro) | two roles, two URLs |
| `LAG_SLOTS` | api | default 150 |
| `TELEGRAM_BOT_TOKEN`, `TELEGRAM_ALLOWLIST` | bot | |
| `VITE_API_BASE`, `VITE_CLUSTER`, `VITE_PROGRAM_ID` | web | public by nature |
| `SENTRY_DSN_*` | services | one per service |

## 5. Definition of done for any PR

```
- [ ] Tests for the thing you changed, including one failure path
- [ ] cargo fmt, clippy -D warnings, cargo deny clean (or tsc/eslint for web)
- [ ] No new public claim without a FACTS row
- [ ] No banned word in built HTML (copy-grep passes)
- [ ] SESSION_LOG entry: what you verified, how, and what you did not
- [ ] If a pre-flight check failed, the PR says so and stops — no improvised workaround
```

## 6. Seat ownership map

| Path | Seat |
|---|---|
| `programs/**`, `crates/{markov-types,markov-venue,markov-chain}`, `tests/program` | Protocol |
| `crates/{markov-guard,book-one,markov-marks}`, `paper/**` | Agents |
| `apps/web/**`, `packages/sdk`, `crates/bot`, `tests/e2e` | Surfaces |
| `docs/**`, `scripts/{copy-grep,parity-check,gate-b-verify}` | Truth |

Empty seat, no work. Two seats touching one file is a design smell — split the file.
