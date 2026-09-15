# 07 — Mainnet Test Plan (real money, real receipts)

Inherit `00_Build_Conventions.md`. This plan is the release gate. Every case is executed on Solana mainnet-beta (and the venues' mainnet surfaces) with capped sizes. No case passes without a receipt id, a transaction signature and an explorer link stored under `evidence/<YYYY-MM-DD>/<case-id>/` (JSON receipt, tx signature, screenshots, adapter responses).

## 1. Wallets and funding

| Wallet | Purpose | Funding |
| --- | --- | --- |
| `rehearsal-owner` (hardware wallet) | owner of the rehearsal Markov account; signs everything owner-side | 0.5 SOL fees; 300 USDC |
| `rehearsal-keeper` (KMS) | invest keeper signer | 0.2 SOL fees; no USDC of its own |
| `rehearsal-mcp-client` | OAuth client subject for MCP cases | — |
| Pacifica account (bound to `rehearsal-owner`) | perps rehearsal | 100 USDC deposited on Pacifica mainnet |
| Drift user account (subaccount 0, `rehearsal-owner`) | perps rehearsal | 100 USDC deposited on Drift mainnet |
| Phoenix trader account (`rehearsal-owner`) | gate P1 + rehearsal if onboarding succeeds | 50 USDC via Ember wrap |
| USDC recurring delegation (`rehearsal-owner` → keeper) | invest rehearsal | $10/week cap via Subscriptions program |

All caps in Conventions §7 apply. Total capital at risk in this plan: under $600.

## 2. Pre-flight gates (must close before §3; results recorded in FACTS)

| Gate | Method | Pass condition |
| --- | --- | --- |
| P1 Phoenix onboarding | `POST /v1/exchange/build-register-ixs` for `rehearsal-owner` with no referral, send, confirm | trader account activated on mainnet; else Phoenix stays read-only and the plan proceeds without §3.4 |
| P2 Pacifica agent-key scope | on **testnet**, create an agent key, attempt `Request Withdrawal` signed by it | outcome recorded; if withdrawal succeeds, agent keys are classified custody-equivalent and are not used on mainnet |
| I2 Subscriptions atomicity | on **devnet** then mainnet at $5: single transaction `[markov.invest_execute, subscriptions.collect, jupiter swap ixs, transfer, markov.finalize_decision]` | lands atomically; receipt reconciled; else two-transaction design is adopted and documented |
| I3 Jupiter composition | mainnet $5 USDC → NVDAx with Token-2022 ATA creation and hook accounts | tokens land; raw + scaled amounts reconcile |
| I6 official price | read Chainlink Data Streams / Pyth feed for NVDA with market status | price and status available with age < 15 s during market hours |
| Global pause drill | multisig `set_global_pause(true)` then `false` | keeper halts and resumes; receipts for both |

## 3. Cases

### 3.1 Program
- C-01 create account (owner-signed) → `MarkovAccount` PDA exists; receipt none (creation) — record tx.
- C-02 set mandate v1: max leverage 2.0x, max notional $500, min safety buffer 20%, max daily loss $50, approved SOL/BTC/ETH → `MandateSet` receipt.
- C-03 set permission for `rehearsal-mcp-client` (account:read, risk:simulate, trade:request) with $50 per action, $100 daily, 7-day expiry → receipt.
- C-04 pause → an owner trade request returns `REJECT · ACCOUNT_PAUSED` with receipt; unpause → receipt.
- C-05 replay: resend a completed `record_decision` with the same `request_id` → transaction fails; evidence = failed signature + error code.

### 3.2 Pacifica (mainnet)
- C-10 link: sign account-binding message; `/venues/accounts` shows balance 100 USDC, health, freshness.
- C-11 open SOL long $50 at 1.5x via AUTO (Pacifica selected or pinned) → `ALLOW`, route snapshot, Ed25519-signed operation, fill, receipt reconciled with Pacifica position.
- C-12 request $2,000 at 3x → `REJECT · MAX_LEVERAGE_EXCEEDED` (observed vs limit in checks); no order created; receipt.
- C-13 reduce 50% → receipt; C-14 close → receipt; positions table empty.
- C-15 stale injection: block the market-state worker for 10 s; ticket shows `STALE_MARKET_DATA`; request returns `REJECT · STALE_MARKET_DATA` with receipt.

### 3.3 Drift (mainnet)
- C-20 link: initialize user account + deposit 100 USDC (owner-signed).
- C-21 open SOL long $50 at 1.5x (Drift pinned) → receipt; on-chain user account reflects position.
- C-22 route comparison for SOL $50 over 3-day horizon shows both venues with bps components; `selected` reason text present; screenshot + JSON.
- C-23 close → receipt reconciled.

### 3.4 Phoenix (mainnet; only if P1 passed)
- C-30 read: markets, orderbook, mark, funding, Hawkeye margin view for the trader account with freshness.
- C-31 Ember wrap 50 USDC → deposit → `PhoenixDeposit` (owner-signed with `record_decision` adjacent) → receipt.
- C-32 open SOL long $25 isolated → receipt; C-33 close → receipt.
- C-34 `DelegateTrader` to a second owner-controlled key; place a $10 order with the position authority; attempt withdrawal with the position authority → must fail; both receipted (evidence for the delegation model).

### 3.5 Invest (mainnet)
- C-40 rule create: NVDAx + SPYx 50/50, $10/week, monthly ceiling $40, reserve floor $50, max cost 40 bps, Market Hours Only, window Fri 09:30–16:00 ET → two signatures (invest mandate tx; Subscriptions recurring delegation $10/week); both receipts.
- C-41 keeper run inside the window → two swaps ($5 each), tokens in the owner's Token-2022 ATAs, `EXECUTED` receipts with tx links; `spent_this_month_usd` = $10.
- C-42 keeper run outside the window (Saturday) → `SKIPPED · MARKET_CLOSED` receipt (on-chain `record_skip`), no transaction to Jupiter.
- C-43 set max cost to 2 bps; run in window → `SKIPPED · COST_LIMIT` with best route bps recorded.
- C-44 delegation insufficient: revoke the delegation, run → `SKIPPED · DELEGATION_INSUFFICIENT`; alert fired 24 h before due time in the next cycle after re-creating.
- C-45 issuer-power handling: verify the receipt's asset metadata shows permanent delegate and pausable flags; UI disclosure present.
- C-46 scaled-UI accounting: record raw amount and multiplier at fill; confirm no PnL event on multiplier change (simulate by reading multiplier on two dates).
- C-47 Buy now $5 AAPLx (owner-signed) → receipt; Sell now $5 SPYx → receipt.
- C-48 monthly ceiling: after $40 spent in the month, next run → `SKIPPED · MONTHLY_CEILING_EXCEEDED`.

### 3.6 MCP (mainnet, rehearsal account)
- C-50 `markov.get_account_state` from Claude Desktop → live state with `data_slot`.
- C-51 `markov.request_trade` SOL $25 1.2x → pending approval; owner signs in `/approvals` → receipt attributed to both.
- C-52 `markov.request_trade` SOL $500 3x → `REJECT · MAX_LEVERAGE_EXCEEDED`; receipt attributed to the client; no approval created.
- C-53 `invest.propose_rule` TSLAx $50/week → `REJECT · MONTHLY_CEILING_EXCEEDED`; receipt.
- C-54 revoke the client's permission → next call `401/403` with `ACTOR_SCOPE_DENIED` receipt.
- C-55 absent tools: client attempts `markov.withdraw` → tool not found; recorded.

### 3.7 Operations
- C-60 reconcile mismatch drill: alter a DB position row; worker flags `RECONCILE_MISMATCH`, freezes automation, corrects from chain; alert evidence.
- C-61 keeper leader failover: kill the leader mid-cycle; the standby completes or skips with receipts; no double execution (verify by `request_id`).
- C-62 RPC failover drill; C-63 key rotation on devnet; C-64 global pause on mainnet (from §2).

### 3.8 Continuous
- C-70 canary: $5/week NVDAx rule on the rehearsal account produces a receipt every cycle for 4 consecutive weeks before the capped-mainnet stage opens to users.

## 4. Evidence format

`evidence/<date>/<case>/receipt.json` (full `ActionReceipt` from the API and the decoded on-chain event), `tx.txt` (signature + explorer URL, cluster), `adapter/*.json` (raw venue responses with timestamps), `ui/*.png` (Terminal screenshots showing stage label and freshness), `notes.md` (operator, wallet, sizes, anomalies). A script `pnpm evidence:verify` re-fetches every receipt from chain and checks hashes.

## 5. Sign-off

Release 0.1 ships when every case in §3 has evidence, §2 gates are recorded in FACTS with dates, the security checklist in `06` is signed, and the canary has four consecutive weekly receipts. Any case that cannot be executed is not "deferred" — its feature is disabled in config and its stage label reflects that.
