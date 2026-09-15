# 00 — Build Conventions (inherited by every Markov build prompt)

Read this first. Every other prompt in `prompts/` assumes these rules. Where a prompt and this file conflict, this file wins.

## 1. What "production-ready, mainnet-ready" means here

- **Mainnet is the target for every component.** Solana mainnet-beta for the Markov program, Jupiter, the Subscriptions & Allowances program, Phoenix, Pacifica and Drift. Devnet, Pacifica testnet and LiteSVM exist for CI and rehearsal only.
- **No mock data anywhere a user or an operator can see.** No fixture markets, no illustrative balances, no synthetic prices, no "demo mode" toggles. Empty states say what is missing and why. The only permitted non-live inputs are (a) recorded real chain data used as unit-test fixtures, labelled with the slot they were recorded at, and (b) LiteSVM runs of the real Phoenix program bytes.
- **Small real money, not fake money.** Rehearsals use funded mainnet wallets with caps in the low tens to low hundreds of USD. Every rehearsal transaction is a real transaction with a real receipt.
- **Stale fails closed.** Any decision path without fresh data (per-adapter freshness window) refuses and receipts the refusal. Never "best effort" with old numbers.
- **Caps are code, not policy documents.** Per-account, per-market, per-day and global notional caps are enforced in the program and in the control plane, with the values in a single versioned config that the Terminal displays.
- **Facts come from `docs/FACTS.md`.** Program IDs, mints, endpoints, extension lists and restrictions are copied from there, never typed from memory. A build that needs a fact absent from FACTS stops and adds it with a source and date.

## 2. Environments

| Name | Cluster / backend | Purpose |
| --- | --- | --- |
| `mainnet` | Solana mainnet-beta; Jupiter mainnet; Pacifica mainnet; Drift mainnet; Phoenix mainnet | production and capped rehearsal |
| `rehearsal` | same as mainnet, separate wallets, hard caps | pre-release verification with real money |
| `devnet` | Solana devnet (Markov program, Drift, Subscriptions program, Pyth devnet feeds) | CI, integration tests |
| `pacifica-testnet` | Pacifica testnet API (`test-api.pacifica.fi`) | Pacifica adapter integration tests |
| `litesvm` | local, real Phoenix/Hawkeye program bytes | Phoenix CPI tests |

Config is one typed object per environment in `packages/config` (see §7), never scattered `.env` lookups.

## 3. Verified constants (copy from FACTS; listed here for convenience, FACTS wins on conflict)

- Markov program: **to be deployed**; upgrade authority = Squads multisig (§5).
- Subscriptions & Allowances: `De1egAFMkMWZSN5rYXRj9CAdheBamobVNubTsi9avR44` (mainnet + devnet).
- Phoenix prod `EtrnLzgbS7nMMy5fbD42kXiUzGg8XQzJ972Xtk1cjWih`; Phoenix beta (mainnet) `phDEVv4w6BcfkLrLNeXr8HhhgQxnxziVGXpGPcaadMf`; Hawkeye `RiSeVw3ZjNfsaXPRb4mgaqYaEEt41pNNJoDvVh7pgQj`; Ember `EMBERpYNE6ehWmXymZZS2skiFmCa9V5dp14e1iduM5qy`; Flight `F1ightu9cujFYo34k9CabifLrJT8qzfDVM2Q7BqhJn2W`; REST `https://perp-api.phoenix.trade`; WS `wss://perp-api.phoenix.trade/v1/ws`.
- Drift v2 `dRiftyHA39MWEi3m9aunc5MzRF1JYuBsbn6VPcn33UH` (mainnet + devnet); SDK `@drift-labs/sdk` stable line.
- Pacifica REST `https://api.pacifica.fi/api/v1`, testnet `https://test-api.pacifica.fi/api/v1`; Ed25519-signed operations; agent keys.
- Jupiter: Tokens API v2 `https://lite-api.jup.ag/tokens/v2/search`, Swap API `https://lite-api.jup.ag/swap/v1/quote` (+ `swap` / `swap-instructions`; confirm exact paths in Jupiter docs at build time and record in FACTS).
- USDC `EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v`.
- xStocks (Token-2022, 8 decimals, extensions: metadataPointer, tokenMetadata, permanentDelegate, defaultAccountState, scaledUiAmountConfig, pausableConfig, confidentialTransferMint, transferHook): TSLAx `XsDoVfqeBukxuZHWhdvWHBhgEHjGNst4MLodqsJHzoB`, NVDAx `Xsc9qvGR1efVDFGLrVsmkzv3qi45LTBjeUKSPmx9qEh`, SPYx `XsoCS1TfEyfFhfvj8EtZ528L3CaKBDBRqRapnBbDF2W`, AAPLx `XsbEhLAtcf6HdfpFZ5xEMdqW8nfAvcsP5bdudRLJzJp`.
- Jupiter Perps `PERPHjGBqRHArX4DySjwM6UJHiR3sWAatqfdBS2qQJu` — not integrated; listed only so nobody adds it by accident.

## 4. Repository layout (monorepo, pnpm + cargo workspaces)

```
markov/
├── programs/markov/            Anchor program (Rust)
├── programs/markov-markets/    (later) native equity perps — not in this build set
├── packages/config/            typed env config, caps, allowlists, freshness windows
├── packages/facts/             FACTS.md parsed into typed constants at build time; CI fails on drift
├── packages/sdk/               TS SDK: program client, account API client, signing helpers
├── packages/adapters/          perp adapters (pacifica, drift, phoenix) + spot adapter (jupiter)
├── packages/policy/            policy engine (pure functions, no I/O)
├── packages/risk/              risk engine (pure functions)
├── packages/router/            route scoring (pure functions)
├── services/api/               control plane HTTP + WS (Fastify), Postgres
├── services/keeper/            Invest keeper + reconciliation workers
├── services/mcp/               MCP server
├── apps/terminal/              Next.js Terminal
├── apps/landing/               Next.js landing
├── brand/                      official assets (see §8)
├── docs/FACTS.md
└── infra/                      IaC, runbooks, dashboards, alert rules
```

Pure packages (`policy`, `risk`, `router`) have no network access and 100% branch coverage on decision code.

## 5. Keys and signing

- **User keys never leave the user's wallet.** Owner actions are wallet-signed transactions (Solana) or wallet-signed messages (Pacifica Ed25519). The browser stores no key material of any kind.
- **Keeper keys** (Invest keeper, reconciliation) live in a cloud KMS/HSM (GCP KMS or AWS KMS with Ed25519) behind a signing service; the service exposes `sign(txBytes)` only, with per-call policy (allowed program IDs, max lamports, rate limits). Rotation runbook in `infra/`.
- **Program upgrade authority and admin** are a Squads multisig (2-of-3 minimum). No single-key admin on mainnet.
- **Pacifica agent keys** (if used for read-heavy or owner-approved flows) are created by the owner in the Pacifica app; Markov stores only what the owner explicitly binds, encrypted at rest, revocable from the Terminal.
- RPC and API secrets: injected at deploy time from the secret manager; never in repo, never in client bundles.

## 6. Testing policy (real by default)

| Layer | What runs | Where |
| --- | --- | --- |
| Unit | pure packages against recorded real chain data (slot-labelled fixtures) | CI |
| Program | Anchor tests + LiteSVM (Markov program; Phoenix CPI with real bytes) | CI |
| Integration | Drift devnet, Subscriptions devnet, Pyth devnet feeds, Pacifica testnet | CI nightly + on demand |
| Rehearsal | funded mainnet wallets, capped sizes, every flow in `07_Mainnet_Test_Plan.md` | before every release |
| Continuous | canary keeper rule on mainnet ($5/week) that must produce a receipt every cycle | production |

A release is blocked if any rehearsal case lacks a receipt and an explorer link in the evidence log.

## 7. Configuration and caps (initial mainnet values; change only via PR + multisig where on-chain)

| Cap | Value |
| --- | --- |
| Per-account gross notional (perps) | $2,000 |
| Per-position leverage (perps, majors) | 2.0x |
| Per-account monthly Invest budget | $500 |
| Per-execution Invest size | $5–$100 |
| Global daily Invest keeper spend | $2,000 |
| Freshness windows | Pacifica 3 s, Drift 2 slots, Phoenix 3 s, Jupiter quote 10 s, official price 15 s |
| Max cost bps (Invest default) | 40 |

## 8. Brand (mandatory on every surface)

Official assets are in `brand/` (downloaded from https://markov.trade/media, kit v1, September 2026). Use only these files; never redraw the mark.

- Mark: `markov-mark.svg` (primary, on cream), `markov-mark-white.svg` (on ink), `markov-mark-mono-white.svg` / `markov-mark-mono-black.svg` (mono; use below 24px), PNGs at 1408px.
- Lockups: `markov-lockup.svg` / `markov-lockup-white.svg` (live text — install Bricolage Grotesque or use the PNGs), `markov-lockup-light.png`, `markov-lockup-dark.png`.
- Hero only: `markov-wordmark-stipple.png`.
- OG image: `markov-og-image.png` (1200×630).
- Tokens: `markov-colors.css`, `markov-colors.json`. Cream `#ECEBE6` (ground), Clay `#F6F5F0` (pressable), Ink `#1C1C1A`, Graphite `#3A3A37`, Markov Blue `#2F3BE0` (the one action color), Blue on ink `#8EA0FF`, Verified `#2E8B57`, Signal `#E8552B`. Signal colors only as status, never decoration.
- Type: Bricolage Grotesque 700–800 (display, tracking −0.03 to −0.04em), Instrument Sans 400–600 (body/UI), JetBrains Mono 400–600 (data, labels, receipts). Self-host from Google Fonts sources.
- Rules: never rotate, recolor, stretch or separate the strokes; clear space = half the mark's width; minimum 20px on screen.
- Voice: short, declarative, numeric, honest about stage. Stage labels (`TEST STAGE`, `CAPPED MAINNET`, `INTEGRATED`) are mandatory where they apply.

## 9. Claims policy (applies to UI copy, docs, receipts, MCP tool descriptions)

Every externally visible claim maps to a verified fact in FACTS or to a receipt. Prohibited anywhere: "best execution", "guaranteed", "never get liquidated", "AI-powered trading", "exchange price 24/7", "Nasdaq price 24/7", unlabelled numbers, Phoenix described as devnet/testnet, Jupiter Perps in any route list. Stage labels are exact: a component that has only been rehearsed at capped size says so.

## 10. Logging, receipts, observability

- Every decision (allow / reject / require-approval / skip) is an `ActionReceipt` persisted in Postgres **and**, for on-chain decisions, emitted by the Markov program (event + PDA). Receipts carry `request_id`, actor, mandate version, data slot / venue timestamp, each check with observed vs limit, route snapshot, tx signature, pre/post state hashes.
- Structured logs (JSON) with `request_id` propagated across API → keeper → adapter → chain.
- Metrics: decisions by outcome, adapter freshness, keeper cycle success, cap utilisation, RPC error rate. Alerts in `infra/alerts.yaml`; paging conditions in `06_Infra_Security_Ops.md`.

## 11. Definition of done (per component)

1. Builds reproducibly from a clean checkout; `pnpm -r build && cargo build --release` green.
2. All constants sourced from `packages/facts`; CI drift check green.
3. Unit + program + integration tests green; coverage gates met.
4. Rehearsal cases for the component executed on mainnet with receipts and explorer links in `evidence/<date>/`.
5. Runbook, dashboards and alerts present for anything that runs continuously.
6. Claims check passes on every UI/copy surface.
7. Security checklist in `06_Infra_Security_Ops.md` §5 signed off.
8. Stage label on every user-facing surface matches what was rehearsed.
