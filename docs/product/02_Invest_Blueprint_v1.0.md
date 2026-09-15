# Markov Invest — Complete Product Blueprint

**Programmable investing on tokenized stocks, on the Markov control plane**
Version 1.0 — 14 September 2026. Grounded in FACTS-STOCKS.md (Verification Pass #2, 11 Sept) and the Stocklana scope. Everything not verified is written as a gate. Stocklana (submission 18 Sept) is Stage 0 of this document, not the whole of it.

---

## Part I — Thesis

### 1. One sentence
**Markov Perps:** tell software how much risk it may take.
**Markov Invest:** tell software how your capital may be invested.
Same account, same mandate, same permissions, same receipts, same MCP. Different asset class, different execution adapter.

### 2. What Invest is
A rule layer over existing tokenized stocks. The user writes rules — what to buy, how much, how often, at what cost, under what conditions, with what reserve — and signs them once. Markov's keeper executes inside an on-chain spend cap the user granted, every action or skip is receipted, and any model or client can propose rules but never execute. Markov holds no shares, issues no instruments, runs no venue.

### 3. What Invest is not
Not a broker, not an issuer, not a custodian, not a venue, not a trading agent, not a source of trade ideas. Not available to US persons for as long as the underlying issuers exclude them.

### 4. Why it wins
Brokerage apps schedule buys. They cannot encode "never above $300 a month, no single name above 40%, keep $1,000 untouched, skip when it costs more than 35 bps or when Nasdaq is closed, and show me why every purchase happened or didn't". Onchain, the tokens exist, the liquidity exists, the recurring-delegation primitive exists (Solana Subscriptions & Allowances), and 24/7 execution is real. What does not exist is the rule layer with proof. That is the product, and it is the same product as Perps.

### 5. The phrase
**Your rules survive the model.**

---

## Part II — Users and problems

| User | Problem today | What Invest gives them |
| --- | --- | --- |
| Non-US retail saver with USDC | Wants disciplined exposure to US (later global) equities; brokerage access is gated by borders; onchain DCA tools have no limits or proof | Rule-bound recurring investing with cost gates, reserves and receipts |
| Crypto-native with idle stablecoins | Already onchain, distrusts bots with broad signing power | Capped delegation enforced by a Foundation program, not an app's custody |
| Operator running others' capital (fund, DAO treasury, family) | Needs enforced constraints and an audit trail, not a dashboard | Mandates as hard predicates, actor scopes, receipts for every decision |
| Agent builder | Wants a model to manage an investing rule without holding keys | MCP: propose and simulate freely; execution stays with the owner or a scoped keeper |

Primary user for Stages 0–2: the first row. The third and fourth rows drive Stages 3–4.

---

## Part III — Architecture

### 6. Shared control plane (from the perps blueprint)
Markov account → mandate (hard rules) + soft preferences → permission model (owner, keeper, MCP client, API key) → policy engine → risk engine → execution router → adapters → receipts. Invest adds one mandate type, one keeper role, one adapter family and one module in the Terminal. It does not fork the account.

### 7. Invest-specific components

| Component | Responsibility | Stage introduced |
| --- | --- | --- |
| **InvestMandate** | capital / market / actor / venue / execution / lifecycle policy classes for investing rules; versioned; owner-signed | 0 |
| **Asset registry (spot)** | pinned mint allowlist per issuer with token program, extensions, contract facts, eligibility metadata; canonical `equity.<ticker>` ids shared with perps | 0 |
| **Rule engine** | evaluates due rules, execution mode, window, conditions, budget/reserve/cost checks; produces execute / skip / retry decisions | 0 |
| **Keeper** | pulls USDC through the user's recurring delegation (Subscriptions & Allowances), builds and submits the swap, delivers tokens, emits receipts; scope `invest:execute` only | 0 |
| **Spot adapter — Jupiter** | quote, route, swap, Token-2022 ATA handling, scaled-UI-aware reconciliation, route-source tagging (`AMM` / `RFQ`) | 0 |
| **Market calendar service** | US exchange calendar and hours; later per-market calendars | 0 |
| **Official-price service** | last official close / live reference price from an on-chain feed; drives Reference Safe and deviation checks | 1 (gate 6) |
| **Corporate-actions service** | splits, dividends (scaled-UI multiplier changes), delistings, symbol changes; ensures a rebase is never booked as a fill | 1 |
| **Eligibility service** | jurisdiction gating per issuer's published lists, per-asset restrictions, disclosure text | 1 |
| **Spot adapter — direct RFQ** | xChange or issuer RFQ access if third-party access exists | 2 (gate) |
| **Spot adapter — Ondo** | mint/redeem access if programmatic and eligibility-compatible | 3 (gate) |
| **Rebalancer** | drift bands, target weights, netting of buys and sells inside a rule | 2 |
| **Instrument-aware router** | scores spot vs perp for the same canonical equity by lifecycle cost; shared with perps | 3 |
| **Cross-product risk graph** | spot holdings and perp positions in one portfolio state; hedge proposals | 4 |

### 8. Enforcement split (stated everywhere)
- *How much may be spent* — enforced on-chain by the Solana Foundation's Subscriptions & Allowances program (audited; live on mainnet and devnet; RPC-verified).
- *What may be bought, at what cost, with what reserve, when* — enforced by the Markov keeper before signing and recorded by the Markov program in the receipt.
- *Who may act* — permission model; MCP clients can never execute or withdraw.
Delegated execution exists in Invest from Stage 0 **because** the on-chain cap bounds it; this is the first delegated use case in the Markov family, and it is spot-only, small-cap, no leverage.

---

## Part IV — Asset universe roadmap

| Stage | Assets | Basis | Gate |
| --- | --- | --- | --- |
| 0 | TSLAx, NVDAx, SPYx, AAPLx | Verified mints, DEX liquidity, routable | — |
| 1 | Full verified xStocks set (60+ names/ETFs) | Issuer's official list; Jupiter `verified` + `xstocks` tag; per-mint liquidity floor | liquidity floor per asset; scam look-alike filter |
| 2 | Index baskets (S&P/Nasdaq via SPYx/QQQx-type ETFs), sector baskets built from allowlisted names | Same assets, composed | none new |
| 3 | Ondo Global Markets names | Programmatic mint/redeem + eligibility hooks | Ondo API; eligibility model; routability (none on DEX today) |
| 3 | Non-US equities via regulated issuers (EU/UK names through an EU issuer; Indian names through a GIFT City/IFSCA-regulated issuer) | Only where an issuer exists with onchain tokens on Solana | issuer existence; eligibility; oracle coverage |
| 4 | Synthetic / 1x exposure for names with no issuer, via the perp layer | Markov becomes issuer of a derivative | counsel; perps native markets (PB-4) |

Rules that never change: pinned mint addresses only; no symbol search into execution; every asset carries its issuer's powers (permanent delegate, pause) and jurisdiction list in the registry and in the UI.

---

## Part V — Rule types roadmap

| Rule | Description | Stage |
| --- | --- | --- |
| Recurring buy | single asset or fixed-weight basket; daily/weekly/monthly; execution window; retry-then-skip | 0 |
| Execution modes | Always On / Reference Safe / Market Hours Only | 0 (Reference Safe gated) |
| Manual buy / sell | one-off, owner-signed, under the same mandate | 0 (stretch) |
| Cost and liquidity gates | max cost bps incl. impact, min route liquidity, quote TTL, stale = deny | 0 |
| Reserve and ceilings | USDC reserve floor, per-period budget, monthly ceiling, single-asset cap % | 0 |
| Conditional buy | buy only if price ≤ X, or ≥ Y% below last official close, or below N-day average | 1 (needs official price) |
| Sell rules | DCA-out, take-profit, stop-value (not a stop-loss guarantee), trim above cap | 1 |
| Rebalancing | target weights with drift bands; netting buys/sells; cadence | 2 |
| Robo portfolio | template portfolios (conservative / balanced / growth) as mandates the user reviews and signs; no discretionary management | 2 |
| Dividend policy | reinvest (default via issuer rebase), or route the rebased increment per rule | 2 (needs corporate-actions service) |
| Budget-linked allowances | allowance to a model or agent to *propose* within a budget; execution still keeper-within-cap | 3 |
| Hedge rules | "protect if NVDA drops > 10%" → perp hedge proposal via the risk graph | 4 |

Every rule is a mandate version; every evaluation is a receipt; every skip has a reason code.

---

## Part VI — Execution roadmap

| Capability | Stage | Gate |
| --- | --- | --- |
| Jupiter AMM routes for allowlisted mints | 0 | rate limits; swap-instructions composition |
| Route source tagging `RFQ` vs `AMM`; prefer RFQ in market hours | 0–1 | whether Jupiter exposes xChange-sourced routes |
| Direct RFQ (xChange) access for size and tighter fills | 2 | third-party access to xChange |
| Netting across a user's basket (one swap set per run) | 2 | — |
| Netting across users in the same window (keeper batches) | 3 | MEV/atomicity design; fairness receipts |
| Instrument-aware routing (spot vs perp) | 3 | perps router live |
| Multi-issuer routing for the same equity (xStocks vs Ondo) | 3 | Ondo access |

Non-goal at every stage: hosting liquidity, running an order book, being a venue for securities-linked tokens.

---

## Part VII — Credit and yield (the third Stocklana wedge, later)

| Item | Position | Stage | Gate |
| --- | --- | --- | --- |
| Yield on idle USDC reserve | Route reserve to an allowlisted lending market under a mandate rule; reserve stays withdrawable within the rule's terms | 2 | market allowlist; withdrawal latency vs reserve semantics |
| Borrowing against tokenized stocks | Use lending markets that accept xStocks as collateral, under LTV and liquidation-buffer rules in the mandate | 3 | which markets accept which mints; issuer pause/permanent-delegate interaction with collateral |
| Dividends | Handled by issuer rebase; Invest exposes the increment in receipts and lets rules act on it | 2 | corporate-actions service |
| Structured products | Not built. If ever, composed from spot + perps under the risk graph | 4+ | counsel |

---

## Part VIII — Consumer surface

| Item | Stage | Notes |
| --- | --- | --- |
| Terminal `/invest` (rule builder, inbox, history, charts) | 0 | desktop-first |
| Mobile-first Invest (rule view, approvals, receipts; no rule creation on phone at first) | 2 | approvals are the mobile use case |
| Spending from a portfolio: sell-to-USDC on demand under a rule, then a merchant allowance via the Subscriptions program | 3 | no card issuance by Markov |
| Social / shared rules: publish a rule template (weights, limits) others can adopt as their own mandate | 3 | templates are rules, not advice; no copy-trading of positions |

---

## Part IX — Infrastructure (the fourth wedge, as internal services)

| Service | Stage | Gate |
| --- | --- | --- |
| Market calendar (US) → per-market calendars | 0 → 3 | — |
| Official-price feed (last close, live reference, market status) | 1 | on-chain equity feeds with these semantics for allowlisted names |
| Corporate-actions (splits, dividends/rebase, delistings, symbol changes) | 1 | issuer/oracle event sources |
| Eligibility and disclosures per issuer | 1 | issuer lists; counsel |
| Receipt explorer (public verifier for any Markov receipt) | 1 | — |
| Analytics: cost per purchase vs mode, skip-reason distribution, off-hours deviation stats | 1 | — |
| Route-quality telemetry (AMM vs RFQ fills, impact by hour) | 2 | route tagging |
| **Listings module** — reference-anchored Meteora DBC launch template for newly tokenized stocks (curve centred on the reference price with steep outer segments, volatility-responsive dynamic fee for off-hours dislocation, USD-equivalent graduation threshold, locked DAMM v2 liquidity) plus **graduation-gated eligibility**: a new token enters the Invest allowlist only after migration with locked liquidity ≥ floor; pre-graduation rules may allocate a capped discovery slice | 1+ (only with a partner issuer) | an issuer that launches via a curve; DBC base tokens are new mints, so this never applies to existing xStocks/Ondo tokens; counsel on primary-sale status |

---

## Part X — Markov MCP for Invest

| Stage | Capability |
| --- | --- |
| 0 | `invest.get_rules`, `invest.get_history`, `invest.simulate_rule`, `invest.propose_rule`, `invest.pause_rule`, (`invest.quote_swap`); scopes read / simulate / propose; owner signs every proposal; rejected proposals receipted and attributed |
| 1 | `invest.get_receipt`, `invest.explain_skip`, `invest.get_eligibility`; per-client rate limits and budgets on proposals |
| 2 | `invest.propose_rebalance`, `invest.propose_template`; proposal expiry; multi-proposal review |
| 3 | Scoped delegated grants: a client may *execute* one-off buys within a per-action and daily cap and an expiry, still inside the on-chain delegation; revocable; every action receipted — the "six-hour capability" |
| 4 | Cross-product proposals (hedge via perps) |

Never at any stage: `invest.withdraw`, allowlist changes, mandate changes without an owner signature.

---

## Part XI — Permission and delegation roadmap

| Stage | Who can execute | Bound by |
| --- | --- | --- |
| 0 | Owner (manual); Markov keeper for due rules | on-chain recurring delegation cap; mandate |
| 1 | Same, plus owner-approved retries | same |
| 2 | Keeper for rebalances and sells | mandate; on-chain cap for buys; sell side bound by mandate only (needs a Token-2022 delegate approval on the stock ATA — design gate) |
| 3 | Scoped model/agent grants with caps and expiry | on-chain cap + mandate + grant |
| 4 | Third-party keepers (permissionless keeper set with bonding) | same + keeper registry |

---

## Part XII — Risk engine for Invest

Checks per evaluation: budget and ceiling; reserve floor after fill; per-asset cap after fill; route cost incl. impact vs limit; route liquidity floor; quote freshness; asset paused / delegate powers exercised; market status and deviation (mode-dependent); concentration across the whole Markov account once perps share the portfolio state.
Monitoring after fill: holdings vs targets, drift, off-hours deviation stats, issuer events (pause, delisting, delegate activity), delegation balance vs upcoming schedule (`DELEGATION_INSUFFICIENT` warning before the due time).
No guarantees, by design: issuer actions, DEX dislocations and pauses can override any rule; Invest never markets "protected".

---

## Part XIII — Compliance and eligibility

- Assets carry their issuer's restrictions; the registry stores the issuer's published lists and the UI gates by declared jurisdiction. Verified today: xStocks exclude US persons and the UK; India unconfirmed.
- Markov is a software layer, not a distributor of securities; where an issuer's terms treat facilitation as distribution, Invest does not onboard users for that issuer in that jurisdiction until counsel clears it. Stage 0 uses the builder's own wallet.
- Disclosures in product, always: price exposure not share ownership; issuer permanent delegate and pause; dividends reinvested by rebase; off-hours DEX price ≠ exchange price; jurisdiction restrictions.
- No token, no yield promises, no investment advice, no discretionary management.

---

## Part XIV — Business model (stated, not decided)

Options, in order of fit: a per-execution fee in bps capped by the mandate's cost limit (the user sees Markov's fee inside the route cost, and the mandate can refuse it); a monthly subscription for rule capacity; operator tier for treasuries and funds (receipts, multi-account, reporting). No token, no spread-taking, no payment for order flow from any venue. Decide at Stage 2 with data on execution volume and rule retention.

---

## Part XV — Release stages and gates

| Stage | Window (indicative) | Ships | Gate to enter | Gate to exit |
| --- | --- | --- | --- | --- |
| **0 — Stocklana** | 12–18 Sept 2026 | Stocklana scope: four assets, recurring buy, modes, keeper within on-chain cap, receipts, MCP propose/simulate, Terminal `/invest`, real mainnet purchases from builder wallet | Stocklana gates 1–7 | submitted with end-to-end demo |
| **1 — Hardening** | 19 Sept – 12 Oct (low intensity; Fair weeks 2–4 belong to perps) | full xStocks allowlist with liquidity floors; official-price and corporate-actions services; eligibility service and disclosures; receipt explorer; conditional buys and sell rules; MCP stage-1 tools | Stage 0 shipped | test suite green on devnet mocks; 4 weeks of unattended keeper runs on builder wallet with zero unreceipted actions |
| **2 — Capped mainnet users** | after the perps security gate (Oct–Nov) | 5–15 allowlisted non-restricted users; caps per user; rebalancing and robo templates; reserve yield; mobile approvals; direct RFQ if gate closes; business-model decision | eligibility cleared for the cohort's jurisdictions; keeper key management and monitoring reviewed | retention: rules active ≥ 3 cycles for ≥ 60% of users; zero mandate violations |
| **3 — Private beta** | with perps PB-1/PB-2 | scoped model grants; borrowing against stocks; instrument-aware routing; Ondo and non-US issuers where gates close; social templates; spending allowances | Stage 2 exit; counsel on distribution per jurisdiction | audit of the invest keeper and mandate program |
| **4 — Expansion** | after PB-2 | cross-product risk graph and hedge rules; third-party keepers; synthetic exposure only with counsel | perps native markets roadmap | — |

Interplay with perps: the Markov program (account, mandate, permission, receipts) is shared and is built once in Stage 0; perps adapters take Fair weeks 2–4; Invest hardening runs in the gaps and does not pull engineers off the perps router until 12 Oct.

---

## Part XV-A — Capability stages (the product boundary)

The release stages above say *who* may use Invest and when. The capability stages below say *what the product is*. The product boundary is capability Stages 0–2; any submission (Stocklana, World's Fair) is a vertical slice through them — a working spine — not Stage 0 completed.

| Capability stage | Contents | Release stage where it becomes complete |
| --- | --- | --- |
| **0 — Trustworthy foundation** | canonical stock registry; pinned verified mints; liquidity requirements; issuer metadata and powers; eligibility and disclosures; reference price and market status; corporate-action awareness; Jupiter execution; exact Token-2022 accounting (scaled UI, hooks, delegate, pause) | 1 (hardening) |
| **1 — Programmable investing** | recurring allocations; reserve floor; monthly budget; cost/slippage limits; execution modes and windows; conditional buys; sell rules; fixed baskets; rebalance rules; pause/revoke; deterministic execute/skip/reject receipts | 2 (capped users) |
| **2 — Software-managed investing** | MCP/API proposals; scoped permissions; **approval policies** (which actions auto-execute inside the mandate, which require the owner's signature, which are always rejected); mobile notifications and approvals; portfolio-level limits across rules and products; reserve/cash management; idle-cash yield where appropriate | 3 (private beta) |

The spine, in one line: *the user sets an objective and hard limits; software proposes; Markov decides — EXECUTE, REQUIRE APPROVAL, or SKIP/REJECT — and every decision is a receipt.* The decision set gains REQUIRE_APPROVAL as a first-class outcome (owner-signed), alongside EXECUTE (keeper inside the on-chain cap) and SKIP/REJECT (receipted, no transaction).

Feature rule (Master D14): a feature ships only if it makes the account more useful. Not just DCA, not a robo-advisor clone, not "AI picks stocks".

## Part XVI — Metrics

Rule retention across cycles; executed vs skipped ratio and skip-reason distribution; average cost bps by mode and hour; off-hours deviation observed vs limit; delegation-insufficient warnings resolved before due time; MCP proposals accepted vs rejected; receipts verified by third parties via the explorer; zero actions outside mandate (the only number that must be zero).

---

## Part XVII — Non-goals (all stages unless a later decision changes them)

Issuing or custodying shares; hosting liquidity or an order book; discretionary or "AI-managed" portfolios; copy-trading positions; merchant purchase triggers and card round-ups (requires a payments-data partner; Stage 3+ at earliest); US persons; yield or protection promises; a token.

---

## Part XVIII — Open gates register

| # | Gate | Blocks |
| --- | --- | --- |
| 1 | India and demo-audience jurisdictions on the issuer's restricted list | Stage 0 public demo with real purchases |
| 2 | Subscriptions program: pull + swap + deliver atomicity; TS client instruction set | Stage 0 keeper design |
| 3 | Jupiter swap-instructions composition, Token-2022 output ATA, rate limits | Stage 0 adapter |
| 4 | Jupiter DCA product overlap — precise differentiation | Stage 0 pitch |
| 5 | Sponsor bounties on the Stocklana page | Stage 0 positioning |
| 6 | On-chain official-price feed with last-close/market-status semantics | Reference Safe; conditional buys |
| 7 | xChange/RFQ route identification in Jupiter data; direct xChange access | route tagging; Stage 2 direct RFQ |
| 8 | Lending markets accepting xStocks as collateral; pause/delegate interaction | Stage 3 borrowing |
| 9 | Ondo programmatic access and eligibility model | Stage 3 Ondo |
| 10 | Non-US issuers with Solana tokens (EU/UK names; GIFT City/IFSCA for Indian names) | Stage 3 non-US assets |
| 11 | Token-2022 delegate approval on stock ATAs for keeper-run sells | Stage 2 sell automation |
| 12 | Distribution/facilitation status under issuer terms per jurisdiction | Stage 2 user onboarding |

---

## Part XIX — Public claims register (extends the perps register)

**May say:** rules on real tokenized stocks (xStocks) bought on Solana mainnet; spend capped on-chain by Solana's Subscriptions & Allowances program; every executed and skipped action receipted by the Markov program; executes 24/7 on-chain with market-hours modes; any model can propose, only the owner signs, no model executes; no token.
**Must not say:** "invest for you", "AI-managed", "available to everyone", "shares", "dividends paid", "exchange price 24/7", "best price", "protected", any issuer or venue Markov does not have verified access to, any devnet tokenized stock, "24/7 venue" or "order book" as Markov features.

---

## Part XX — Relationship to Markov Perps

One account, one mandate system, one receipt format, one MCP. Invest is the spot side of the equity story; perps are the leveraged side; the instrument-aware router (Stage 3) chooses between them for the same canonical equity by lifecycle cost; the cross-product risk graph (Stage 4) lets a spot holding be hedged with a perp under a rule. Native equity perps (perps PB-4) close the loop for names no issuer tokenizes. None of Stages 0–2 depends on any of that.
