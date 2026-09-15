# Markov Invest — Stocklana Scope (11–18 September 2026)

Built on FACTS-STOCKS.md (Verification Pass #2, 2026-09-11). Everything here is either verified or explicitly marked as a gate.

---

## 1. One sentence

**Markov Invest turns "buy $25 of NVDA every Friday, never more than $100 a month, keep $50 in reserve, skip if it costs more than 40 bps" into an on-chain rule the user signs once — with the spend capped by Solana's own Subscriptions program, the purchase governed by a Markov mandate, and a receipt for every action taken or skipped.**

Same protocol as the perps product: mandate, permission, receipt. Different asset class, different execution adapter. Venue agents and models can propose rules; Markov decides what the capital is allowed to do.

## 2. Why this fits Stocklana and why it is not the Fair entry

- Stocklana rules (published 12 Sept): "Tokenized stocks already trade on Solana. Build what makes owning and using them better than today's brokerage app" — five wedges (Trading, Investing, Credit and yield, Infrastructure, Consumer), **"pick one wedge and make it excellent"**. Judging is one question — could this be a real app people will actually use — scored on a real user and problem, a working end-to-end demo, a reason it belongs on Solana, and quality of execution. One submission per team; original work; open-source components allowed if disclosed. 165 registered, 14 submitted as of 12 Sept. Deadline **Friday 18 September, 4:00 pm ET = Saturday 19 September, 1:30 am IST**; judging through 2 October.
- Markov Invest is the **Investing** wedge verbatim: recurring buys and index baskets on existing tokenized stocks. It is not a new stock instrument, which the brief does not ask for.
- The World's Fair keeps its one-sentence story (perps that obey your rules). Invest is a separate submission, separate demo, separate landing section — the Fair pitch does not mention it.
- The overlap is the Markov program (account, mandate, permission, receipts), which is Fair week-1 work anyway. Stocklana pulls that forward and swaps one perp adapter for one spot adapter in week 1. Cost: Pacifica + Drift adapters compress into weeks 2–4.

## 3. Architecture (what is new vs reused)

```
User wallet
  │  signs once: (a) Markov mandate v1  (b) USDC recurring delegation to Markov keeper
  ▼
Solana Subscriptions & Allowances  ──── on-chain spend cap (≤ $25 / week, resets)   [REUSED: Foundation program]
  │  keeper pulls ≤ cap when due
  ▼
Markov program (devnet for demo pipeline; mainnet for the real purchase)          [REUSED: Fair week-1 program]
  ├─ InvestMandate: allowlisted mints, per-period budget, reserve floor, max cost bps, skip rules, pause
  ├─ Permission: keeper scope = invest:execute only; no withdraw, no perps
  └─ ActionReceipt: EXECUTED | SKIPPED_{reason} | REJECTED_{reason}, tx sig, pre/post balances
  ▼
Spot adapter (Jupiter)                                                            [NEW]
  ├─ quote → cost check against mandate → swap USDC → xStock (Token-2022 ATA)
  └─ deliver to user's ATA; reconcile with scaled-UI-aware balance read
```

**Enforcement split, stated honestly:**
- *How much can be spent* — enforced on-chain by the Foundation program (audited, not Markov's code).
- *What it may buy, at what cost, with what reserve* — enforced by the Markov keeper before signing, and checked by the Markov program when it records the receipt. In the hackathon build the keeper is Markov-run; that is delegated execution and is labelled as such.

## 4. Scope — in

| Item | Detail |
| --- | --- |
| Asset allowlist | Pinned mint addresses only: TSLAx, NVDAx, SPYx, AAPLx (verified 11 Sept). Symbol search is never an execution input; the UI shows the mint and the Jupiter `verified` flag. |
| Rule types | Recurring buy (single asset or fixed-weight basket of the four), weekly or daily cadence, with an **execution window** (e.g. Friday 09:30–16:00 ET) and retry-within-window before a skip. Three execution modes the user picks at rule creation: **Always On** (execute whenever due, any hour, if cost/risk limits pass), **Reference Safe** (outside market hours execute only if spread, liquidity and deviation from last official close are within limits — needs **gate 6**), **Market Hours Only** (execute only while the US reference market is open; calendar-based, no oracle needed). Route source is tagged `AMM` or `RFQ` when identifiable (**gate 7**). |
| Hard rules (InvestMandate) | per-period budget; monthly ceiling; USDC reserve floor; max execution cost (bps, from Jupiter quote incl. impact); allowlisted mints; pause/revoke. |
| Skip reasons (receipted) | `COST_LIMIT`, `RESERVE_FLOOR`, `BUDGET_EXHAUSTED`, `ASSET_PAUSED` (issuer pausable flag), `NO_ROUTE`, `STALE_QUOTE`, `DELEGATION_INSUFFICIENT`, `MARKET_CLOSED`, `OFF_HOURS_DEVIATION` (last two only if gate 6 closes). |
| Manual Buy / Sell now (stretch, day 6) | Owner-signed one-off swap USDC ↔ allowlisted xStock through the same Jupiter adapter, same mandate check (allowlist, cost limit, reserve floor, monthly ceiling for buys), same receipt. Not a venue, not an order book — a swap button under the rules. Sell path must handle Token-2022 input (permanent delegate, pause) and scaled-UI balances. |
| Execution | Real mainnet purchases at $5–$25 from the builder's own wallet for the demo; devnet pipeline with mock Token-2022 mints (same extensions enabled) for the automated test suite. Both clearly labelled. |
| Surfaces | Terminal `/invest`: rule builder, upcoming actions, history with receipts, pause/revoke, and (stretch) a Buy / Sell now panel with policy preview; MCP tools `invest.get_rules`, `invest.simulate_rule`, `invest.propose_rule` (owner must sign), `invest.pause_rule`, and (stretch) `invest.quote_swap` read-only. No withdraw tool, no MCP execution. |
| Charts | Contributions vs holdings over time (bar + line); cost-per-purchase in bps with the limit line; skipped-vs-executed timeline. |
| Disclosures in product | Price exposure, not share ownership; issuer permanent delegate and pause powers; 24/7 DEX price ≠ exchange price outside market hours; jurisdiction restrictions per the issuer. |

## 4A. Markov MCP for Invest

**The sentence:** any model can propose an investing rule; only the owner can sign one; no model can execute. The rule and its receipts live in the Markov account, so they survive whichever client proposed them — switch from Claude to ChatGPT to a custom agent and `invest.get_rules` returns the same mandate.

**Transport and auth:** one MCP server (Streamable HTTP) in front of the Markov API; per-client tokens with scopes `invest:read`, `invest:simulate`, `invest:propose`. There is no `invest:execute` scope for MCP clients in this entry; the keeper is the only actor with it, and it acts inside the on-chain recurring delegation.

**Tool catalog (this entry):**

| Tool | Scope | Input | Output | Side effects |
| --- | --- | --- | --- | --- |
| `invest.get_rules` | read | `{account}` | active InvestMandate, execution mode, window, next scheduled action, allowlist with mints | none |
| `invest.get_history` | read | `{account, since?, limit?}` | receipts (executed / skipped / rejected) with reason codes and tx links | none |
| `invest.simulate_rule` | simulate | candidate rule (assets + weights, period budget, monthly ceiling, reserve, max cost bps, mode, window) | policy result per check, projected monthly spend, per-asset cap check, current route cost per asset, `data_slot`, freshness | none |
| `invest.propose_rule` | propose | same as simulate | `proposal_id`, signable mandate diff for the owner, policy result; REJECT if any hard rule fails | creates a pending proposal in the Terminal inbox; no transaction |
| `invest.pause_rule` | propose | `{rule_id, reason}` | signable pause instruction | pending proposal; owner signs |
| `invest.quote_swap` *(stretch)* | simulate | `{mint, side, usdc_amount}` | route cost bps, impact, source tag (`RFQ`/`AMM` if identifiable), policy preview | none |

Absent by design: `invest.execute`, `invest.withdraw`, `invest.set_allowlist`, `invest.change_mandate` (the mandate changes only through a signed proposal).

**Approval flow:** model calls `simulate` → `propose` → proposal appears in `/invest` inbox with the diff and the checks → owner signs (mandate v+1) or declines → receipt either way, attributed to the MCP client subject. A proposal that fails a hard rule is rejected at `propose` time with the reason (`MONTHLY_CEILING_EXCEEDED`, `ASSET_NOT_ALLOWLISTED`, `SINGLE_ASSET_CAP`), and that rejection is receipted too — the demo shows this.

**Model-agnostic by construction:** the server has no model-specific logic; every response is deterministic from the account state and the data slot. Tool outputs are the same numbers the Terminal shows.

**What this is not:** not a trading agent, not a strategy source, not memory for the model. Venue agents and models decide what they want; Markov decides what the capital is allowed to do. Your rules survive the model.

## 5. Scope — out (say so in the submission)

Merchant purchase triggers and card round-ups; Ondo assets (no DEX route); automated sell/rebalance rules (manual sell is the stretch item above); leverage or hedging of any kind; building a venue or an order book of any kind (Markov routes to existing DEX liquidity, it does not host it); spot-vs-perp instrument routing for the same equity (see §12); user onboarding at scale (distribution/eligibility is the issuer's perimeter and a gate); basket weights other than fixed; anything that requires a payments-data partner.

## 6. Gates — close by Monday 14 Sept or the entry is off

1. **Eligibility:** read assets.backed.fi legal documentation; confirm India and the demo audience are not on the prohibited/restricted list. If unclear, the demo runs on devnet mocks only and says so.
2. **Subscriptions program mechanics:** read `solana-program/subscriptions` and the TS client; confirm recurring-delegation create → keeper `collect` flow for USDC; confirm whether pull + swap + deliver can be one transaction or must be two (affects atomicity and the receipt design).
3. **Jupiter composition:** confirm the swap-instructions endpoint can be embedded into a keeper-built transaction with a Token-2022 output ATA; confirm rate limits for a keeper polling four mints.
4. **Jupiter DCA overlap:** check what Jupiter's own recurring product offers so the pitch states the difference precisely (Markov: mandate rules, receipts, reserve floor, cost gate, MCP; not just a scheduler).
5. ~~Hackathon terms~~ **CLOSED 12 Sept:** rules, wedges and judging published (see §2); registered. No bounty tracks yet — check daily for sponsor bounties only.
6. **Official-price feed for Reference Safe:** confirm whether an on-chain equity price feed with market-status/last-close semantics exists on Solana for the four allowlisted names (Pyth or Chainlink equity feeds — unverified this pass). If none, Reference Safe is dropped for the entry and only Always On and Market Hours Only ship; `OFF_HOURS_DEVIATION` goes with it.
7. **xChange/RFQ route identification:** determine whether Jupiter quote/route data exposes xChange-sourced liquidity (xStocks say it is live on Solana via top aggregators during market hours). If identifiable, tag routes `RFQ` vs `AMM` in receipts and prefer RFQ inside market hours; if not, omit the tag — never infer it.

## 7. Seven-day plan

| Day | Deliverable |
| --- | --- |
| Fri 12 | Gates 1–5. Decide mainnet-real vs devnet-mock demo. Register. |
| Sat 13 | Markov program: account, InvestMandate, keeper permission, receipt events (devnet). Mock Token-2022 mints with permanentDelegate + pausable + scaledUiAmount for tests. |
| Sun 14 | Spot adapter: Jupiter quote → cost check → swap → deliver; scaled-UI-aware reconciliation. First mainnet $5 purchase from builder wallet with a receipt. |
| Mon 15 | Subscriptions integration: recurring delegation create/revoke, keeper `collect`; end-to-end: rule → pull → swap → receipt. Skip paths receipted. |
| Tue 16 | Terminal `/invest` (rule builder, upcoming, history, pause) + MCP tools. Market-hours rule if gate 6 closed. |
| Wed 17 | Charts, disclosures, devnet test suite green, demo video, README with FACTS-STOCKS excerpt and the enforcement split. **Stretch:** manual Buy / Sell now panel — only if everything above is green by noon. |
| Thu 18 | Submit by evening IST. Hard deadline is **Sat 19 Sept, 1:30 am IST** (Fri 18 Sept, 4:00 pm ET); do not plan to use the last hours. |

Perps work resumes Fri 19 with the Markov program and receipts already built; Pacifica and Drift adapters take weeks 2–4 of the Fair.

## 8. Demo script (≤ 4 minutes)

1. Connect wallet. Create Markov account. Sign InvestMandate v1: NVDAx + SPYx 50/50, $20/week, $80/month, $50 USDC reserve, max 40 bps, pause allowed.
2. Sign the USDC recurring delegation ($20/week) — show it on the Subscriptions program, not Markov's custody.
3. Keeper runs: quote both mints, cost 6 bps and 4 bps, reserve OK → two swaps → tokens land in the user's Token-2022 ATAs → two `EXECUTED` receipts with tx links.
4. Change the cost limit to 2 bps and re-run: `SKIPPED_COST_LIMIT` receipt, no transaction.
5. From an MCP client: "add TSLAx at $50/week" → `invest.propose_rule` returns a signable mandate v2 with the reason it exceeds the monthly ceiling → owner declines → receipt.
6. Pause rule. Show history chart: contributions, holdings (scaled UI), cost per purchase against the limit.
7. *(If built)* Buy now: $10 AAPLx, policy preview `Allowed`, sign, receipt. Sell now $5 SPYx → receipt. Then set the market-hours rule and show a keeper run on a weekend produce `SKIPPED_MARKET_CLOSED`.

## 9. Public claims for this entry

**May say:** real tokenized stocks (xStocks) bought on Solana mainnet from the builder's wallet; spend capped on-chain by Solana's Subscriptions & Allowances program; every executed and skipped action receipted by the Markov program; rules editable and revocable by the owner only; MCP can propose, never execute.
**Must not say:** "invest for you", "available to everyone", "shares", "dividends paid" (they are reinvested by the issuer), "best price", any Ondo integration, any devnet tokenized stock, "autonomous" without "keeper-run within an on-chain cap", "24/7 venue" or "order book" as something Markov provides (Markov routes to existing DEX liquidity), "exchange price" for an off-hours DEX quote.

## 10. Later — native perps on equities

Phoenix already lists NVDA/TSLA/SPY perps and Pacifica some; the perps router will accumulate demand data on equity markets. Native Markov equity perps remain PB-4 and carry two extra design facts: market-hours/calendar handling for oracles and funding, and a liquidity model. Invest gives the spot side of that story; nothing in this entry depends on it.


## 10A. Policy classes (structure for the mandate UI and docs)

The InvestMandate exposes the same classes the perps mandate will use, so both products read as one control plane:

| Class | Invest fields (this entry) |
| --- | --- |
| Capital | monthly ceiling, USDC reserve floor |
| Market | allowlisted mints (pinned), per-asset cap %, basket weights |
| Actor | owner: full; keeper: `invest:execute` within the rule; MCP: read / simulate / propose (owner signs); unknown: deny |
| Venue | Jupiter routes to allowlisted DEX liquidity; RFQ preferred in market hours when identifiable |
| Execution | max cost bps (incl. impact), quote TTL, stale quote = deny, min route liquidity |
| Lifecycle | execution mode (Always On / Reference Safe / Market Hours Only), execution window, retry-then-skip |

Persistence belongs to the account: the rule and its receipts survive whichever model or client proposed them. Constraints are enforced; **targets** (e.g. "keep exposure between X and Y") require an actor to act and are out of scope until delegated execution exists beyond the invest keeper.

## 11. Judging map (12 Sept rules)

| Judges look for | What the demo shows |
| --- | --- |
| A real user and problem | A non-US retail investor who wants disciplined, rule-bound buying of US stocks on-chain. Brokerage apps schedule buys but cannot encode "never above $100/month, keep $50 in reserve, skip if it costs more than 40 bps, and show me why every purchase happened or didn't". |
| A working end-to-end demo | Real xStocks purchases on Solana mainnet from the builder's wallet, pulled through an on-chain recurring delegation, delivered to the user's Token-2022 ATA, with an executed receipt and a skipped receipt. |
| A reason it belongs on Solana | Token-2022 tokenized stocks with DEX liquidity exist here; the Solana Foundation's Subscriptions & Allowances program makes capped recurring delegation native; sub-second settlement and 24/7 execution make a weekly rule cheap to run. |
| Quality of execution | Pinned mint allowlist against scam look-alikes, issuer-power disclosures, scaled-UI-aware accounting, receipted skip reasons, a model-agnostic MCP that can propose and simulate but never execute or withdraw, with rejected proposals receipted. |

**Not this entry:** synthetic or Markov-issued equity exposure (a new instrument, not "using tokenized stocks"; derivative-issuer exposure; not buildable to "excellent" in a week). It belongs to the perps roadmap and the World's Fair, which the Stocklana page itself names as the next stop.


## 12. Trading wedge — later, not this entry

The Stocklana Trading wedge ("24/7 venues, order books, stock-to-stablecoin swaps") maps onto Markov as follows:

| Item | Markov position | When |
| --- | --- | --- |
| Stock-to-stablecoin swaps | Already inside Invest via the Jupiter spot adapter; exposed as manual Buy / Sell now under the mandate (stretch). | Stocklana (stretch) |
| 24/7 venues | Exist already (Raydium, Byreal, others route xStocks today). Markov does not host liquidity. The 24/7 problem Markov solves is a rule: market-hours and off-hours-deviation skips. | Stocklana if gate 6 closes; otherwise PB-1 |
| Order books | Not built. Routing to existing books only. Hosting a book for securities-linked tokens makes Markov a venue for securities, which is out of scope by decision. | Never, unless the studio decides to become a venue |
| **Instrument-aware routing** | Canonical `equity.nvda` with instruments `spot:NVDAx (Backed)`, `spot:NVDAon (Ondo, unroutable today)`, `perp:NVDA (Phoenix)`. Router scores lifecycle cost across instruments: long holds favour spot (no funding), short or leveraged views favour the perp. Same score, same receipt, same mandate. | PB-1, after the perp router is live; add to the blueprint as the Trading wedge |
| Synthetic / 1x exposure token | A new instrument and a derivative-issuer role; not "using tokenized stocks". | Perps roadmap, PB-4 or later, with counsel |
