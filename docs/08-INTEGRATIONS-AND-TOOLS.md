# 08 — Integrations and Tools
Markov Book · 31 August 2026 · v0.1

Everything the system touches. Three columns matter: **why it exists**, **is it required for Gate B**, and **what has to be verified before a line of code depends on it**.

Verification rule: no version, program ID, or endpoint in this file is authoritative. Each one carries a `FACTS key`. Week 0 verifies it, writes the resolved value with a date and a source into `docs/FACTS.md`, and code reads it from there. If a verification fails, stop and report — do not substitute a guess.

---

## 1. Chain and program toolchain

| Tool | Role | Gate B | Verify in Week 0 | FACTS key |
|---|---|---|---|---|
| Solana **devnet** | the only cluster | required | cluster health, faucet limits, devnet USDC-d mint we will use | `CLUSTER`, `USDC_D_MINT` |
| **Anchor** | program framework, IDL, event codecs | required | exact version; Anchor v1.0.0 shipped 2 Apr 2026 as the first stable major, v1.0.1 is a patch — pin one and record it | `ANCHOR_VERSION` |
| **Agave / solana CLI** | keygen, deploy, program dump | required | version compatible with the pinned Anchor; note Anchor v1.0 removed the hard Solana CLI dependency, but deploys and keygen still want it | `SOLANA_CLI_VERSION` |
| **Rust toolchain** | programs + all services | required | pinned in `rust-toolchain.toml`; must build SBF for the pinned Agave | `RUST_VERSION` |
| **LiteSVM** | fast in-process program tests | required | it is an Anchor v1.0 default test target; confirm the crate version that matches | `LITESVM_VERSION` |
| **Surfpool** | local validator / mainnet-fork daemon, Anchor v1.0 default | recommended | whether it is needed at all for a devnet-only, mock-venue build; if it only duplicates LiteSVM, skip it | `SURFPOOL_VERSION` |
| **AVM** | Anchor version manager | recommended | install path in CI | — |
| **Codama** | generate the TS client from the IDL | required for web | generator version and that it emits a Kit-compatible client | `CODAMA_VERSION` |

**Pinned-toolchain rule.** `Anchor.toml [toolchain]`, `rust-toolchain.toml`, and the CI image agree on one set of versions. A developer machine that differs is a bug report, not a workaround.

## 2. Client SDK and wallet

| Tool | Role | Gate B | Note |
|---|---|---|---|
| **`@solana/kit`** | JS/TS SDK for the web app | required | Kit is the successor to `@solana/web3.js` v2; new apps build directly on Kit. Verify the current major before pinning. |
| **`@solana/kit-plugin-wallet`** | Wallet Standard discovery | required | Kit does **not** use `@solana/wallet-adapter`; wallet-adapter is superseded for new work. Do not pull in the legacy adapter bundle. |
| **`@solana/react`** | React bindings / hooks over the Kit client | required | |
| **`@solana/web3.js` v1 or v3-rc** | legacy interop | **not used** | only relevant if a dependency forces it; if one does, record why in FACTS |

**Decision:** the browser signs with the owner's own Wallet Standard wallet. There is no custodial path, no session key, no relayer, no "sign once and we handle the rest" in Gate B. Every owner verb is a transaction the owner signs.

## 3. Price / oracle

| Tool | Role | Gate B | Verify |
|---|---|---|---|
| **Pyth Hermes** (off-chain) | pull the SOL/USD price payload for marking | required | current Hermes endpoint, the SOL/USD feed id, and the rate limit |
| **Pyth Solana receiver / price feed accounts** | on-chain mark for `demo_perps` freshness checks | required | Pyth Core on Solana had an upgrade dated 18 Aug 2026 — confirm the **current** receiver and price-feed program addresses and whether devnet is on the upgraded contracts before hardcoding anything |
| **`pyth-solana-receiver-sdk`** (Rust) | read a price update inside the program | required if reading Pyth directly on-chain | **known integration risk:** published compatibility has trailed Anchor releases (documented as compatible up to 0.31.1 at time of writing). If it does not build against the pinned Anchor v1.x, use the fallback below and record the decision. |
| **Fallback: house `mark-poster` + `MarkAccount`** | a tiny signed-by-us mark account with `price`, `expo`, `slot`, `source` | required as fallback | keeps the *freshness gate real* even if the SDK is not compatible. Clearly labelled on the dashboard as a house-posted devnet mark, not an oracle claim. |

**Why the fallback is acceptable and how to say it honestly:** devnet money is fake and the venue is a mock. A house-posted mark is fine *as long as the page says so* and the freshness gate is genuinely enforced on-chain. It becomes unacceptable the moment real capital or a real venue appears — that is a Gate C blocker, written into `SECURITY.md` now.

## 4. RPC and streaming

| Tool | Role | Gate B | Note |
|---|---|---|---|
| **Primary RPC provider** (Helius / Triton / QuickNode class) | agent, indexer, submitter | required | devnet plan, rate limits, WebSocket `logsSubscribe` support and its subscription cap |
| **Public `api.devnet.solana.com`** | fallback only | required | rate-limited; enough to keep reads alive, not enough to run on |
| **Provider gRPC stream** (Laserstream / Yellowstone class) | lower-latency ingestion | optional, later | the indexer is written behind an ingestion trait so this slots in without touching the parser |
| **Priority fee source** | land txs during congestion | optional | devnet rarely needs it; the code path exists with a constant, provider API later |

**Decision:** Gate B ingests with plain WebSocket `logsSubscribe` at `confirmed` plus signature backfill. No vendor-specific streaming until there is a reason. One RPC vendor outage must not be able to make the dashboard lie — hence the parity job.

## 5. Off-chain services

| Tool | Role | Gate B | Alternative considered |
|---|---|---|---|
| **Rust + tokio** | agent, indexer, api, bot | required | TS was faster to write; rejected because the guard and `BlockReason` must be shared types with the program |
| **axum + sqlx** | `data-api` | required | |
| **Postgres 16+** | read model | required | Railway Postgres or Neon; it is a cache, rebuildable from chain |
| **teloxide** | Telegram bot | required for B6 | grammY (TS) rejected for the same one-language reason |
| **`tracing` + JSON logs** | structured logs with `tick_id` | required | |

## 6. Hosting and delivery

| Tool | Role | Gate B | Note |
|---|---|---|---|
| **Railway** | agent, paper, emergency+bot, indexer, api, Postgres | required | separate services, separate env scopes; the emergency key lives in a different service than the operator key |
| **Vercel** | web app | required | preview per PR; production domain is what the tape records |
| **GitHub + Actions** | source, CI, release | required | |
| **Domain + TLS** | `/book` on HTTPS (B1) | required | one apex domain, no mixed content, no localhost anywhere in the tape |

## 7. Frontend stack

| Tool | Role | Gate B | Note |
|---|---|---|---|
| **TanStack Start (Vite + TanStack Router, SSR)** | web app framework | required | chosen to match the existing design source at `markovhq.grok.me`, which is a TanStack Start app — the design ports 1:1 instead of being re-derived |
| **Tailwind CSS v4** | styling | required | the reference stylesheet uses v4 tokens (`--color-*` theme vars, `bg-linear-to-b`); staying on v4 keeps class names identical |
| **Google Fonts: Outfit + JetBrains Mono** | display/body + data | required | exactly the two families the reference loads |
| **lucide-react** | the few icons used | required | reference uses it (moon icon) |
| **zustand** | tiny client store for the blotter | optional | reference uses a small store; local state is fine |

## 8. Quality, safety, and truth tooling

| Tool | Role | Gate B |
|---|---|---|
| `cargo clippy -D warnings`, `cargo fmt --check` | Rust lint/format | required |
| `cargo deny` | dependency licence + advisory check | required |
| `cargo audit` | RustSec advisories | required |
| `eslint` + `prettier` + `tsc --noEmit` | web | required |
| `anchor test` (LiteSVM) | program invariants | required |
| **`scripts/copy-grep.sh`** | B15 enforcement — fails CI if banned words appear in built HTML | required |
| **`scripts/parity-check`** | independent chain count vs API count | required |
| **`scripts/gate-b-verify.sh`** | runs the whole freeze list against hosted URLs and prints red/green | required |
| Playwright | one UI test: withdraw enabled in Active/Paused/Revoked (B10) | required |
| Sentry or equivalent | error tracking for agent + indexer + api + web | recommended |
| Uptime monitor | pings `/health` and `/book` | recommended |

**`copy-grep` banned list (Gate B):** `APY`, `%\s*APY`, `annualized`, `guaranteed`, `risk-free`, `audited`, `Jupiter`, `Drift`, `Velocity`, `Pacifica`, `Hyperliquid`, `mainnet`, `all eleven`. Allowed only inside `docs/` and code comments; a hit in built HTML fails the job. Venue brands are banned in **public copy** until the ADR-03 Week-0 checklist passes for that venue, which cannot happen in Gate B.

## 9. Explicitly not integrated (and why)

| Not using | Why not, in one line |
|---|---|
| Jupiter Perps / any live venue | ADR-03 checklist has not passed; naming one in copy is a B15 failure |
| Pooled vault / share accounting | ADR-02; a pooled NAV is a different legal object and a single drain target |
| Any token, points, or leaderboard | ADR-05 non-goals |
| x402 facilitator settlement | ADR-11 defers it; spend caps exist in-program without it |
| MCP server | side tool at best; not the MVP (`03-MVP-DEVNET` §8) |
| Redis / queue / websockets to the browser | no load justifies them; a 5s poll meets the 10s requirement |
| Wallet-adapter legacy bundle | superseded by Kit + Wallet Standard; pulls deprecated transitive deps |
| An LLM anywhere near signing | ADR-04 |
| Analytics that track wallets | there is no reason to profile a depositor to close Gate B |

## 10. Accounts and access to open in Week 0

A boring list that blocks everything if it is late:

1. RPC provider account with a devnet plan and a WebSocket endpoint.
2. Railway project `markov-devnet` with six services and one Postgres.
3. Vercel project bound to the repo, production domain attached.
4. GitHub repo with branch protection and Actions enabled; secrets scoped per environment.
5. Telegram bot token from BotFather, plus the chat/user allowlist for `/pause` and `/revoke`.
6. Three keypairs generated on a machine that is not CI: `owner-demo`, `operator`, `emergency`. Public keys go into FACTS; private keys go into service env only.
7. Error-tracking project (one DSN per service).
8. Price source access confirmed (endpoint reachable, feed id resolved).

## 11. Cost sketch (devnet, monthly, order of magnitude)

| Line | Estimate |
|---|---|
| RPC devnet plan | free tier to low tens of dollars |
| Railway (6 small services + Postgres) | low tens of dollars |
| Vercel hobby/pro | free to low tens |
| Domain | annual, negligible monthly |
| Error tracking | free tier |
| **Total** | **tens of dollars a month** — Gate B is not an infrastructure problem |

Devnet SOL is free from the faucet; budget a top-up runbook anyway, because an agent that cannot pay fees looks exactly like a broken agent.
