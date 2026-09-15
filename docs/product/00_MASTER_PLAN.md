# Markov — Master Plan

**The financial control plane on Solana: capital may only do what the owner allowed.**
Version 1.0 — 14 September 2026. This is the top-level document. It does not repeat the product blueprints; it binds them, sets the order of work, and holds the decisions, gates and risks that cut across products. Read this first, then `docs/FACTS.md`, then the blueprint for the product you are working on.

---

## 1. Thesis

Models, bots and humans decide what they want to do with capital. Markov decides what the capital is allowed to do. The rules live in an on-chain account as hard predicates — not in a prompt, a session, or a model's memory — so they survive the model, the client and the venue. Every decision, allowed or refused, leaves a receipt.

Three sentences the company is built around:
- **Your rules survive the model.**
- **Venue agents suggest trades. Markov decides what the capital is allowed to do.**
- **Existing venues first; native markets later, if earned.**

What Markov is not: a trading agent, a source of trade ideas, a venue, an issuer, a custodian, a broker, a token.

## 2. Product family

| Product | One sentence | Asset class | Execution | Status (14 Sept 2026) |
| --- | --- | --- | --- | --- |
| **Markov Perps** | Tell software how much risk it may take | Perpetual futures | Multi-venue router over Pacifica, Drift, Phoenix | Test-stage build for Colosseum Crypto World's Fair, 14 Sept – 12 Oct |
| **Markov Invest** | Tell software how your capital may be invested | Tokenized stocks (xStocks first) | Rule engine + keeper inside an on-chain spend cap; Jupiter spot adapter | Stage 0 build for Stocklana, 12–18 Sept |
| **Markov Markets** | Native, physically backed, deliverable equity perps on tokenized stocks | Equity perps | Pool model (custody per token), session-aware index, delivery | Design v0.1 (15 Sept); counsel and issuer terms next; no build before 12 Oct |
| **Later** | Same account, other asset classes | lending, options, RWAs, payments | adapters | not started; only when a product above has users |

Both products share one account, one mandate system, one permission model, one receipt format, one MCP server, one Terminal. A user who has both sees one portfolio and, at Invest Stage 4 / Perps PB-4, one risk graph.

## 3. Shared architecture (what is built once)

```
ANY HUMAN · ANY BOT · ANY MODEL (via MCP) · ANY APP (via API) · TERMINAL
                                   │
                          MARKOV CONTROL PLANE
                  policy engine · risk engine · router
                                   │
                          MARKOV PROGRAM (Solana)
        account · mandate (versioned) · permissions · action receipts
                                   │
                ┌──────────────────┼──────────────────┐
           PERP ADAPTERS       SPOT ADAPTERS         KEEPERS
       Pacifica · Drift ·      Jupiter (xStocks) ·   invest keeper (inside Solana
       Phoenix (mainnet)       RFQ / Ondo (gated)    Subscriptions & Allowances cap)
```

Enforcement split, stated in every document and on every surface:
- *Who may act and what they may ask for* — Markov permissions and mandate.
- *How much a keeper may spend* — the Solana Foundation's Subscriptions & Allowances program (Invest).
- *On-chain policy gating of venue execution* — documented for Phoenix (PDA as trader authority, position authority, typed CPI); unverified for Drift (PDA delegate); impossible for Pacifica (off-chain matching).
- *Owner signature* — required for every risk-increasing perps action in the test stage and closed beta; required for every mandate change in every product.

### 3A. The account API (B2B surface, PB-1)

Any app, agent or UI keeps its own users, model and interface and asks Markov one question it should not answer itself — *is this action allowed with this user's capital?*

```
checkPolicy(action)      -> decision, checks[], reason_code, data_slot
simulateAction(action)   -> projected state, risk deltas
requestTrade(action)     -> request_id, signables[] (owner signs in the app's own wallet flow)
getAccountState()        -> mandate, permissions, positions, holdings, headroom
getReceipts(filter)      -> deterministic receipts
```

Positioning today; product at PB-1 (audited program, SDK, docs, uptime). The Terminal is the first client of this API, not a separate product.

## 4. Verified facts that shape the plan (from `docs/FACTS.md`)

- Phoenix exists only on mainnet-beta (prod, beta and Hawkeye all absent on devnet — RPC-verified). Phoenix execution cannot exist before the capped mainnet beta; in the test stage it is live data + LiteSVM proof.
- Pacifica testnet (88 markets) and Drift devnet are the executable test-stage venues. Jupiter Perps is mainnet-only, three markets, keeper-filled, no delegation — not in the launch set.
- DOGE, FARTCOIN and PUMP are on all three perp venues; equities and gold on Phoenix; Phoenix sampled fees 3.5/0.5 bps.
- xStocks (TSLAx, NVDAx, SPYx, AAPLx) are Jupiter-verified Token-2022 mints with ~$1–4M DEX liquidity and a $25 swap fills at ~0 impact; Ondo's Solana tokens exist but have no DEX route; no tokenized stock exists on devnet.
- The Subscriptions & Allowances program (`De1egAF…`) is live on mainnet and devnet and supports recurring delegations on Token-2022.
- xStocks: not for US persons, not currently in the UK; India unconfirmed. Issuer holds permanent-delegate and pause powers; dividends rebase via scaled UI amount.
- xChange (RFQ into primary-market liquidity, market hours only) is live on Solana via aggregators; route identification is unverified.

## 5. Unified roadmap

| Window | Perps | Invest | Shared |
| --- | --- | --- | --- |
| **12–18 Sept** | Fair week-1 program work pulled forward | **Stage 0 — Stocklana:** four assets, recurring rules, three execution modes, keeper inside on-chain cap, receipts, MCP propose/simulate, Terminal `/invest`, real mainnet buys from builder wallet; submit by Thu 18 evening IST (hard stop Sat 19, 1:30 am IST) | Markov program v0: account, mandate, permissions, receipts on devnet |
| **19 Sept – 12 Oct** | **Test stage:** Pacifica testnet + Drift devnet adapters, canonical registry, router with lifecycle cost, mandates, risk simulation, reduce/close, MCP/API, Terminal, Phoenix live-read + LiteSVM CPI proof; Fair submission | **Stage 1 — Hardening, low intensity:** full xStocks allowlist, official-price + corporate-actions + eligibility services, receipt explorer, conditional/sell rules | Landing page (test-stage claims), receipts explorer |
| **Oct – Nov** | Security gate → **capped closed mainnet:** Phoenix + Pacifica + Drift, 5–15 allowlisted users, ≤2x majors, owner-signed | **Stage 2 — capped mainnet users:** 5–15 non-restricted users, rebalancing, robo templates, reserve yield, mobile approvals, business-model decision | Keeper key management review; monitoring; first audit scope |
| **PB-1 / PB-2** | Private beta: 2–3+ venues, portfolio risk, long-tail tiers, rebalance, SDK, MCP; per-venue delegated reduce-only after proof | **Stage 3:** scoped model grants, borrowing against stocks, instrument-aware spot-vs-perp routing, Ondo / non-US issuers where gates close, social templates, spending allowances | Audit of program + keepers; SDK |
| **PB-3 / PB-4** | Solver/RFQ pilot; one capped native market only if routing shows unmet demand | **Stage 4:** cross-product risk graph, hedge rules, third-party keepers; synthetic exposure only with counsel | Company formation decisions; regulatory counsel per jurisdiction |

Rule for the overlap: the Markov program is built once, in Stocklana week. Fair weeks 2–4 belong to the perps router; Invest hardening fills gaps and never pulls work off the router before 12 Oct.

## 6. Decision register (locked; date; where documented)

| # | Decision | Date | Doc |
| --- | --- | --- | --- |
| D1 | Markov is a programmable control plane for perp markets, not a SOL/BTC/ETH terminal | 10 Sept | Perps §1 |
| D2 | Multi-venue architecture from day one; `PerpVenueAdapter`, never `PhoenixService` | 10 Sept | Perps §28 |
| D3 | Test-stage executable venues: Pacifica testnet + Drift devnet; Phoenix live-read + LiteSVM; Phoenix executes first at capped mainnet | 10 Sept | Perps §28, §49, K8 |
| D4 | Jupiter Perps not in the launch set | 10 Sept | Perps K5 |
| D5 | Owner-signed baseline; delegation per venue only after proof and gates | 10 Sept | Perps §29 |
| D6 | Solver/RFQ and native perps deferred; intent schema + solver interface shipped, router as solver zero | 10 Sept | Perps §58–59 |
| D7 | Public claims must map to verified facts; landing/pitch checked against Appendix L | 10 Sept | Perps App. L |
| D8 | No Invest module in the Fair submission; Invest is a separate product line | 11 Sept | Invest §3 |
| D9 | Stocklana entry = Investing wedge on xStocks; not synthetic; not Trading/Credit wedges | 12 Sept | Stocklana §2, §11 |
| D10 | Markov does not issue or custody shares, host liquidity or run an order book | 12 Sept | Invest §3, §17 |
| D11 | Recurring purchases run inside the Solana Subscriptions & Allowances cap — the first delegated use case in the family | 11 Sept | Invest §8 |
| D12 | Three execution modes (Always On / Reference Safe / Market Hours Only); off-hours is a user choice, never a default claim of exchange price | 14 Sept | Stocklana §4 |
| D13 | MCP clients can read, simulate and propose; never execute or withdraw; delegated model grants are PB-1 / Invest Stage 3 | 14 Sept | Stocklana §4A, Invest Part X |
| D14 | Feature rule: a feature ships only if it makes the Markov *account* more useful; anything that only makes the Terminal a broader app is cut | 15 Sept | Master §6 |
| D15 | The Terminal is a reference client, not the company. No bridges, no every-chain, no proprietary agent tab, no super-app surface | 15 Sept | Master §2, §3 |
| D16 | Apps like Spectrum are the app layer above Markov, not competitors; the B2B surface is the account API (`checkPolicy`, `simulateAction`, `requestTrade`, `getAccountState`, `getReceipts`) — positioning now, go-to-market at PB-1 once the program is audited and the SDK exists | 15 Sept | Master §3, §5 |
| D17 | Invest product boundary is capability Stages 0–2 (foundation → programmable investing → software-managed investing); hackathon submissions are snapshots of that product, not its boundary | 15 Sept | Invest §15A |
| D18 | Crypto perps stay a router over existing venues; the first native Markov market is equity perps, justified by the off-hours mechanism gap and the verified spot market, not by routed demand | 15 Sept | Markets design §2 |

## 7. Unified gate register (open items that block a stage)

| # | Gate | Blocks | Owner action |
| --- | --- | --- | --- |
| P1 | Phoenix onboarding for a fresh wallet without referral code | capped mainnet route set | test `build-register-ixs` on a capped mainnet trader account |
| P2 | Pacifica agent-key withdrawal scope | delegation vs custody classification | attempt `Request Withdrawal` with agent key on testnet |
| P3 | Drift PDA as subaccount delegate via CPI | on-chain enforcement beyond Phoenix | devnet prototype |
| P4 | Comparable fee inputs (Pacifica fee tiers endpoint; Drift on-chain fee params) | router fee component | read live |
| P5 | Phoenix mainnet smoke budget (low three figures USDC) | pre-beta Phoenix execution test | fund |
| I1 | India / demo-audience jurisdictions vs issuer restricted list | Stocklana public demo with real buys | read assets.backed.fi legal docs |
| I2 | Subscriptions program: pull + swap + deliver atomicity; TS client instruction set | Stocklana keeper | read `solana-program/subscriptions` |
| I3 | Jupiter swap-instructions composition; Token-2022 output ATA; rate limits | Stocklana adapter | test |
| I4 | Jupiter DCA overlap — precise differentiation | Stocklana pitch | read |
| I5 | Stocklana sponsor bounties | positioning | check daily |
| I6 | On-chain official-price feed with last-close/market-status semantics | Reference Safe; conditional buys | search Pyth/Chainlink equity feeds |
| I7 | xChange/RFQ route identification in Jupiter data; direct access | route tagging; Stage 2 RFQ | inspect route data |
| I8 | Lending markets accepting xStocks as collateral; pause/delegate interaction | Invest Stage 3 borrowing | verify |
| I9 | Ondo programmatic access and eligibility model | Invest Stage 3 | verify |
| I10 | Non-US issuers with Solana tokens (EU/UK; GIFT City/IFSCA) | Invest Stage 3 | verify |
| I11 | Token-2022 delegate approval on stock ATAs for keeper sells | Invest Stage 2 sell automation | design |
| I12 | Distribution/facilitation status under issuer terms per jurisdiction | user onboarding | counsel |

Gate discipline: a failed gate stops the dependent work and is reported; nothing is improvised around it.

## 8. Risk register

| Risk | Effect | Mitigation |
| --- | --- | --- |
| Phoenix access is invite-gated at mainnet | capped beta loses its on-chain-enforcement venue | P1 early; builder-route onboarding; Pacifica + Drift beta still viable |
| Regulatory: perps as derivatives; Invest as facilitation of securities | jurisdictional exposure to the operating entity | owner-signed baseline; no issuance; issuer eligibility gating; counsel before Stage 2 onboarding; no US persons |
| Keeper key compromise (Invest) | loss bounded by the on-chain cap per user, but reputational | HSM/KMS, per-user caps, monitoring, kill switch, audit before Stage 2 |
| Stale or wrong data | wrong decision with a valid receipt | stale-fails-closed everywhere; data slots in receipts; official-price service |
| Issuer actions (pause, permanent delegate, delisting) | rules skip or holdings move outside Markov's control | registry carries issuer powers; skip reasons; disclosures |
| Dependency on Jupiter, Subscriptions program, venue APIs | outages halt execution | fail closed, receipt the failure, no silent retries beyond the window |
| Venue-native agents and limits erode the single-venue wedge | weaker pull for one-venue users | pitch cross-venue portability, on-chain enforcement, receipts — never "we have risk limits" |
| Time: two hackathons, one builder, other commitments | thin demos | Stocklana week builds the shared program; Fair weeks 2–4 protected; Invest Stage 1 low intensity |
| Bus factor | everything on one person | docs discipline (this bundle), FACTS-first, receipts explorer, SDK by PB-1 |

## 9. Company and business model (stated, not decided)

Perps: fee inside the route cost that the mandate can refuse; operator tier for delegated capital (receipts, multi-account). Invest: per-execution bps capped by the mandate's cost limit, or subscription for rule capacity; operator tier for treasuries. No token, no spread-taking, no payment for order flow, no ads. Decide with data at capped mainnet / Invest Stage 2. Entity and jurisdiction decisions with counsel before onboarding external users.

## 10. Metrics that matter

Perps: executions across venues with receipts; rejects with reasons; router decisions where the chosen venue differed by horizon; zero mandate violations. Invest: rule retention across cycles; executed/skipped ratio and reasons; cost bps by mode and hour; zero actions outside mandate. Company: users who keep rules active 30 days; receipts verified by third parties.

## 11. Documentation set and reading order

| # | File | Purpose | Update rule |
| --- | --- | --- | --- |
| 0 | `00_MASTER_PLAN.md` | this document | on any cross-product decision |
| 1 | `docs/FACTS.md` | single source of truth: venues, program IDs, mints, cluster status, endpoints, restrictions — dated and sourced | before coding; on every verification pass; a fact absent here is not a fact |
| 2 | `01_Perps_Blueprint_v1.5.md` | Markov Perps: product, MCP, architecture, policy/risk, security, test-stage → mainnet plan, solver/native pilots, claims register (App. L), change log (App. M) | version bump + change-log entry per change |
| 3 | `02_Invest_Blueprint_v1.0.md` | Markov Invest: thesis, architecture, asset/rule/execution roadmaps, credit, consumer, infrastructure, MCP, permissions, stages, gates, claims | same |
| 4 | `03_Stocklana_Scope.md` | Invest Stage 0: 7-day scope, gates, plan, demo, MCP, claims, judging map, Trading-wedge placement | frozen at submission; then archived |
| 4a | `04_Markov_Markets_Design_v0.1.md` | native deliverable equity perps: pool model, delivery, session-aware index, risk, program sketch, stages M0–M3, gates | version bump per change; nothing scheduled before 12 Oct |
| 5 | `brand/` | official brand kit v1 from markov.trade/media | only when the kit changes |
| 6 | `prompts/00–07` | production build prompts (conventions, program, control plane, MCP, Terminal, landing, infra/security/ops, mainnet test plan) — mainnet target, real data only | when FACTS, caps or stage change |

Workflow (unchanged): verify → implement → self-validate → evidence. Three to four independent checks before implementing anything. If a pre-flight gate fails, stop and report.

## 12. Next seven days

1. Close I1–I5 on Monday 15 Sept; register and read the Stocklana page daily.
2. Build the Markov program v0 (account, mandate, permissions, receipts) on devnet — shared by both products.
3. Spot adapter, keeper, Subscriptions integration, Terminal `/invest`, MCP tools; first real $5 mainnet purchase with a receipt by Sunday.
4. Submit Stocklana Thursday evening IST.
5. Friday 19 Sept: perps week-1 items that remain (canonical registry, Pacifica testnet adapter, Drift devnet adapter, Phoenix LiteSVM harness) with the program already in place.
