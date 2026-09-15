# Markov

Programmable account for leveraged and spot capital on Solana. Humans, software and models express intent; the account enforces what the capital is allowed to do. Existing venues first: Pacifica, Drift, Phoenix. Native markets later, if earned.

This repository is the control-plane stack:

- `programs/markov` — account, mandate, permissions, receipts
- `packages/*` — facts, config, policy, risk, router, sdk, adapters
- `services/api` — HTTP + WS control plane (`0.0.0.0:$PORT`)
- `services/keeper` — Invest keeper (inside the Subscriptions & Allowances cap)
- `services/mcp` — model-neutral tools (read / simulate / propose; never execute)
- `apps/terminal` — reference client (`:3000`, proxies `/v1` to the API)
- `apps/landing` — markov.trade (`:3100`)

Stage today: **TEST STAGE** — program and adapters aimed at Solana **devnet**, Pacifica **testnet**, Drift **devnet**, Phoenix **mainnet reads**. Capped mainnet is a later gate, not a claim. The program id in `docs/FACTS.md` is generated and **not deployed** until `solana program show` says it is.

Rules that survive every surface: no mock market data, stale fails closed, owner signs every risk-increasing perp action, every decision leaves a receipt. Facts live in `docs/FACTS.md`.

## Quick start

```bash
pnpm install
pnpm --filter @markov/policy test
pnpm --filter @markov/adapters smoke:pacifica
MARKOV_ENV=devnet pnpm --filter @markov/api dev          # :4000
pnpm --filter @markov/terminal dev                       # :3000, /v1 → API
pnpm --filter @markov/landing dev                        # :3100
```

Program tests: `cargo test -p markov --lib`. SBF build needs the Agave/Anchor toolchain in `Anchor.toml`.

## What this is not

A trading agent, a venue, an issuer, a custodian, a broker, or a token.
