# Gate B — Book One hosted
Markov Book · 31 August 2026 · v0.1  
**Closes:** 27 September 2026 (day before Colosseum).  
**Rule:** if a check needs localhost, it is not Gate B.

Gate B is the apply line for any grant and the baseline Colosseum discloses. Gate C (real-venue spike, 15-min cold-start doc) is in-window. Tiny mainnet is after both.

---

## 1. What “closed” means in one sentence

A stranger on a phone opens a public URL, funds a Book One mandate on Solana devnet, watches the house agent act and get refused, revokes, and withdraws — and every step has an explorer signature.

Feeling we want: *there is a book. it is bounded. I watched it get told no. I took my coins out.*

---

## 2. Freeze list (all must be true)

| ID | Check | Proof |
|---|---|---|
| B1 | `/book` hosted HTTPS. Label: `devnet · marked PnL, not a promised rate`. No APY. No marketplace. No named live venue. | URL opened logged-out |
| B2 | Fund mandate with USDC-d. Tokens sit in mandate PDA, not operator. | tx sig |
| B3 | House agent running hosted. Interval ≥60s. Default action = skip. Three distinct keys (owner demo, operator, emergency). | process up + pubkeys in FACTS |
| B4 | ≥1 ActionReceipt with `strategy_id = BOOK_ONE` in the last hour | sig + `/v1/receipts` row |
| B5 | ≥1 RefusalReceipt `OverTxCap` with `BOOK_ONE` | sig |
| B6 | RefusalReceipt `Revoked` after UI or Telegram revoke | sig pair: revoke + next attempt |
| B7 | One more on-chain refuse from {SlippageExceeded, OverSpendCap, OverSpendDailyCap} | sig |
| B8 | `owner_withdraw` succeeds while Revoked. Owner balance up. | sig |
| B9 | Dashboard shows B4–B8 within 10s of confirm. Counts = independent chain query. `chainReady: true` with lag. | `/health` + `/v1/receipts/stats` |
| B10 | Withdraw button enabled in Active, Paused, Revoked (UI test). | screenshot + test name |
| B11 | `demo_perps` implements adapter trait: `mark, open, close, increase, reduce, positions` | trait + impl in repo |
| B12 | Paper runner: ≥7 consecutive dated files `paper/YYYY-MM-DD.md`. Ugly days kept. Started-late note allowed; invented days not. | folder |
| B13 | `SECURITY.md`: single-key upgrade authority on devnet, accepted. Emergency key cannot unpause. | file |
| B14 | 90s tape of §5, recorded against production hosts. Eight proofs committed in `docs/demo/GATE-B.md` | video + sig list |
| B15 | Zero unverified live-yield / named-venue / “all eleven refusals in public” sentences on the Book page | grep + incognito |

B1–B15 green → Gate B closed. One red → not closed.

---

## 3. What we build (only this)

### 3.1 `demo_perps` + adapter trait

Same interface a future real venue will use. Mark from a pulled price (mainnet mark used only to *mark* the mock). Position state lives in the mock. CPI only through the mandate program.

Not in Gate B: Jupiter/CLOB integration.

### 3.2 House agent `book-one`

```
sidecar  →  regime: chop | trend | halt
core     →  propose: open | increase | reduce | flatten | skip
guard    →  veto on delta, gross, per-tx, daily, slippage, oracle freshness, daily-loss 5%
program  →  last gate; receipt
```

- Guard is a pure function + unit tests. No LLM inside the guard.
- Sidecar may be a stub that returns `chop` every tick. That is enough for Gate B.
- Redteam schedule (not the happy-path loop) must force B5–B7 on chain. Happy path mostly `skip`.

### 3.3 Data

Indexer writes only from program logs.  
`GET /v1/receipts?strategy=BOOK_ONE`  
`GET /v1/book/stats` → `{net_delta, gross, funding_7d, actions, refusals, circuit}`  
`GET /health` → `{chainReady, last_slot, lag}`  
Private `ledger.json` is not a source.

### 3.4 `/book`

One page. Counters, circuit chip, receipt list with BlockReason badges, Fund / Pause / Revoke / Withdraw. Withdraw never disabled by state. Explorer links with `cluster=devnet`.

Optional: receipts feed filtered to BOOK_ONE. Not required if `/book` already lists them.

### 3.5 Bot

`/pause` and `/revoke` submit real devnet txs against the demo mandate. Enough for B6. Alerts are nice; not a Gate B blocker.

### 3.6 Paper

Same binary, `VENUE=shadow`. Daily file:

```
date
mark used
regime
actions proposed / skipped / sent
hedge error
daily-loss halt? 
notes (one line)
```

No APY field.

---

## 4. Policy template locked for Gate B

```
strategy_id        BOOK_ONE
venues             [demo_perps]
tokens             [USDC-d, SOL-d]
per_tx_cap         50
daily_cap          200
max_slippage_bps   50
expiry_days        14
max_net_delta_usd  20     off-chain guard
max_gross_usd      100    off-chain guard
daily_loss_halt    5%     off-chain guard
```

Owners may only tighten. Demo beat at 0:45 lowers `per_tx_cap` below the next clip.

---

## 5. Ninety-second tape (the close)

Record once on production hosts. Commit sigs.

| t | Beat | Proof ID |
|---|---|---|
| 0:00 | `/book`, counters, devnet label | URL |
| 0:15 | Fund. Tokens in mandate PDA | SIG-FUND |
| 0:30 | Agent hedge. ActionReceipt | SIG-ACT |
| 0:45 | Lower per-tx cap | SIG-AMEND |
| 0:55 | OverTxCap | SIG-CAP |
| 1:05 | Revoke (UI or Telegram) | SIG-REV |
| 1:15 | Next attempt Revoked | SIG-REV2 |
| 1:25 | Withdraw in Revoked | SIG-WD |

---

## 6. Calendar (31 Aug → 27 Sep)

Today is 31 Aug. Twenty-seven days. Four seats. If a seat is empty, that track waits.

| Window | Track | Exit |
|---|---|---|
| 31 Aug – 6 Sep | Paper v0 + blank `/book` route + adapter trait sketched | 3 paper days |
| 7 – 13 Sep | `demo_perps` + guard tests + one allow + one OverTxCap on chain | SIG-ACT, SIG-CAP exist |
| 14 – 20 Sep | `/book` hosted, fund/withdraw, indexer parity | B1 B2 B8 B9 B10 |
| 21 – 27 Sep | Bot revoke, redteam B7, tape, SECURITY.md, FACTS | B1–B15 green |

Slip after 27 Sep: Colosseum starts with Gate B still open. Then Colosseum *is* Gate B, disclosed as such. Do not also promise a real venue in the same week.

---

## 7. Seats

| Seat | Owns | Gate B items |
|---|---|---|
| Protocol | trait, demo_perps, program wiring, tests | B11, part B4–B8 |
| Agents | book-one, guard, paper, redteam ticks | B3 B4 B5 B7 B12 |
| Surfaces | `/book`, wallet flows, bot revoke | B1 B2 B6 B8 B10 B14 |
| Truth | FACTS, sig list, SECURITY.md, copy grep | B13 B15, all proof rows |

---

## 8. FACTS keys to fill as we close

```
BOOK_URL
PROGRAM_ID                  (already known if unchanged)
DEMO_PERPS_ID
OPERATOR_PUBKEY
EMERGENCY_PUBKEY
SIG-FUND SIG-ACT SIG-AMEND SIG-CAP SIG-REV SIG-REV2 SIG-WD
SIG-SLIP-OR-SPEND
PAPER_START_DATE
HEALTH_URL
RECEIPTS_API_URL
UPGRADE_AUTHORITY
```

---

## 9. Out of Gate B (BACKLOG)

Real venue adapter, MCP, marketplace, pooling, fees, token, sub-second quoting, LLM sidecar that changes orders, restyle of markovhq.com, mainnet, promised rate, named venue in copy.

---

## 10. Close ritual

1. Open every URL logged-out.  
2. Replay the tape once from a second wallet.  
3. Chain-query receipt count vs API.  
4. Write `docs/demo/GATE-B.md` with the eight proofs.  
5. Mark Gate B **CLOSED** + date in FACTS.  

Until step 5, say “Gate B open.”
