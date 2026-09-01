# 03 — Devnet MVP
Markov Book · 30 August 2026 · v0.1

## 1. What “MVP” means here

A stranger with a Solana devnet wallet can:

1. Open the Book One page.
2. Create a mandate from the Book One template (or have a house script do it for the demo).
3. Fund it with devnet USDC-d.
4. Watch the house agent propose actions against `demo_perps`.
5. See fills and refusals on a public dashboard within 10 seconds of confirmation.
6. Hit a cap (or flip a demo toggle) and watch a RefusalReceipt with a real BlockReason.
7. Revoke from the page or Telegram.
8. Withdraw in the Revoked state. Explorer shows the funds moved to the owner, not the operator.

If any of those eight fail on hosted URLs, it is not an MVP. Local `ledger.json` does not count.

## 2. What the user sees (the surface)

**One page: Book One.**

```
BOOK ONE · house strategy · devnet
marked PnL, not a promised rate

[ net delta  ] [ gross ] [ funding 7d ] [ refusals ]
[ live / paused / daily-loss halt ]

receipts
  14:02  hedge SOL-PERP  +12.0  allowed   sig
  14:02  increase SOL    +40.0  ⊘ OverTxCap
  13:51  flatten          0.0   allowed   sig

[ Fund mandate ] [ Pause ] [ Revoke ] [ Withdraw ]
Withdraw is always enabled.
```

No marketplace grid. No “publish a strategy.” No fake APY badge.

Optional second surface: the existing receipts feed filtered to `strategy_id = BOOK_ONE`.

## 3. Components in the MVP

| Component | Job in Book One | New work? |
|---|---|---|
| `markov-program` | Gates, receipts, mandate accounts | Small: adapter accounts for `demo_perps`; optional policy fields |
| `demo_perps` | Mock venue behind the real adapter trait | **Yes** — replace “swap/yield only” as the book’s venue |
| House agent `book-one` | research sidecar + book-core + risk-guard | **Yes** — this is the product |
| Indexer + data-api | Receipts, stats, health/`chainReady` | Wire Book One filters |
| Book UI | Dashboard + fund/revoke/withdraw | **Yes** — can live in float-web as `/book` to avoid a fourth host |
| Telegram bot | pause/revoke only | Reuse |
| Paper runner | Shadow book vs live prices | **Yes** — can be the same codebase with `VENUE=shadow` |

## 4. Book One policy template (v0)

```text
strategy_id:        BOOK_ONE
venues:             [demo_perps]
tokens:             [USDC-d, SOL-d]
per_tx_cap:         50
daily_cap:          200
max_slippage_bps:   50
spend_per_call:     small
spend_daily:        small
expiry_days:        14
max_net_delta_usd:  20     (off-chain guard in v0)
max_gross_usd:      100    (off-chain guard in v0)
daily_loss_halt:    5%     (off-chain guard)
```

Owners may only tighten.

## 5. Agent loop (devnet)

Every N seconds (start at 60s, not 400ms):

1. Read mandate state from chain + indexer.
2. Read mark prices (devnet oracle or pulled mainnet price used only to mark `demo_perps`).
3. Sidecar writes a regime label: `chop | trend | halt`.
4. Core proposes at most one action: `open | increase | reduce | flatten | skip`.
5. Guard votes. If veto → either skip silently off-chain **or** submit a proposal that the program will refuse. At least the scheduled red-team ticks must hit the chain so refusals exist.
6. If pass → submit `execute_*` through the program. Program gates run. Receipt emits.
7. Indexer lands. Dashboard polls.

**Skip is the most common action.** A book that trades every minute is a bug.

## 6. Demo script (90 seconds, hosted)

This replaces the old “three subscribers, one strategy” as the acceptance tape.

| t | Beat | Proof |
|---|---|---|
| 0:00 | Book One dashboard, live counters, “devnet / marked” label | URL |
| 0:15 | Owner funds a mandate. Explorer: tokens in mandate PDA, not operator | sig |
| 0:30 | Agent hedges. ActionReceipt on the feed | sig |
| 0:45 | Owner (or demo script) lowers per-tx cap below next clip size | sig |
| 0:55 | Next clip → `OverTxCap` RefusalReceipt | sig |
| 1:05 | Telegram `/revoke` or UI revoke | sig |
| 1:15 | Next attempt → `Revoked` | sig |
| 1:25 | Withdraw still works. Owner balance up | sig |

Record against production hosts. Commit the eight signatures.

## 7. Definition of done — gate list

Must all be true:

- [ ] `demo_perps` implements the venue adapter trait used by the future real venue
- [ ] House agent running on Railway (or equivalent), three keys distinct
- [ ] ≥1 ActionReceipt and ≥1 RefusalReceipt with `strategy_id = BOOK_ONE` in the last hour
- [ ] All eleven historical BlockReasons still exist in the program; Book One must have triggered at least OverTxCap, Revoked, and one spend or slippage reason
- [ ] Dashboard hosted, `chainReady: true`, counts match an independent chain query
- [ ] `owner_withdraw` demonstrated in Revoked
- [ ] Paper runner producing a daily log for ≥3 consecutive days (can overlap build)
- [ ] SECURITY.md lists upgrade authority and accepted devnet risks
- [ ] No APY on the page

## 8. Explicitly out of MVP

Real Jupiter fills, real USDC, performance fees, pooling, marketplace, MCP as the centrepiece, token, X-sentiment trading, sub-second quoting, “AI analyses everything.”

MCP can exist as a side tool so an external agent *inspects* Book One. It is not the MVP.

## 9. What a judge or reviewer should feel

Not “they built permissions.”  
“There is a book. It is bounded. I watched it get told no. I took my coins out.”
