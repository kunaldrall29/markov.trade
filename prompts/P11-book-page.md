# P11 — `/book`, the desk
Seat: Surfaces · Window: 14–24 Sep · Inherits `P00-conventions.md`

## Goal
The page Gate B grades: **B1, B2, B6, B8, B9 (surface), B10**. A stranger on a phone funds, watches, revokes, and withdraws.

## Pre-flight (STOP and report if any fails)
1. `data-api` is hosted and `chainReady` is true.
2. The generated client (`packages/sdk`, Codama from the IDL) builds and can construct `fund`, `pause`, `revoke`, `owner_withdraw`.
3. A Wallet Standard wallet connects on devnet through `@solana/kit` + `@solana/kit-plugin-wallet`. **Do not** pull in the legacy `@solana/wallet-adapter-*` bundle — Kit does not use it, and it drags deprecated transitive deps.
4. Design tokens from `docs/13-FRONTEND-SPEC.md` §1 are in `app/styles/tokens.css` verbatim.

## Deliverables
Per `docs/13-FRONTEND-SPEC.md` §6:
- Header with wordmark, nav, pulsing `DEVNET` dot, theme toggle persisted at `localStorage["markov-theme"]` with a no-flash inline script.
- Label above the fold: `devnet · marked PnL, not a promised rate`.
- Stat strip (`net delta ±band`, `gross cap`, `funding 7d`, `refusals 24h`) as a `<dl>` with `gap-px bg-line` seams, mono `tabular-nums`.
- Circuit chip: `live | paused | revoked | daily_loss_halt | global_halt | stale_mark`, plus `skip = default` and mark age.
- Receipt list as `<ol>` with `tape-in`, `aria-live="polite"`, `BlockReason` shown verbatim in `refuse`, explorer link with `?cluster=devnet` on every row.
- Owner verbs: Fund / Pause / Revoke / Withdraw, each with a plain-words confirm that names what stays possible.
- Off-chain-enforced numbers carry an `off-chain guard` marker.
- Degraded banner when `chainReady` is false, naming the failing term; counters grey with a last-updated stamp.
- Polling: 5s visible, paused hidden, 30s after three errors.

## Hard constraints
- **`Withdraw` is never `disabled`.** Not while loading, not while pending, not when revoked, not when the API is down. If state is unknown it still builds a withdraw transaction.
- No APY, no projection, no marketplace surface, no venue brand.
- Zero refusals renders as `no refusals in this window`, not as a success badge.
- Nothing on this page is read from a private file; everything comes from `data-api` or the chain.

## Acceptance
```
e2e: withdraw-enabled-in-every-state      # Active, Paused, Revoked -> B10
e2e: fund-lands-in-mandate-pda            # B2, asserts the explorer account
e2e: revoke-then-next-attempt-refused     # B6, captures the signature pair
e2e: withdraw-while-revoked               # B8
e2e: receipt-visible-within-10s           # B9 surface
e2e: labels-and-copy                      # B1/B15
```
Lighthouse ≥95 performance and accessibility on a mobile profile.

## Evidence
Hosted URL opened logged-out, the five signatures, Playwright output, Lighthouse scores.
