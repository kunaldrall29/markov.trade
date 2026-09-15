*Markov product stack - build the control layer first, borrow liquidity initially.*

# MARKOV - Programmable Perpetuals Protocol

**Complete Product, Protocol, Multi-Venue Router, MCP, Test-Stage and Private Mainnet Beta Blueprint**

Version 1.5 - 10 September 2026 (supersedes v1.4; merges the RPC cluster register, public-claims register, venue-agent positioning and change log — see Appendices K8, L, M)


---

# Document Map

| Part | Contents |
| --- | --- |
| I | Executive decision, problem, thesis, scope and non-goals |
| II | Market, users, competition and differentiation |
| III | Product specification: Terminal, programmable account, mandates and receipts |
| IV | MCP/API/SDK specification and AI trust boundaries |
| V | Solana multi-venue technical architecture, canonical markets and program design |
| VI | Risk engine, policy engine, execution and monitoring |
| VII | Security, failure modes, regulation and operational controls |
| VIII | Current build: multi-venue test-stage release (Pacifica testnet + Drift devnet), World's Fair demo and capped closed-mainnet beta |
| IX | Private-beta completion path: solver/RFQ pilot and Markov-native perps |
| X | Business model, GTM, PMF, metrics, moat and fundraising narrative |
| Appendices | Schemas, APIs, tools, test plan, runbooks, glossary and source register |

> **Reading rule** The current product is intentionally multi-venue, but release claims must match verified venue capabilities. The free test-stage executable router is built across Pacifica testnet and Drift devnet. Phoenix remains a core adapter and the strongest documented on-chain enforcement path, validated locally with LiteSVM and against live public market data; RPC verification on 2026-09-10 confirmed that Phoenix prod, Phoenix beta and Hawkeye are present on mainnet-beta and absent on devnet, so `phDEV...` is a second mainnet deployment, not a devnet environment. The capped closed-mainnet beta targets Phoenix + Pacifica + Drift. Jupiter Perps remains a later adapter target: its program is RPC-verified on mainnet and not deployed on devnet, it supports SOL/ETH/BTC only in this pass, uses keeper-filled oracle-pool execution, and lacks an official Perps REST SDK/API. Solver/RFQ and Markov-native perps remain the two major execution layers deferred from the initial launch gate.


---

# Part I - Executive Decision and Product Thesis

## 1. Executive Decision

> **Core decision** Markov is a multi-venue programmable perpetuals protocol. The current build owns the programmable account, mandate, permission, canonical-market, portfolio-risk, router and action-receipt layers while executing against existing Solana perp liquidity. The test-stage router must execute end-to-end on at least two non-production environments: Pacifica testnet and Drift devnet. Phoenix remains a core protocol adapter because it uniquely documents PDA ownership, delegated position authority, typed CPI helpers and LiteSVM testing; Phoenix has no devnet deployment and therefore joins real-money routing only in the capped closed-mainnet beta after its access/security gates pass. Solver/RFQ and Markov-native perp markets stay outside the initial release gate and return as gated private-beta completion stages.

## 2. One-Sentence Thesis

> **Thesis** Markov lets humans, software and AI express leveraged financial intent while a programmable account enforces what the capital is allowed to do.

## 3. Positioning Variants

| Audience | Recommended positioning |
| --- | --- |
| Trader | Trade perps with rules that stay active for the entire life of the position. |
| Power user | Tell Markov the exposure you want and the risk you will accept; Markov keeps the position inside those limits. |
| Developer / agent builder | A policy-controlled execution layer for leveraged capital. |
| Protocol / wallet | Programmable margin accounts and a common risk language across perp venues. |
| Investor | Infrastructure for programmable, agent-compatible perpetual exposure rather than another liquidity venue. |

## 4. Why Perps

## 5. The User Problem

- Trading interfaces are order-centric; users often think in portfolio states and constraints, not isolated orders.
- Stop-loss and take-profit orders are narrow triggers. They do not express portfolio leverage, daily loss ceilings, allowed markets, delegated permissions, funding limits, or cross-position risk.
- Automation becomes dangerous when a bot or AI receives broad signing authority without protocol-level limits.
- Each venue has its own account, margin, risk and API semantics; a user cannot easily express one portable risk policy across venues.
- Execution quality is not only entry price. Holding cost, funding, fees, exit cost, collateral usage and venue-specific risk all matter over the life of a leveraged position.
- When a risk action happens automatically, users need an auditable explanation: which rule fired, what data was used, what action was taken, and what the resulting state became.

## 6. Product Hierarchy

*Figure 1 - Markov product and protocol hierarchy.*

| Layer | What Markov owns | Current-build status |
| --- | --- | --- |
| Terminal | Human UX for markets, routing, trade, portfolio, risk, mandates and receipts | LOCKED |
| MCP | Model-neutral control interface for compatible AI clients | LOCKED |
| API/SDK | Programmatic surface for the same Markov account/router capabilities | LOCKED |
| Programmable account | Ownership, mode, linked venues, active mandate, replay domain | LOCKED |
| Mandate | Hard account, portfolio, venue, market and actor constraints | LOCKED |
| Permission layer | Who/what may request which actions | LOCKED |
| Canonical market registry | Venue-neutral markets and dynamic venue mappings | LOCKED |
| Risk-tier policy | Per-market/risk-tier controls, including long-tail assets such as DOGE when supported | LOCKED |
| Risk engine | Current/projected portfolio risk, stress and mandate headroom | LOCKED |
| Multi-venue router | Venue eligibility, quote normalization, expected-cost scoring, AUTO/manual routing | **LOCKED - CURRENT BUILD** |
| Cross-venue portfolio layer | Aggregate positions/exposure/risk across linked venue accounts | **LOCKED - BETA BUILD** |
| Lifecycle manager | Funding/risk monitoring, reduce/close, owner-approved rebalance/migration | **LOCKED - BETA BUILD** |
| Solver/RFQ network | External market makers compete directly for Markov flow | DEFERRED FROM INITIAL LAUNCH |
| Native Markov perps | Markov-owned matching/margin/liquidation/markets | DEFERRED; PRIVATE-BETA COMPLETION PILOT |

## 7. Non-Goals for the Initial Release

- No new AMM, central-limit order book or Markov-owned liquidity venue in the test-stage/initial closed-beta launch.
- No solver/RFQ network as a launch blocker.
- No native Markov perp engine as a launch blocker.
- No protocol token.
- No retail yield vault.
- No proprietary AI trading agent; MCP/API remain the model-neutral interfaces.
- No promise to prevent every liquidation or guarantee best execution.
- No cross-chain product in the current Solana build.
- No requirement to integrate every Solana perp venue: the free test-stage release requires Pacifica testnet + Drift devnet execution, while Phoenix is validated through LiteSVM/live read surfaces and becomes mandatory for the capped closed-mainnet route set after access/security gates pass. RPC verification confirms there is no Phoenix devnet; `phDEV...` is mainnet-beta. Jupiter is not a launch blocker.
- No hard-coded BTC/ETH/SOL-only product boundary. DOGE is already live on Phoenix, Pacifica and Drift mainnet surfaces as of the 2026-09-10 verification pass; long-tail assets are supported whenever the relevant environment exposes them and Markov risk gates pass. Symbols and contract multipliers must be normalized rather than assumed identical.
- No custody of owner withdrawal keys through MCP or backend services.
- No offchain database as the source of truth for hard mandate limits.


---

# Part II - Market, Users, Competition and Differentiation

## 8. Target Users

| Persona | Primary job | Pain | Initial Markov value |
| --- | --- | --- | --- |
| Active perp trader | Maintain leveraged positions without babysitting every risk variable | Funding, leverage drift, liquidation risk, fragmented controls | Mandate + live risk + governed automation |
| Power user / quant | Express repeatable constraints around a strategy | Bots have broad permissions; venue APIs differ | Common policy and execution interface |
| AI-assisted trader | Use preferred AI client to inspect and control positions | AI tools should not hold unrestricted keys | MCP with typed tools and hard policy gates |
| Wallet / terminal builder | Offer perp execution without rebuilding safety logic | Integration and risk semantics are venue-specific | Markov SDK/API |
| Fund / desk | Delegate constrained actions to operators/software | Need auditability and permission separation | Scopes, receipts, portfolio rules |

## 9. Jobs to Be Done

- "Let me open this position only if it stays inside my risk limits."
- "Do not let a delegated tool increase risk beyond the limits I set."
- "Show me the effect of this trade before I sign it."
- "If my account becomes unsafe, reduce risk according to a pre-approved rule."
- "Let me use the same account from a terminal, script or compatible AI client."
- "Explain every automated action in a way I can verify."
- "Eventually, choose the best venue for the lifecycle of my position, not just the best displayed price."
- "Let me trade DOGE or another higher-volatility market when it exists on a connected venue, while applying stricter risk rules if I choose them."

## 10. Competitive Reality

| Competitor / category | What it already does | Why Markov must be different |
| --- | --- | --- |
| Phoenix | Fully on-chain perp exchange; FIFO orderbook + spline liquidity; 82 live markets; delegated position authority, PDA ownership path, typed CPI and LiteSVM reference tests | Markov can demonstrate its strongest protocol-enforced policy path here, but must not pretend Phoenix has a documented public devnet. |
| Pacifica | Hybrid off-chain CLOB / on-chain settlement; live testnet; 77 mainnet markets; agent-wallet signing; venue-native vault limits (per-symbol whitelist/blacklist/max leverage); in-app **AI Trading Agent** (chat with preset strategy sets, indicator selection, market analysis and trade planning; advisory, single-venue) and an intel feed | Cheapest end-to-end test venue. Markov enforcement is off-chain before signing unless venue-native controls are used; agent-key withdrawal scope must be tested. |
| Drift | Fully on-chain Anchor perp protocol; devnet + faucet; broad market set; no-withdraw account delegation | Core free devnet venue. Markov must verify PDA-delegate/CPI feasibility rather than assume it. |
| Jupiter Perps | Oracle-priced JLP pool; keeper-fulfilled PositionRequests; SOL/ETH/BTC only; RPC-verified no devnet deployment or official Perps REST SDK at this pass | Keep as a later adapter. It is not execution-equivalent to an orderbook venue and should not distort the launch router abstraction. |
| Ranger | Aggregation, unified data, smart order routing | Routing becomes a capability, not the moat. |
| VOOI | Multi-venue backend, routing and intent-based liquidity concepts | Markov must own continuous policy/state, not merely trade placement. |
| Tempo | Trading strategy automation on Solana | Markov governs what any strategy is allowed to do. |
| Intent/RFQ protocols | Intent-based derivatives / solver execution | An intent is input; Markov differentiation is persistent mandate + policy enforcement + receipts. |

## 11. Defensible Wedge

> **The wedge** Portable, policy-enforced leveraged accounts with explicit authorization boundaries and stateful action receipts.

- Venue-neutral risk language: max leverage, max notional, minimum safety buffer, daily loss, approved markets, allowed action types.
- Hard-versus-soft separation: hard mandates are non-bypassable within the Markov execution path; optimization preferences remain changeable.
- Actor-aware permissions: owner, delegate, keeper, MCP client, API key and future institutional operator may have different action scopes.
- Continuous lifecycle: the mandate persists after execution and can govern later risk-reduction actions.
- Machine-readable action receipts: a portable audit trail of request, data snapshot, rule evaluation, authorization and result.
- Cross-venue normalization over time: the same policy is translated into each venue's margin/risk semantics.

## 12. Differentiation Test

| Question | Good Markov answer |
| --- | --- |
| Why not use Phoenix directly? | Phoenix is the venue. Markov is the portable account policy and execution-control layer that can govern Phoenix and later other venues. |
| Why not Phoenix Vulcan MCP? | Vulcan exposes Phoenix. Markov MCP exposes Markov mandates, simulation, receipts and eventually multiple venues. MCP itself is not the moat. |
| Why not Pacifica's AI Trading Agent, or any venue agent? | Venue agents produce trade ideas for one venue and disclaim advice. Markov is not a source of trade ideas; venue agents, third-party models and humans are. Markov's claim is what those ideas are allowed to do with the capital — across venues, with actor-scoped permissions and a receipt showing which rule fired. On a single venue, venue-native limits cover part of this; the wedge is portability, protocol enforcement where the venue supports it, and the receipt trail. |
| Why not Ranger? | Ranger optimizes access and routing. Markov makes a persistent financial state and risk policy first-class. |
| Why not Tempo? | Tempo runs strategies. Markov constrains the actions any strategy or actor may take with capital. |
| Why not a stop loss? | A stop loss is one trigger. A mandate can cover leverage, notional, daily loss, market allowlists, delegated scopes and ongoing account health. |
| Why not an exchange? | Markov avoids the liquidity bootstrapping problem until it has earned flow and has evidence a native venue creates user value. |


---

# Part III - Product Specification

## 13. The Programmable Perp Account

```text
MarkovAccount
- owner: Pubkey
- account_id: u64
- status: ACTIVE | PAUSED | CLOSED
- execution_mode: OWNER_SIGNED | DELEGATED_REDUCE_ONLY | DELEGATED_SCOPED
- active_mandate_version: u32
- linked_venue_accounts[]
- nonce / replay_domain
- created_at_slot
- updated_at_slot
```

## 14. Position Mandate

| Rule | V0 definition | Enforcement point |
| --- | --- | --- |
| Max leverage | Post-action effective leverage must not exceed user ceiling | Before any risk-increasing action |
| Max position notional | Absolute position notional per approved market must remain <= ceiling | Before open/increase |
| Min liquidation / safety buffer | Projected risk metric must remain above user threshold | Before increase; monitor live; trigger risk-reduction workflow |
| Max daily loss | Realized + defined unrealized loss budget cannot exceed ceiling for risk-increasing actions | Before new risk; monitor |
| Approved markets | Only enumerated canonical Markov market IDs may be traded; venue symbols are resolved through adapters | Every order request |

## 15. Soft Preferences

- Preferred venue, optional minimum venue allocation, explicit venue override, or AUTO venue selection.
- Allowed risk tiers and optional per-tier leverage/notional ceilings (V1 extension; V0 may use explicit market limits).
- Prefer lower funding versus better execution.
- Expected holding period.
- Target exposure or leverage rather than hard maximum.
- Acceptable estimated execution cost.
- Notification preferences.
- Whether risk-reduction actions should be proposed, one-click approved, or executed under delegated authority.

> **Rule** Soft preferences may guide optimization. They must never override a hard mandate.

## 16. Permission Model

| Actor | Typical scopes | Default V0 |
| --- | --- | --- |
| Owner wallet | read, simulate, trade, change mandate, grant/revoke delegation, withdraw via venue | Full |
| Terminal session | read + owner-signed actions | Allowed |
| MCP read token | portfolio, markets, risk, simulate, receipts | Allowed |
| MCP trade scope | request trade / reduce / cancel within mandate | Optional; approval-first |
| Keeper | reduce-only, cancel risk-increasing orders | Optional after demo |
| API key / bot | scoped by market, notional, action and expiry | NEXT |
| Third-party agent | same as delegated API actor; never automatically privileged | LATER |

## 17. Action Receipts

```text
ActionReceipt
- account_id
- request_id
- actor_pubkey / actor_subject
- action_type
- market
- requested_params_hash
- market_data_slot / timestamp
- mandate_version
- checks[]: {rule, observed, limit, result}
- simulation_summary
- decision: ALLOW | REJECT | REQUIRE_APPROVAL
- reason_code
- tx_signature (optional)
- pre_state_hash
- post_state_hash
- created_at
```

| Receipt reason code | Example |
| --- | --- |
| MAX_LEVERAGE_EXCEEDED | Requested post-trade leverage 5.2x; mandate allows 3.0x |
| MARKET_NOT_ALLOWED | Requested canonical market is not on the account allowlist |
| DAILY_LOSS_BUDGET_EXCEEDED | New risk blocked after account passes daily drawdown budget |
| MIN_SAFETY_BUFFER | Projected account health below configured floor |
| STALE_MARKET_DATA | Quote/risk state older than freshness window |
| SLIPPAGE_LIMIT | Simulated impact above user ceiling |
| ACTOR_SCOPE_DENIED | MCP token may read/simulate but not execute |
| EXECUTION_CONFIRMED | Transaction landed and post-state reconciled |

## 18. Position Lifecycle

*Figure 2 - Markov position lifecycle.*

1. User or software creates a position request.
2. Markov fetches fresh authoritative venue/market state.
3. Policy engine evaluates hard rules against current and projected state.
4. Risk engine simulates the post-trade account.
5. Authorization layer decides whether owner signature or delegated capability is sufficient.
6. Execution adapter builds and submits the transaction.
7. Reconciler confirms actual venue state.
8. Markov emits the action receipt.
9. Monitoring continues; if a hard threshold is crossed, Markov proposes or executes an allowed risk-reduction action depending on account mode.

## 19. Terminal Product

| Screen | Required V0 content |
| --- | --- |
| Overview | Equity, total notional, effective leverage, daily PnL, safety status, active mandate summary |
| Markets | SOL/BTC/ETH market price, funding, liquidity/quote preview, allowed/not-allowed badge |
| Trade ticket | side, size/notional, leverage target, order type, slippage; live mandate pass/fail preview |
| Positions | size, entry, mark, PnL, leverage contribution, risk contribution, reduce/close actions |
| Mandate | five hard rules, version, owner signature flow, pause controls |
| Risk | current venue account health, stress scenarios, distance to each mandate threshold |
| Receipts | chronological allow/reject/execute events with tx links and rule explanation |
| MCP / Integrations | connection endpoint, OAuth state, scopes, revoke button, tool catalog |

### 19A. Market Explorer and Venue Selection

Markov is not a BTC/ETH/SOL-only terminal. Those three markets are the V0 proving set. The production market universe should be discovered from connected venues and normalized into Markov canonical markets. DOGE is an explicit example of the long-tail/high-volatility assets users should be able to trade when at least one connected venue supports the market and Markov's market-risk gate passes.

| Terminal concept | Behavior |
| --- | --- |
| Market categories | Majors, alts, high-volatility/meme, RWA and All Markets; categories are UX labels, not legal classifications. |
| Venue availability | Show which adapters currently support the canonical market and which capabilities are available. |
| Risk badge | Show Markov risk tier plus the main reasons: volatility, depth, oracle quality, market age and venue concentration. |
| AUTO venue | Markov chooses among eligible venue adapters using executable cost/risk data. |
| Manual venue | Advanced user can pin a venue if the mandate and venue capability checks pass. |
| Unsupported market | Markov must say unavailable; it must never fabricate or synthesize a market silently. |

**Design principle:** one user-facing `DOGE-PERP` market can map to one or more venue-specific instrument IDs. The user may choose AUTO or a venue explicitly. Markov's canonical identity remains stable even when venue listings change.

## 20. Key UX Principle

> **UX principle** Never force the user to understand Markov internals before trading. Show the outcome first - "Allowed", "Would exceed 3x", "Safety buffer falls below 20%" - and make the detailed proof expandable.


---

# Part IV - MCP, API and AI Interface

## 21. Why MCP Instead of a Markov AI Agent

*Figure 3 - AI/MCP trust boundary: the model requests; Markov validates and authorizes.*

## 22. MCP Design Principles

- The LLM is not a source of market truth.
- The LLM is not a liquidation calculator.
- The LLM is not allowed to bypass the mandate.
- Read and simulation tools should be broader than execution tools.
- Write tools must have narrow JSON schemas and explicit units.
- Every write must carry an idempotency/request identifier.
- All calculations returned to the model should include timestamps/slots and data freshness.
- Owner-only actions such as modifying a hard mandate or withdrawing collateral should remain outside MCP V0 or require explicit wallet-level approval.
- Tool names should describe economic actions rather than venue implementation details whenever possible.

## 23. Current-Build MCP Tool Catalog

| Tool | Purpose | Risk class |
| --- | --- | --- |
| markov.get_account | Read account, linked venues, mandate version and permissions | Read |
| markov.get_portfolio | Read aggregate cross-venue positions/exposure/PnL | Read |
| markov.get_positions | Read normalized venue positions | Read |
| markov.get_markets | Read canonical markets, risk tiers and venue availability | Read |
| markov.get_risk | Read current portfolio/venue risk and mandate headroom | Read |
| markov.compare_routes | Compare eligible venue quotes, fees, funding and route score | Read/compute |
| markov.simulate_trade | Return projected post-trade state for a chosen/AUTO route | Read/compute |
| markov.check_action | Evaluate action against hard policy | Read/compute |
| markov.quote_trade | Build route plan + executable quote + freshness metadata | Read/compute |
| markov.request_trade | Create an owner-signable or scoped executable trade request | Sensitive |
| markov.reduce_position | Reduce-only request within permission ceilings | Sensitive |
| markov.rebalance_position | Request owner-approved movement/rebalance between linked venues | Sensitive |
| markov.cancel_orders | Cancel specified or risk-increasing orders | Sensitive |
| markov.get_receipts | Read route/policy/execution/reconciliation history | Read |

Initial releases intentionally omit unrestricted `markov.modify_mandate` and `markov.withdraw`. Hard-policy editing and collateral withdrawal remain first-party owner-authorized actions.

## 24. Example MCP Conversation

```text
User: What happens if SOL falls 10%?
AI -> markov.get_positions()
AI -> markov.get_risk()
AI -> markov.simulate_market_shock({asset:"SOL", move_bps:-1000})

Markov returns:
- equity_after
- leverage_after
- venue_account_health_after
- mandate_checks[]
- data_slot / timestamp

AI explains the result. It does not calculate the risk itself.
```

```text
User: Reduce my SOL risk so the safety buffer is at least 25%.
AI -> markov.plan_risk_reduction({market:"SOL", target_buffer_bps:2500})
Markov -> {recommended_reduce_notional, projected_state, checks}
AI -> markov.request_trade(...)
Markov -> REQUIRE_OWNER_SIGNATURE (or ALLOW if a valid reduce-only delegation exists)
```

## 25. MCP Authentication and Authorization

| Scope | Examples | Default |
| --- | --- | --- |
| account:read | account, positions, receipts | Grant |
| market:read | market data, funding, metadata | Grant |
| risk:simulate | trade/shock simulations | Grant |
| trade:request | create owner-approval transaction request | Optional |
| trade:reduce | reduce-only delegated action | Off by default |
| orders:cancel | cancel orders | Off by default |
| mandate:write | modify hard constraints | Never in V0 MCP |
| collateral:withdraw | withdraw venue collateral | Never in V0 MCP |

## 26. MCP Security Threats

| Threat | Control |
| --- | --- |
| Prompt injection asks model to perform risky trade | Hard mandate + scoped OAuth token + explicit action schema |
| Compromised AI client token | Short-lived tokens, revocation, per-tool scopes, ceiling/market restrictions |
| Confused deputy | Bind actor subject + account + scopes + request id in receipt and authorization decision |
| Hallucinated market values | All financial data must come from Markov tool results; include freshness metadata |
| Replay of old write request | Nonce/idempotency key + expiry + slot freshness checks |
| Tool schema ambiguity | Integer/decimal units explicit; use bps and atomic units; reject unknown fields |
| Model attempts mandate change | Tool not exposed; owner-only terminal flow |
| Model attempts withdrawal | Tool not exposed in V0 |

## 27. REST/SDK Surface

```text
client.account.get()
client.portfolio.get()
client.positions.list()
client.markets.list()
client.venues.listCapabilities()
client.routes.compare(params)
client.routes.quote(params)
client.risk.get()
client.risk.simulateTrade(params)
client.policy.check(action)
client.trades.request(params)
client.positions.reduce(params)
client.positions.rebalance(params)
client.orders.cancel(params)
client.receipts.list()
```


---

# Part V - Solana and Multi-Venue Technical Architecture

## 28. Evidence-Grounded Multi-Venue Architecture

### 28A. Generic Perp Venue Adapter

```text
PerpVenueAdapter
- venue_id
- capabilities()
- list_markets()
- resolve_market(canonical_market_id)
- get_market_state()
- get_funding_state()
- get_account_state()
- quote(action)
- simulate(action)
- submit(action)
- get_execution_status(request_or_tx_id)
- build_reduce_or_close()
- build_cancel()
- reconcile(request_or_tx_id)
```

The interface is capability-aware and execution-model-aware. A venue adapter must report whether execution is an on-chain orderbook, hybrid signed API, AMM/JIT path, or keeper-filled request; whether orders fill synchronously or asynchronously; whether it supports market/limit/conditional orders, cross or isolated margin, delegated authority, builder codes, transaction construction, CPI/on-chain composition, authoritative risk views and free test environments. Markov must never pretend feature parity across venues.

**Verified launch-set decision (2026-09-10):**

| Venue | Test environment | Launch role | Enforcement posture |
| --- | --- | --- | --- |
| Pacifica | Live testnet API; 88 markets observed | **Executable dev/test router venue** | Off-chain Markov preflight + signed agent/main-wallet operation; agent-key withdrawal scope is a release gate |
| Drift | Devnet + USDC faucet | **Executable dev/test router venue** | Owner/delegate path; delegate cannot withdraw; PDA/CPI path remains to be proven |
| Phoenix | **Mainnet-beta only (RPC-verified); prod, beta and Hawkeye absent on devnet.** LiteSVM is the deterministic free test path. | **Core adapter + local protocol-enforcement proof; mandatory closed-mainnet route target** | Strongest documented Markov path: position authority, PDA owner registration and typed CPI |
| Jupiter Perps | **Mainnet only (RPC-verified no devnet deployment)** | **Later adapter, not launch set** | Owner-signed PositionRequest/keeper model; no delegation documented |

This means the World's Fair/test-stage product is still genuinely multi-venue: Pacifica + Drift execute end-to-end without real funds, while the Phoenix adapter and on-chain policy path are exercised deterministically in LiteSVM and against live public read APIs. Phoenix is not a devnet venue: RPC verification confirms the `phDEV...` program is a second mainnet-beta deployment.

### 28B. Canonical Market Registry

```text
CanonicalMarket
- market_id: "crypto.doge.perp.usd"
- base_asset_id: "crypto.doge"
- display_symbol: "DOGE-PERP"
- instrument_type: PERPETUAL
- category: MAJOR | ALT | HIGH_VOL | RWA | OTHER
- quote_unit: USD
- risk_tier: 1..5
- status: ACTIVE | WATCH | DISABLED
- venue_mappings[]:
    - venue_id
    - venue_market_id
    - contract_multiplier
    - settlement_asset
    - execution_model
    - capabilities
    - discovered_at
- risk_inputs
- updated_at
```

A canonical market becomes executable only when at least one adapter reports a supported instrument and Markov has sufficiently fresh market/risk data. The mapping **must carry a contract multiplier**, not just a symbol: e.g. BONK may appear as `kBONK` or `1MBONK`, while gold can appear as `GOLD`, `XAU` or `PAXG` and may represent different underlyings/contracts. Markov must never infer economic equivalence from a ticker string alone.

DOGE is not hypothetical whitespace: the 2026-09-10 verification found it live on Phoenix, Pacifica and Drift mainnet surfaces. Phoenix reports DOGE with a 10x venue maximum; Pacifica reports DOGE with 20x; Drift includes DOGE in its market constants, but live leverage/status must be read from on-chain `PerpMarket` accounts at runtime. Markov's own risk ceiling may be lower than the venue maximum.

### 28C. Risk-Tiered Asset Universe

Markov should support high-volatility and long-tail markets, but it should not pretend that all perps have BTC-like risk. Risk tiers are Markov control metadata, not moral labels and not permanent classifications. They should eventually be computed from measurable inputs such as realized/implied volatility, executable depth, spread, market age, oracle robustness, open interest, funding instability, venue concentration and liquidation behavior.

| Tier | Illustrative profile | Default policy posture |
| --- | --- | --- |
| 1 | Deep, mature majors | Highest eligible leverage/notional ceilings |
| 2 | Liquid large-cap alts | Moderate ceilings |
| 3 | Volatile / long-tail but established, e.g. DOGE depending on current metrics | Lower ceilings; tighter monitoring |
| 4 | Very high-volatility or thin meme/alt markets | Low leverage, smaller notional, isolated treatment preferred |
| 5 | New, illiquid, weak-oracle or unstable markets | Disabled by default; explicit opt-in only if enabled at all |

Example future mandate extension:

```text
max_leverage_by_risk_tier:
  1: 5.0x
  2: 3.0x
  3: 2.0x
  4: 1.5x
  5: disabled

allowed_markets:
  crypto.doge.perp.usd: true
```

Test environments may begin with conservative static Markov tiers while market-data pipelines mature. Mainnet beta uses measured inputs, explicit staleness gates and per-venue ceilings. The user's allowable leverage is `min(user mandate, Markov market ceiling, venue ceiling)`.

## 29. Recommended Current-Build Execution and Enforcement Modes

| Venue / mode | How it works | Security / product implication |
| --- | --- | --- |
| Phoenix owner-signed | Markov checks policy and composes Phoenix action; user signs | Baseline real-money mode; withdrawals remain owner-controlled |
| Phoenix delegated position authority | `DelegateTrader` sets a position authority that may sign orders while withdrawal authority remains with owner | Strong automation path; documented |
| Phoenix Markov-PDA ownership/CPI | Phoenix registration can accept off-curve authorities; typed CPI contexts exist for delegated market orders, capabilities, registration, cancel, transfer and risk views | Best candidate for protocol-level enforcement; prove in LiteSVM before mainnet |
| Pacifica signed API | Markov gates action before deterministic JSON is signed by main wallet or agent key | Enforcement is off-chain by construction; test whether agent key can request withdrawal before storing one |
| Drift delegated subaccount | Markov/authorized delegate can trade/deposit/swap/cancel but documented delegate cannot withdraw | Good delegation primitive; PDA/CPI path is still unverified |
| Jupiter owner-signed PositionRequest | User signs a request; keeper later executes/rejects | Async semantics and no documented delegation make it a later adapter |

> **Release choice** Risk-increasing actions default to owner approval/signature unless a venue-specific restricted delegate path is proven. Reduce-only automation may be enabled per adapter after capability tests. No common `execute()` abstraction may silently promise atomicity or synchronous fills where the venue does not provide them.

## 30. Onchain Program Model

*Figure 4 - Proposed Markov program objects feeding a capability-aware multi-venue router and venue adapters.*

## 31. Proposed Solana Accounts

| Account | Seeds / identity | Contents |
| --- | --- | --- |
| MarkovAccount | ["account", owner, account_id] | owner, status, mode, active mandate version, nonce |
| Mandate | ["mandate", markov_account, version] | hard limits, approved markets, activation slot, hash |
| Permission | ["permission", markov_account, actor] | scopes, ceilings, allowed markets, expiry, revocation nonce |
| ReceiptIndex (optional) | ["receipt-index", markov_account] | counter / Merkle or event checkpoint if receipts are mostly events |
| VenueLink | ["venue", markov_account, venue_id] | venue account identifier, adapter config, status |

## 32. Proposed Instructions

| Instruction | Signer | Purpose |
| --- | --- | --- |
| create_account | owner | Create Markov account and default status |
| set_mandate | owner | Create and activate immutable mandate version |
| pause_account | owner | Block risk-increasing actions |
| grant_permission | owner | Create scoped actor permission |
| revoke_permission | owner | Invalidate actor permission |
| link_venue | owner | Bind an approved venue account / adapter configuration |
| record_route_plan | owner or authorized actor | Bind selected venue/quote/plan hash to mandate version and freshness window |
| check_and_execute_venue | owner or valid delegated actor | Validate selected venue action and invoke/compose the adapter-supported execution path |
| reduce_risk | owner or reduce-only delegate | Perform allowed risk-reduction action on an eligible linked venue |
| record_rebalance | owner | Record owner-approved cross-venue rebalance/migration steps and resulting state |
| emit_receipt / event | program | Record decision, route, reason, transaction and reconciliation metadata |

## 33. Transaction Validation Order

*Figure 5 - Required validation pipeline for sensitive actions.*

1. Validate Markov account and active mandate ownership.
2. Validate actor identity and permission expiry/scopes.
3. Validate venue identity, adapter capability set and linked trader/subaccount/account identifier.
4. Fetch or invoke authoritative risk views when required.
5. Check market-data/slot freshness.
6. Compute projected post-action state.
7. Apply hard mandate predicates.
8. Enforce action-specific ceilings and reduce-only semantics.
9. Submit through the selected venue-specific execution path (transaction/CPI, signed API operation, or asynchronous request) or return the owner-signable payload.
10. Reconcile actual venue state/fill status and emit receipt/event only after the adapter reaches a defined terminal or accepted state.

## 34. Venue State, Delegation and Risk Data

### Phoenix
- Mainnet public data: REST `https://perp-api.phoenix.trade`, WS `wss://perp-api.phoenix.trade/v1/ws`; public market data needs no key, trader routes use JWT.
- Official SDK surface: `@ellipsis-labs/rise` (verified 0.4.67), Rust `phoenix-rise` / `phoenix-rise-ix`, with typed on-chain CPI contexts.
- Account model: cross-margin parent (`subaccount_index=0`) plus isolated child accounts; USDC wraps to Phoenix canonical collateral via Ember.
- Delegation: `position_authority` can sign position actions; owner remains withdrawal authority. Registration docs explicitly permit off-curve authorities such as PDAs.
- Risk/read surface: Hawkeye exposes margin/liquidation/BBO/funding views.
- Test reality **[LIVE/DOCS]**: RPC `getAccountInfo` on 2026-09-10 confirmed Phoenix prod, Phoenix beta and Hawkeye are executable on mainnet-beta and absent on devnet. The `phDEV...` program is a second mainnet deployment, not devnet. LiteSVM reference fixtures are the deterministic free test path.

### Pacifica
- Mainnet API `https://api.pacifica.fi/api/v1`; testnet `https://test-api.pacifica.fi/api/v1` was live with 88 markets on 2026-09-10.
- Execution is an off-chain CLOB with on-chain settlement/custody; POST operations use Ed25519-signed deterministic JSON.
- Agent-wallet keys are bound by the main wallet and can sign trading operations. Whether they can request withdrawal is **UNVERIFIED and a custody/security gate**.
- Unified margin combines USDC, spot collateral and perp PnL; the venue also has per-symbol vault whitelist/blacklist/max-leverage controls that overlap with part of Markov's mandate surface.

### Drift
- Fully on-chain Anchor program with DLOB/JIT/AMM liquidity; **[LIVE/DOCS]** RPC verification on 2026-09-10 confirmed the Drift program is executable on both mainnet and devnet, and devnet has a USDC faucet.
- Each subaccount may have one delegate. Official docs state the delegate may deposit, swap and place/cancel orders but **cannot withdraw**.
- Markov must read live status/leverage/fees from on-chain `PerpMarket` accounts. SDK constant lists are discovery aids, not live truth.
- PDA delegate and CPI-friendliness are **UNVERIFIED** and must be prototyped before claiming on-chain Markov enforcement over Drift.

### Jupiter Perps
- Oracle-priced JLP counterparty model, not a CLOB. A trader creates a `PositionRequest`; a keeper later executes or rejects it.
- Perps supports SOL, ETH and BTC in the verified pass. **[LIVE]** RPC verification on 2026-09-10 confirmed the Jupiter Perps program is executable on mainnet; the same address is non-executable/system-owned on devnet, so Jupiter Perps is not deployed there.
- Official Perps API page is work-in-progress; no official Perps REST API appears in the main Jupiter API index, and docs point to community IDL parsing. Jupiter CLI supports perp operations.
- No delegation is documented. The adapter therefore needs asynchronous request/status semantics and remains outside the launch set.

## 35. Offchain Services

| Service | Responsibility | Source of truth? |
| --- | --- | --- |
| API Gateway | Auth, rate limit, REST/SDK entry | No |
| MCP Gateway | MCP transport, OAuth, tool schemas | No |
| Market Data Service | Normalized snapshots from active adapters: Phoenix REST/WS, Pacifica REST/WS, Drift on-chain subscriptions/gateway; Jupiter when later enabled | No - cached view |
| Risk Service | Deterministic calculations, stress simulations | No - reproducible |
| Policy Service | Preflight mirror of onchain mandate checks | No - onchain mandate authoritative for hard limits |
| Execution Planner | Build quote and transaction plan across eligible venue adapters | No |
| Reconciler | Confirm Solana tx + venue state; produce final receipt | No |
| Notifier | Push trigger / risk alerts | No |
| Postgres | Profiles, OAuth grants, indexed receipts, telemetry | No for hard policy |


---

## 36. Suggested Technology Stack

| Component | Recommendation |
| --- | --- |
| Solana program | Rust; Anchor if it speeds founder iteration, otherwise minimal native/Pinocchio-compatible program depending on CPI needs |
| Venue integrations | Phoenix: Rise TS/Rust + CPI/LiteSVM; Pacifica: signed REST/WS (+ official Python reference); Drift: TS SDK/on-chain subscriptions/devnet; Jupiter later: IDL/request-state adapter |
| Frontend | Next.js / React / TypeScript; Solana wallet adapter or embedded wallet layer as needed |
| Backend | TypeScript for orchestration; Rust service only where latency/math warrants it |
| MCP server | TypeScript MCP SDK v2 targeting 2026-07-28 spec |
| Database | PostgreSQL; Supabase is acceptable for product metadata/auth support, not risk truth |
| Streaming | Phoenix WS, Pacifica WS, Drift account subscriptions/Solana RPC; venue-specific snapshot reconciliation after gaps |
| Queue | Simple durable job queue for receipts/notifications/router reconciliation; avoid unnecessary infrastructure |
| Observability | Structured logs + metrics + tracing + alerting; correlation by request_id/account_id/tx_signature |
| Testing | Deterministic unit/program tests + Phoenix LiteSVM; Pacifica testnet integration; Drift devnet + faucet integration; gated mainnet smoke tests with tiny caps |

## 37. Normalized Data Model

```text
CanonicalMarketRef
- canonical_market_id
- base_asset_id
- display_symbol
- category
- risk_tier
- venue_market_mappings[]
- availability_status
- updated_at

NormalizedPosition
- venue_id
- canonical_market_id
- venue_market_id
- side
- base_size
- notional_usd
- entry_price
- mark_price
- unrealized_pnl
- realized_pnl_today
- funding_accrued
- margin_mode
- venue_account_id
- last_updated_slot

NormalizedRiskState
- equity
- gross_notional
- net_delta_usd
- effective_leverage
- venue_account_health
- normalized_safety_buffer
- daily_loss
- mandate_headroom[]
- data_slot
- computed_at
```


---

# Part VI - Policy, Risk, Execution and Monitoring

## 38. Policy Engine

```text
allow(action, current_state, projected_state, mandate, permission) =
  actor_scope_ok
  AND market_allowed
  AND projected_leverage <= max_leverage
  AND projected_abs_notional <= market_notional_limit
  AND projected_safety_buffer >= min_safety_buffer
  AND projected_daily_loss <= max_daily_loss
  AND action_amount <= actor_ceiling
  AND data_is_fresh
  AND request_not_replayed
```

## 39. Risk Engine

| Calculation | V0 requirement |
| --- | --- |
| Equity / effective collateral | Use Phoenix account/risk semantics rather than wallet balance alone |
| Gross notional | Sum absolute perp notionals |
| Effective leverage | Defined consistently from venue-effective collateral; document exact formula |
| Daily loss | Choose and document realized + unrealized treatment and reset boundary; do not silently change |
| Safety buffer | Map to authoritative venue margin/headroom metric; show raw Phoenix metric alongside normalized display |
| Post-trade projection | Include fees, estimated slippage, order side, open orders and updated margin |
| Stress simulation | Deterministic price shocks; label as scenario, not forecast |
| Funding | Read current/estimated venue funding; do not treat current rate as guaranteed holding cost |

### 39A. Market and Venue Risk Gates

Before Markov offers a long-tail market in AUTO mode, the risk engine should evaluate both the instrument and the venue. A market being listed somewhere is necessary but not sufficient. Eligibility inputs should include executable depth for the requested size, spread, oracle/source freshness, volatility, funding instability, market age, open interest, venue operational status and concentration of available execution in a single venue.

A user may opt into riskier tiers, but an MCP client, bot or delegated actor cannot silently expand the account's allowed risk universe. Adding a new market or raising the tier/leverage ceiling remains an owner-authorized policy change.

## 40. Hard Rules vs Risk Actions

| Breach / condition | Default V0 response |
| --- | --- |
| Requested action exceeds max leverage | Reject |
| Requested action uses non-approved market | Reject |
| Requested action would breach min safety buffer | Reject |
| Daily loss budget already exhausted | Reject risk-increasing action; allow reduce/close |
| Live safety buffer crosses threshold | Notify + prepare recommended reduction; execute only if user/delegation mode allows |
| Market data stale | Pause sensitive execution; refetch |
| Venue/API unavailable | Fail closed for risk-increasing action; keep monitoring/retry policy explicit |
| Account paused by owner | Only allow safe read and owner-defined reduce/close paths |

## 41. Liquidation-Risk Automation

> **No guarantee** Markov must never market "never get liquidated." Gaps, oracle failures, venue halts, RPC failures, network congestion and disappearing liquidity can bypass any monitoring threshold.

```text
if current_buffer < mandate.min_buffer:
    plan = solve_minimum_reduce_notional(target_buffer = mandate.recovery_buffer)
    if plan.executable and permission.allows_reduce_only:
        execute(plan)
    else:
        notify_owner(plan)
```

## 42. Multi-Venue Execution Optimization - Current Build

```text
ExpectedLifecycleCost(v) =
    EntrySlippage(v)
  + TradingFees(v)
  + ExpectedFunding(v, holding_horizon)
  + ExpectedExitCost(v)
  + CollateralOpportunityCost(v)
  + VenueRiskPenalty(v)
  + Migration/OperationalCost(v)
```

The current router must: (1) resolve the canonical market into venue instruments; (2) discard stale/degraded/mandate-ineligible venues; (3) request executable quotes; (4) normalize fee, slippage, funding and margin semantics; (5) compute a route score for the stated/estimated holding horizon; (6) show the user the ranking; (7) honor AUTO or explicit venue selection; (8) revalidate immediately before signature/execution; and (9) fail closed or re-quote when the chosen route expires.

The test-stage router must compare and execute across at least Pacifica testnet and Drift devnet. Split routing and cross-venue migration belong in the private-beta build only after single-route reconciliation is reliable. A migration is not assumed atomic: Markov should prefer pre-funded linked accounts and open-replacement-before-close-old sequencing when the risk model permits it.

## 43. Monitoring and Alerting

| Signal | Alert / action |
| --- | --- |
| Risk data stream gap | Mark stale; block risk-increasing execution until refreshed |
| Safety buffer within warning band | Notify; precompute reduction |
| Safety buffer crosses hard threshold | Trigger approved risk workflow |
| Daily loss > 80% budget | Warn; display remaining headroom |
| Daily loss >= 100% budget | Block new risk |
| Unexpected position delta after tx | Reconcile immediately; pause automation if mismatch persists |
| Permission near expiry | Notify owner/delegate |
| Repeated RPC/venue errors | Fail closed and surface degraded mode |
| Mandate version changed | Invalidate stale quotes/requests built against old version |


---

# Part VII - Security, Failure Modes and Regulation

## 44. Security Model

## 45. Threat Model

| Threat | Impact | Mitigation |
| --- | --- | --- |
| Owner wallet compromise | Attacker can change mandate / move assets | Outside full Markov prevention; hardware wallet support, clear signatures, optional delays for permission changes later |
| Delegated signer compromise | Unauthorized trading within signer privileges | Reduce-only/scoped permissions, ceilings, expiry, rapid revocation, no withdrawals |
| MCP OAuth token theft | Unauthorized tool calls | Short TTL, narrow scopes, actor binding, revocation, owner signature for high-risk actions |
| Prompt injection | AI requests malicious action | Deterministic policy gates; no withdrawal/mandate tools; confirmation |
| Stale oracle/market data | Bad risk decision | Slot/timestamp freshness rules; authoritative venue views; fail closed |
| TOCTOU between quote and execute | Projected state differs before landing | last_valid_slot/expiry, slippage bounds, recheck in transaction where possible |
| RPC failure | Unable to submit/confirm | Multiple RPC paths, explicit unknown state, reconciliation before retry |
| Duplicate/replayed request | Repeated trade | Idempotency key, nonce, expiry, tx reconciliation |
| Venue exploit / insolvency | Collateral loss or abnormal state | Venue allowlist, risk monitoring, no false guarantee; multi-venue diversification later |
| Keeper outage | Risk action not executed | Redundant workers; owner alerts; native venue conditionals where useful |
| Markov program bug | Policy bypass or denial of service | Small code surface, invariant tests, audit before mainnet delegation |
| Backend database compromise | Tampered UI/history | Hard mandate onchain, verify receipts/txs, no DB authority over policy |

## 46. Critical Invariants

- No actor may modify an active hard mandate without owner authorization.
- A permission may never exceed the action scope encoded by its owner-approved grant.
- Risk-increasing actions must fail if required risk data is stale or cannot be computed.
- Reduce-only permissions may never increase absolute directional exposure under the defined semantics.
- An action built against mandate version N must not execute under mandate version N+1 without revalidation.
- No MCP token can imply withdrawal authority in V0.
- Every executed Markov-governed action must be reconcilable to a Solana transaction and resulting venue state.
- Rejection must be the default when program IDs, venue accounts, market IDs, actor identity or freshness checks do not match expectations.

## 47. Audit and Mainnet Gate

| Gate | Required before meaningful mainnet delegation |
| --- | --- |
| Program tests | Instruction and invariant coverage; adversarial cases |
| Integration tests | Phoenix fills, cancels, stop/conditional behavior, margin edge cases, stream gaps |
| Property/fuzz tests | Limit arithmetic, decimal conversions, reduce-only, nonce/replay |
| Threat review | MCP/authorization, signer custody, RPC failure, venue mismatch |
| External audit | Mandatory before promoting autonomous delegated execution with meaningful funds |
| Bug bounty / disclosure | Public process after stable release |
| Operational runbooks | Key rotation, pause, RPC failover, incident communication |

## 48. Regulatory and Access Considerations

- V0 should be positioned as software/protocol infrastructure and use the venue's supported access model.
- Do not market to restricted jurisdictions where the execution venue itself is unavailable.
- Do not promise permissionless global availability as a legal conclusion.
- Obtain specialist counsel before mainnet launch, especially before introducing autonomous delegated execution, fees on routed derivatives volume, custody-like signer arrangements, or native Markov markets.
- When Markov eventually operates its own perp venue, regulatory analysis becomes materially more complex and must be a separate launch workstream.


---

# Part VIII - Test-Stage and Private Mainnet Beta Build Plan

## 49. Release Objective

> **Current-build success definition** A user can connect one Markov account, discover a canonical perp market, compare the same exposure across at least two executable non-production venues, select AUTO or pin a venue, pass/fail a persistent mandate, execute through Pacifica testnet and Drift devnet, inspect the aggregate portfolio, control the same account through Terminal/API/MCP, and receive route + policy + execution receipts. Phoenix is concurrently integrated and exercised through LiteSVM plus live read APIs; it becomes a mandatory route in the capped closed-mainnet beta after its access/security gates pass. RPC verification confirms Phoenix has no devnet staging environment.

The router is a current product feature. Solver/RFQ and Markov-native perps are the only major execution layers intentionally outside the initial launch gate. Jupiter Perps is not one of those two strategic deferrals; it is simply a lower-priority venue adapter whose verified capabilities do not justify launch-set complexity.

## 50. Current Build Scope

| Area | COMMITTED deliverable |
| --- | --- |
| Markets | Canonical dynamic market registry. SOL/BTC/ETH are mandatory proving markets; DOGE and other long-tail/high-volatility markets are executable whenever a connected venue exposes them in the target environment and Markov risk gates pass. |
| Venues | **Test-stage execution: Pacifica testnet + Drift devnet.** Phoenix: core adapter, live reads + LiteSVM policy/CPI proof, then mandatory closed-mainnet integration. Jupiter Perps: later capability-aware adapter, not launch set. |
| Router | Venue eligibility, quote normalization, fee/slippage/funding comparison, expected-lifecycle-cost score, AUTO/manual route, quote freshness, re-quote/fallback and reconciliation. |
| Programmable account | One logical Markov account linking multiple venue accounts and one active hard mandate. |
| Mandates | Core leverage/notional/safety/daily-loss/market rules plus venue ceilings, per-market/risk-tier limits and actor scopes as the beta matures. |
| Portfolio | Aggregate cross-venue notional, delta, PnL, leverage, daily loss, venue concentration and mandate headroom. |
| Trade lifecycle | Open/increase/reduce/close/cancel on supported adapters; owner-approved cross-venue rebalance/migration in beta. |
| Risk | Current + projected risk, stress simulation, market/venue risk gates, stale-data fail-closed behavior. |
| Receipts | Quote snapshot, eligible venues, selected route, mandate evaluation, authorization, tx signature(s), reconciliation and resulting state. |
| Terminal | Market explorer, route comparison, trade ticket, portfolio, positions, mandates, risk, receipts and MCP connection surface. |
| MCP | Read portfolio/markets/risk; compare routes; simulate; request owner-signable trades; reduce/cancel; request rebalance; read receipts. No raw-key custody or unrestricted withdrawal. |
| API/SDK | Same normalized account/market/router primitives used by Terminal and MCP. |
| Mainnet target | Early-feedback, conditional, allowlisted, capped closed-mainnet beta with Phoenix + production-ready Pacifica/Drift routes, followed by a broader private beta. |
| Deferred from initial launch | Solver/RFQ network and Markov-native perp engine/markets. |

## 51. World's Fair Dev/Test Sprint

| Week | Engineering target | Validation target |
| --- | --- | --- |
| Sep 14-20 | Programmable account + mandate; canonical registry; Pacifica testnet adapter; Drift devnet adapter; Phoenix live-read + LiteSVM harness; Terminal shell | Prove SOL quote/state normalization across Pacifica + Drift; run Phoenix PDA/delegation/CPI tests locally; complete the five venue gates |
| Sep 21-27 | Multi-venue AUTO/manual router; owner-approved trade path on Pacifica/Drift; receipts/reconciliation; market-risk tiers; Phoenix instruction builder/CPI prototype | 10+ trader interviews; measure whether route panel/mandates change decisions; test failure/requote paths |
| Sep 28-Oct 4 | Cross-venue portfolio view; MCP/API route tools; risk simulation; reduce/close flows; DOGE path if present in both target test environments; mainnet read-only venue comparison | Test with 5+ users and 2+ MCP clients; measure route choice, manual override and repeat use |
| Oct 5-12 | Hardening; stale-data fail-closed logic; optional third read-only adapter; dev/test release; docs/video/pitch; capped-mainnet checklist | 20+ end-to-end rehearsals; freeze Phoenix onboarding, Pacifica agent-key and Drift delegation blockers |

**Test-stage launch gate:** Pacifica testnet + Drift devnet must both execute end-to-end. Phoenix must pass deterministic LiteSVM policy/CPI tests and live read/market discovery, but we do not describe it as devnet execution because RPC verification confirms no Phoenix devnet exists. A fake multi-venue UI that silently executes everything on one venue is prohibited.

## 52. Multi-Venue Router Requirements

```text
Trade request
   -> canonical market resolution
   -> venue capability + health filter
   -> mandate / permission eligibility
   -> executable quotes from eligible venues
   -> normalize fees + slippage + funding + margin/risk
   -> expected lifecycle cost + venue-risk score
   -> AUTO ranking or manual venue override
   -> immediate pre-signature revalidation
   -> execute selected route
   -> reconcile actual fill + resulting risk
   -> emit receipt
```

Router invariants:

- Never route to a venue/market/capability not returned by the active adapter registry.
- Never use a stale quote after its adapter-specific validity window.
- A manual venue override may change optimization preference but may not bypass hard mandate or venue-risk gates.
- If AUTO route A fails before execution, Markov must re-quote rather than silently use stale route B.
- Route scoring must expose the main cost components; no opaque "AI best venue" label.
- Funding is a stochastic holding cost, not a guaranteed static fee. Markov must show the assumed holding horizon and freshness.
- Split routing is optional during devnet and should be enabled only after single-venue execution/reconciliation is boringly reliable.
- Cross-venue migration is multi-step unless a future venue pair exposes an atomic path. The UI must communicate intermediate exposure and collateral risk.

## 53. Dev/Test Demo Script

1. Connect wallet and create one Markov programmable account + mandate.
2. Discover `SOL-PERP` and show normalized Pacifica-testnet and Drift-devnet market state, including venue-specific capabilities.
3. Request a $10,000 SOL long with a hard leverage/position/safety mandate.
4. Markov evaluates both routes, shows expected cost/constraints and selects AUTO; user can pin the other venue.
5. Execute on one venue and produce a route + mandate + execution receipt; reconcile actual venue state.
6. Repeat on the second venue to prove the router is not cosmetic; aggregate both positions in the Markov portfolio.
7. Simulate a risk breach and perform an allowed reduce/close path while showing which hard rule fired.
8. Show Phoenix live market discovery and the local LiteSVM on-chain policy/CPI proof. Explain that Phoenix enters real-money routing at capped closed-mainnet after access/security gates pass; RPC verification confirms there is no Phoenix devnet.
9. Show `DOGE-PERP` as a canonical market using live verified mainnet metadata: DOGE exists on Phoenix/Pacifica/Drift. Only execute DOGE in the demo if it is actually present and safe in the chosen test environments.
10. From an MCP-compatible client, read the same account, simulate an action, request a valid trade, then request a mandate-violating trade and show deterministic rejection.

## 54. Conditional Capped Closed-Mainnet Beta

The first mainnet release is deliberately private, allowlisted and capped. It is an early-feedback environment, not a public launch. Mainnet access opens only after the devnet router and every enabled venue adapter pass the security/reconciliation gate.

**Pre-mainnet blockers (venue verification pass #1):**

1. **Phoenix access:** onboard a fresh wallet through the builder/no-referral registration path; secure referral/builder arrangement if still gated.
2. **Pacifica agent-key scope:** test `Request Withdrawal` with an agent key. Do not store/use Markov-controlled agent keys until withdrawal capability is conclusively excluded or safely constrained.
3. **Drift PDA delegate/CPI:** set a PDA as delegate on devnet and attempt a trade via the intended program path; otherwise classify Drift enforcement as off-chain/delegate-signer only.
4. **Comparable fees:** resolve Pacifica current fee tiers and Drift live fee params before expected-cost routing is allowed to claim fee optimization.
5. **Phoenix beta-cluster question - CLOSED 2026-09-10:** RPC verification confirmed `phDEV...` is on mainnet-beta and absent on devnet. There is no Phoenix devnet; use LiteSVM for free protocol integration tests.
6. **Phoenix mainnet smoke budget:** fund one capped mainnet trader account (low three figures USDC) for pre-beta smoke tests of the real onboarding, Ember deposit, order, cancel and receipt path. This is the only way to exercise Phoenix execution before the closed beta; LiteSVM proves logic, not access.

- No unresolved critical/high issue in Markov program/router signing or permission paths.
- Deterministic adapter health checks, quote freshness and fail-closed behavior.
- Reconciliation tests for partial fill, failed tx, RPC gap, venue outage and stale risk data.
- Owner emergency pause and global beta pause procedure.
- No MCP withdrawal capability; risk-increasing actions remain owner-signed initially.
- Independent review/audit proportionate to the final custody/signing architecture before meaningful caps.
- Legal/access review for the frontend/beta cohort and each connected venue.

**Provisional initial caps (operational proposal, not immutable protocol constants):** start with approximately 5-15 allowlisted users, low four-figure collateral per account, low five-figure global collateral, 2x or lower leverage on mature markets, tighter leverage on higher-risk tiers, and a small global notional ceiling. Exact figures must be frozen only after final venue/account semantics and security review. Tier-4/5 markets should begin disabled; DOGE may be enabled only if the market/risk data and venue support pass the launch gate.

**Closed-beta behavior:**

- Multi-venue AUTO/manual routing is live with real funds.
- Initial production target is Phoenix + Pacifica + Drift, but only adapters that pass their specific auth/delegation/reconciliation gates are enabled. At least two must be production-ready before any multi-venue mainnet claim; adapters are added one at a time.
- Risk-increasing trades require owner approval/signature.
- Delegated automation is reduce-only and narrowly capped until proven.
- Every route/execution must reconcile to venue state or the account enters degraded/paused automation mode.
- Caps increase only after a defined incident-free observation window and successful chaos/incident drills.

## 55. Private Mainnet Beta Launch and Completion

The broader private beta is the actual product launch target after closed-beta feedback. It should include the complete external-venue Markov product: multi-venue routing, canonical markets, long-tail/risk-tier support, cross-venue portfolio risk, owner-approved lifecycle actions, Terminal, API/SDK, MCP and auditable receipts.

| Private-beta stage | Included | Launch condition |
| --- | --- | --- |
| PB-1 - Launch | 2-3+ reliable venues, AUTO/manual routing, portfolio/mandates, broader market registry, MCP/API, owner-approved lifecycle actions | Closed beta demonstrates stable reconciliation, no unresolved Sev-1/2 security issue, caps and incident runbook approved |
| PB-2 - Expansion | More venues/assets, funding-aware routing, portfolio limits, optional split routing, collateral planner, scoped reduce/rebalance permissions | Sufficient live telemetry and adapter reliability |
| PB-3 - Solver/RFQ pilot | Opt-in maker quotes compete with external-venue routes for selected users/markets | Markov originates enough flow and credible makers commit to quote |
| PB-4 - Native Markov perp pilot | One carefully selected Markov-native market, isolated/capped, private makers, separate audit and risk controls | Solver/route data proves unmet demand and team is ready to own margin/liquidation/oracle risk |

**Interpretation:** PB-3 and PB-4 are how the private-beta program can conclude, but they are not allowed to block the test-stage build, capped closed beta, or PB-1 launch. This preserves the ambitious end-state without turning the current build into an exchange-liquidity bootstrapping project.

---

# Part IX - Private-Beta Completion: Solver Network and Native Markov Perps

*Figure 6 - The release path now ships multi-venue routing first; solver/RFQ and native markets are gated beta-completion layers.*

## 56. What Is No Longer Roadmap-Only

The following capabilities moved into the committed devnet/private-beta product and must no longer be described as future V1/V2 ideas: multi-venue executable routing, canonical market discovery, DOGE/long-tail support when venue/risk-approved, funding-aware expected-cost routing, cross-venue portfolio state, portable mandates, lifecycle reduce/close/rebalance workflows, and Terminal/API/MCP access to the same account.

## 57. Release Stages

| Stage | Purpose | Required execution model |
| --- | --- | --- |
| Dev/test / World's Fair | Prove real product and multi-venue architecture | Pacifica testnet + Drift devnet execute; Phoenix live-read + LiteSVM protocol proof |
| Conditional capped closed mainnet | Early real-money feedback under strict limits | External venues only; owner-sign risk increases; narrow reduce-only delegation |
| Private beta launch | Real multi-venue product for an allowlisted cohort | External venues + full Markov router/portfolio/lifecycle layer |
| Private beta completion | Test whether Markov should internalize execution | Add solver/RFQ pilot, then one native market only if gates are met |

## 58. Solver / RFQ Network - Deferred from Launch, Included as Beta-Completion Pilot

```text
User / MCP / API intent
        |
        v
Markov policy + portfolio risk
        |
        v
Current venue router ----------------------+
        |                                   |
 Phoenix / Jupiter / Pacifica / Drift      |
                                            |
                                  Solver / RFQ quotes
                                  MM A / MM B / MM C
                                            |
                       <---- same normalized route score ---->
                                            |
                                      best valid plan
```

The solver network must compete with, not replace, existing venues on day one. A solver quote enters the same eligibility, mandate, freshness, price-impact and settlement checks as a venue route. Maker reputation, quote commitment, failure penalties and settlement guarantees must be designed before material mainnet flow.

**Solver launch gates:** repeated Markov-originated flow, multiple credible makers, measurable improvement versus venue routing, deterministic settlement/reconciliation, abuse controls, and a separate security review.

## 59. Native Markov Perps - Deferred from Launch, Included as Final Private-Beta Pilot

> **Native-market rule** Markov earns the right to own a perp market only after the router proves demand that external venues/solvers cannot satisfy well enough.

The first native pilot should be deliberately narrow: one market, USDC collateral, isolated margin, capped open interest, capped account notional, whitelisted/known market makers during the private pilot, robust oracle/mark methodology, explicit liquidation/backstop design, emergency controls and a separate audit. It should not begin with dozens of listings or portfolio margin.

Native-market evidence should include meaningful routed notional, repeated unmet-demand telemetry, maker commitments, a reason external venues are insufficient, audit/capital readiness and a jurisdiction/access plan.

## 60. Native Perp Design Space

| Subsystem | Initial private-pilot bias | Later options |
| --- | --- | --- |
| Matching | Solver/RFQ-informed or simple transparent book with known makers | Fully open CLOB, hybrid, auction/batch |
| Collateral | USDC, isolated | Cross margin / portfolio margin |
| Markets | One high-evidence market | Additional majors, long-tail, RWA if earned |
| Funding | Simple transparent premium/index model | Adaptive/bounded variants after data |
| Liquidation | Explicit partial liquidation + backstop/insurance design | More advanced portfolio liquidation |
| Oracle/mark | Robust external oracle plus internal executable-price sanity checks | Multi-source/impact models |
| Risk controls | Hard OI/account caps, circuit breakers, private cohort | Gradual permissionless expansion |
| Agent/MCP access | Same Markov mandate/permission layer; no special bypass | Broader scoped automation |

## 61. Final Operating Model

```text
HUMANS / SOFTWARE / AI CLIENTS
          |
 Terminal | API/SDK | MCP
          |
          v
 MARKOV PROGRAMMABLE ACCOUNT
          |
 Mandates + Permissions + Portfolio Risk + Receipts
          |
          v
 MULTI-VENUE ROUTER  (ships now)
          |
 Phoenix / Jupiter / Pacifica / Drift / Others
          |
          +----> Solver / RFQ network  (private-beta completion)
          |
          +----> Markov-native perp market(s)  (final gated pilot)
```

The product users adopt first is the router + programmable account + Terminal/MCP. Solver and native markets are vertical-integration options that should be earned by actual flow, not assumptions.

---

# Part X - Company, Business Model, GTM and PMF

## 62. Business Model

| Revenue stream | When | Who pays | Notes |
| --- | --- | --- | --- |
| Builder / routed execution fee | Devnet/private beta | Trader through venue builder program | Only where venue supports user-approved builder economics; transparent fee cap |
| Pro subscription | Private beta | Active traders | Advanced monitoring, receipts, simulations, automation preferences |
| API / SDK usage | Private beta | Wallets, terminals, bots | Tiered by requests or active accounts |
| Managed mandate infra | Private beta+ | Protocols / desks | Policy and account-control service |
| Execution optimization fee | Private beta+ | Trader / integrator | Could be bps or share of measurable execution savings; validate |
| Institutional licensing | V2+ | Funds / fintechs | Permissions, audit trail, deployment/support |
| Native venue economics | Native-pilot+ | Traders / makers | Only if/when Markov operates markets |

## 63. Go-To-Market Wedge

| Stage | Primary channel | Message |
| --- | --- | --- |
| Hackathon | Solana perp traders / builders | Your perp account should enforce rules, not just display them. |
| Closed/private beta | Power users, bots, AI-assisted traders | Use Markov from terminal or MCP without giving software unrestricted authority. |
| Developer GTM | Wallets, terminals, agent frameworks | Add policy-controlled perp execution via SDK/API. |
| Institutional | Small funds / desks | Delegated trading with enforceable limits and audit receipts. |
| Network stage | Market makers / venues | Compete for Markov-originated flow. |

## 64. PMF Hypotheses

| Hypothesis | How to test | Kill / pivot signal |
| --- | --- | --- |
| Traders value persistent risk mandates | Observe repeat mandate creation and willingness to leave them active | Users only use simple TP/SL and ignore policy screen |
| Users will use AI/MCP for trading workflows | Instrument tool calls and repeat sessions | MCP is a novelty demo; users return exclusively to terminal |
| Policy makes delegated execution acceptable | Offer narrow reduce-only delegation | Users refuse any delegation even with hard limits |
| Developers want a reusable risk layer | Pitch SDK to wallets/bots/terminals | Every integrator prefers direct venue APIs and sees no value in policy abstraction |
| Cross-venue policy has value | Add second venue and compare | Users are venue-loyal and never want portability |

## 65. Moat Development

| Potential moat | Strength | How it compounds |
| --- | --- | --- |
| User interface | Weak | Easy to copy |
| MCP server | Weak alone | Phoenix already exposes MCP; must be tied to Markov policy |
| Venue adapters | Moderate | Integration depth and normalized semantics accumulate |
| Policy account standard | Strong if adopted | Wallets/apps can target one risk language |
| Historical execution/risk data | Strong over time | Better routing, expected-cost models, incident/risk benchmarking |
| Authorization + receipt standard | Strong if integrated | Becomes trusted boundary for software/AI capital access |
| Originated flow | Very strong | Allows solver network and eventual native liquidity |
| Market-maker network | Very strong later | Improves execution and makes new markets easier to launch |

## 66. Kill Conditions

- Traders do not care about persistent mandates beyond ordinary stop-loss/take-profit controls.
- MCP usage is demo-only and does not produce repeat behavior.
- Users will not delegate even reduce-only actions, and owner-signed workflows are not valuable enough on their own.
- Venues rapidly ship equivalent portable policy primitives and make a neutral layer unnecessary.
- Markov cannot demonstrate measurable value: prevented forbidden action, improved risk response, saved cost, or simplified integration.
- Regulatory constraints make the intended distribution or business model impractical in the initial target markets.
- Technical dependency on venue APIs/CPI is too unstable to provide trustworthy lifecycle management.

## 67. Core Metrics

| Metric family | Metrics |
| --- | --- |
| Acquisition | qualified trader signups, linked accounts, MCP connections |
| Activation | mandate created, first valid trade, first receipt viewed |
| Retention | weekly active accounts, repeated mandate usage, repeated MCP/terminal sessions |
| Volume | Markov-governed notional, routed notional, active open interest under mandate |
| Safety | rejected unsafe actions, successful risk reductions, failed/unknown automation events, stale-data blocks |
| Economics | builder revenue, revenue per active account, execution cost vs venue-direct baseline |
| Developer | SDK keys/apps, active integrations, tool-call volume |
| Quality | quote-to-execution divergence, receipt reconciliation success, latency, RPC/venue failure rate |

## 68. Fundraising Narrative

## 69. Brand and Messaging

| Do say | Avoid saying |
| --- | --- |
| Programmable perps | AI-powered trading bot |
| Account-level risk rules | Never get liquidated |
| Use from terminal, API or compatible MCP clients | Works identically with every AI product |
| Policy-controlled execution | Autonomous money manager |
| Existing venues first; native markets later if earned | We are launching a new DEX immediately |
| Humans, software and AI can request actions; Markov enforces limits | The AI decides whether a trade is safe |


---

# Appendix A - Detailed MCP Schemas

## A1. get_account

```text
Input: { account_id?: string }
Output:
{
  account_id, owner, status, execution_mode,
  mandate_version, linked_venues[],
  equity_usd, gross_notional_usd,
  effective_leverage, updated_at_slot
}
```

## A2. simulate_trade

```text
Input:
{ market, side, notional_usd, order_type, max_slippage_bps }
Output:
{
  quote, current_state, projected_state,
  mandate_checks[], estimated_fees, estimated_slippage_bps,
  decision, reason_codes[], data_slot, expires_at
}
```

## A3. request_trade

```text
Input:
{
  quote_id, account_id, request_id,
  authorization_mode: "owner_signature" | "delegated",
  expected_mandate_version
}
Output:
ALLOW -> unsigned/signed transaction payload + expiry
REQUIRE_APPROVAL -> transaction for owner signature
REJECT -> structured reason + failed checks
```

## A4. reduce_position

```text
Input:
{ market, reduce_notional_usd?, target_safety_buffer_bps?, request_id }
Rules:
- action must be demonstrably risk-reducing
- delegation scope must include trade:reduce
- no market expansion or direction flip
- resulting state is simulated and receipted
```

# Appendix B - Proposed API Error Taxonomy

| Category | Example codes |
| --- | --- |
| AUTH | UNAUTHORIZED, TOKEN_EXPIRED, SCOPE_DENIED |
| POLICY | MAX_LEVERAGE_EXCEEDED, MARKET_NOT_ALLOWED, LOSS_BUDGET_EXCEEDED |
| RISK | MIN_SAFETY_BUFFER, RISK_CALCULATION_UNAVAILABLE |
| DATA | STALE_MARKET_DATA, VENUE_STREAM_GAP |
| QUOTE | QUOTE_EXPIRED, SLIPPAGE_LIMIT, LIQUIDITY_INSUFFICIENT |
| VENUE | VENUE_UNAVAILABLE, VENUE_REJECTED, MARKET_HALTED |
| CHAIN | TX_SIMULATION_FAILED, TX_EXPIRED, TX_UNKNOWN, TX_REVERTED |
| SYSTEM | RATE_LIMITED, INTERNAL_ERROR |

# Appendix C - Database / Index Schema (Offchain)

```text
users(id, wallet_pubkey, created_at)
markov_accounts(id, onchain_pubkey, owner_pubkey, status, indexed_slot)
mandates(account_id, version, onchain_pubkey, json_cache, hash, active_from_slot)
venue_links(account_id, venue_id, venue_account_id, status)
oauth_grants(id, account_id, subject, scopes, expires_at, revoked_at)
requests(request_id, account_id, actor, action_type, payload_hash, status, created_at)
receipts(id, account_id, request_id, decision, reason_code, tx_signature, slot, payload_json)
positions_snapshots(account_id, venue_id, slot, data_json)
risk_snapshots(account_id, slot, data_json)
telemetry_events(request_id, trace_id, component, severity, data_json, created_at)
```

# Appendix D - Test Matrix

| Test class | Examples |
| --- | --- |
| Policy unit | Boundary exactly at 3x; one atomic unit over limit; market allowlist; daily loss reset |
| Decimal/rounding | SOL/BTC lot sizes, quote/base conversions, bps arithmetic, overflow/underflow |
| Authorization | Expired permission, revoked scope, wrong account, wrong owner, replayed nonce |
| Data freshness | Old quote, WS sequence gap, block delay, changed mandate version |
| Venue | Partial fill, no fill, fill at slippage bound, rejected order, halted market |
| Risk | Open orders consume margin, funding changes collateral, positive PnL discount, isolated vs cross |
| MCP | Prompt injection text, malformed schema, unknown fields, duplicate request id, write call with read token |
| Reconciliation | Submitted but confirmation lost; landed tx after client timeout; state mismatch |
| Chaos | RPC failover, backend restart, WebSocket disconnect, keeper duplicate worker |
| UI | All rejected reasons understandable; no green state when data is stale |

# Appendix E - Operational Runbooks

| Incident | Immediate action |
| --- | --- |
| Phoenix API/WS degraded | Mark venue data stale, stop risk-increasing quotes, fall back to fresh snapshots/RPC where safe |
| Solana RPC degraded | Switch provider; do not resubmit unknown transaction until reconciled |
| Delegated signer suspected compromised | Revoke permission/rotate position authority; pause automation; notify users |
| Markov program vulnerability | Pause Markov execution paths if pause exists; publish status; recommend direct venue control |
| Receipt/reconciler lag | Do not claim execution complete; show pending/unknown state |
| MCP auth incident | Revoke affected tokens/issuer keys; disable sensitive scopes; preserve read-only where safe |
| Risk-calculation bug | Disable automated risk actions; retain read-only/raw venue data; patch and independently validate |

# Appendix F - Deferred / Experimental Execution Backlog

The major product capabilities above the existing-venue router are intentionally narrowed to two deferred execution layers:

- Solver/RFQ network: maker registry, quote requests, commitment/expiry rules, failure penalties, settlement guarantees, maker reputation, RFQ/venue route comparison and pilot monitoring.
- Markov-native perps: first-market selection from unmet-demand telemetry, matching mechanism, margin math, funding, oracle/mark, liquidation/backstop/insurance design, OI caps, maker onboarding, audit and incident operations.

Everything else required to make Markov useful as a multi-venue programmable perp product - canonical markets, adapters, router, portfolio state, mandates, risk tiers, lifecycle actions, Terminal, API/SDK, MCP and receipts - belongs in the devnet/private-beta build rather than this deferred backlog.

# Appendix G - Venue Gates and Open Questions Requiring Validation

| Question / gate | Decision deadline |
| --- | --- |
| Phoenix fresh-wallet onboarding through builder/no-referral path | Before closed-mainnet onboarding flow is frozen |
| Phoenix `phDEV...` cluster identity | **CLOSED 2026-09-10:** mainnet-beta deployment, not devnet; no Phoenix testnet/devnet execution claim is allowed |
| Pacifica agent-key ability to request withdrawal | Before Markov stores or operates an agent key |
| Drift PDA as subaccount delegate + practical CPI route on devnet | Before claiming on-chain Markov enforcement for Drift |
| Pacifica fee tiers after `fee_levels` returned 404 | Before fee-aware AUTO routing is marketed |
| Drift live perp fee/leverage/status parameters | Read on-chain at runtime; freeze parser before beta |
| Exact normalized safety-buffer definition across Phoenix/Pacifica/Drift | Freeze before user testing |
| Daily loss definition and reset timezone/epoch | Freeze before mainnet beta |
| Whether receipts are full PDAs or mostly program events + indexed proofs | Week 1-2 |
| MCP write-action compatibility across priority clients | Test during week 3; maintain live matrix |
| Initial pricing for builder/pro subscription | After usage data |
| Initial closed-mainnet beta caps | Freeze after security review and final venue semantics; start deliberately small |
| Cross-venue migration sequencing | Prototype pre-funded/open-new-before-close-old path; never assume atomic migration |
| Jupiter Perps adapter | Post-launch: design around async `PositionRequest`/keeper semantics; do not force CLOB semantics onto it |

# Appendix H - Reference Architecture Summary

```text
HUMAN / SOFTWARE / AI
        |
 Terminal | API/SDK | MCP
        |
        v
 MARKOV PROGRAMMABLE ACCOUNT
        |
 +------+-------------------+
 | Mandate | Permissions | Receipts |
 +------+-------------------+
        |
 Policy -> Risk -> Authorization
        |
 Capability-aware Venue Router
        |
 +------+-------------------------------+
 |                                      |
 v                                      v
TEST-STAGE EXECUTION                    PROTOCOL/READ PROOF
Pacifica testnet                        Phoenix live data + LiteSVM
Drift devnet                            (Phoenix has no devnet; LiteSVM only)
        |
        v
CAPPED CLOSED MAINNET
Phoenix + eligible Pacifica + Drift
        |
        +----> Jupiter Perps later adapter (async request model)
        |
        v
Private-beta completion: solver/RFQ -> native Markov pilot if earned
```

# Appendix I - Glossary

| Term | Meaning |
| --- | --- |
| Mandate | Owner-defined hard rules governing allowable account actions and states. |
| Preference | Soft optimization input that cannot override a mandate. |
| Programmable Perp Account | Markov account binding owner, mandate, actors, venues and receipts. |
| Permission | Scoped authorization for a non-owner actor or client. |
| Receipt | Structured evidence of a request, checks, decision, execution and resulting state. |
| Venue adapter | Capability-aware component translating Markov actions/risk into a specific perp venue. |
| Canonical market | Stable Markov market identity mapped to one or more venue-specific perp instruments, including contract multiplier and execution-model metadata. |
| Risk tier | Markov control metadata describing the current risk posture of a market; used for limits and UI, not a permanent asset label. |
| AUTO venue | Mode where Markov chooses among currently eligible venue adapters instead of pinning one venue; route comparison must respect different synchronous/asynchronous execution semantics. |
| Risk-reducing action | Action proven under defined semantics to reduce exposure/account risk; exact rules must be formalized. |
| MCP | Model Context Protocol, used to expose typed Markov tools to compatible AI hosts. |
| Hawkeye | Phoenix read-only risk/view program surfaced through Phoenix SDK/docs. |
| Native Markov perps | Markov-owned matching/margin/liquidation/market system; excluded from initial launch and introduced only as a gated final private-beta pilot. |

# Appendix J - Source Register (Current as of 10 September 2026)

| Ref | Source | URL |
| --- | --- | --- |
| [R1] | Solana Foundation - "Build Fully Onchain Perps on Solana" (1 Jun 2026) | https://solana.com/it/news/build-onchain-perps |
| [R2] | Colosseum - Crypto World's Fair / Hackathon page | https://colosseum.com/ |
| [R3] | DefiLlama - Solana Perp DEX & Futures Volume snapshot | https://defillama.com/perps/chain/solana |
| [R4] | Phoenix - Rise SDK documentation | https://docs.phoenix.trade/sdk/rise |
| [R5] | Phoenix - Vulcan CLI / MCP documentation | https://docs.phoenix.trade/cli |
| [R6] | Phoenix - Margin Math documentation | https://docs.phoenix.trade/phoenix/margin-and-risk/margin-math |
| [R7] | Ranger Finance - perp aggregation / smart routing product page | https://rangerfinance.netlify.app/ |
| [R8] | VOOI - Perps API / multi-venue backend | https://vooi.io/perps-api |
| [R9] | Colosseum - Tempo company profile | https://colosseum.com/companies/tempo |
| [R10] | IntentX documentation - onchain derivatives / intent architecture | https://docs.intentx.io/ |
| [R11] | Model Context Protocol - 2026-07-28 specification release | https://blog.modelcontextprotocol.io/posts/2026-07-28/ |
| [R12] | Model Context Protocol TypeScript SDK v2 | https://ts.sdk.modelcontextprotocol.io/v2/ |
| [R13] | OpenAI Help - Developer mode and MCP apps in ChatGPT (availability is time-sensitive) | https://help.openai.com/en/articles/12584461-developer-mode-apps-and-full-mcp-connectors-in-chatgpt-beta |
| [R14] | MCP Apps - Authorization guidance | https://apps.extensions.modelcontextprotocol.io/api/documents/authorization.html |
| [R15] | Phoenix - On-Chain Programs / CPI / Hawkeye | https://docs.phoenix.trade/sdk/on-chain-programs |
| [R16] | Phoenix - Accounts and position authority | https://docs.phoenix.trade/phoenix/collateral-and-accounts/accounts |
| [R17] | Colosseum - Accelerator | https://colosseum.com/accelerator |
| [R18] | Pacifica - Builder Program | https://docs.pacifica.fi/programs/builder-program |
| [R19] | Pacifica - Operation Types / agent wallet and API operations | https://docs.pacifica.fi/api-documentation/api/signing/operation-types |
| [R20] | Solana - DeFi development / composability overview | https://solana.com/docs/defi |
| [R21] | Jupiter Developer Platform - trader/market-maker platform includes Perps as an advanced-strategy surface | https://developers.jup.ag/ |
| [R22] | Drift v3 / updates - long-tail perp market support and expanded market-making direction; exact live listings remain time-sensitive | https://www.drift.trade/updates/introducing-drift-v3-built-to-outperform |
| [R23] | Drift Updates index - historical DOGE-PERP listing illustrates long-tail perp support; Markov must still discover live availability dynamically | https://www.drift.trade/updates |
| [R24] | Phoenix - Matching Engine | https://docs.phoenix.trade/phoenix/matching-engine |
| [R25] | Phoenix - LiteSVM Testing | https://docs.phoenix.trade/sdk/litesvm-testing |
| [R26] | Phoenix - API / Authentication | https://docs.phoenix.trade/api |
| [R27] | Phoenix - Registration / off-curve authorities | https://docs.phoenix.trade/sdk/register |
| [R28] | Pacifica - API Documentation | https://docs.pacifica.fi/api-documentation/api |
| [R29] | Pacifica - API Signing / Agent Keys | https://docs.pacifica.fi/api-documentation/api/signing/api-agent-keys |
| [R30] | Drift Protocol v2 repository / devnet | https://github.com/drift-labs/protocol-v2 |
| [R31] | Drift - Delegated Accounts | https://docs.drift.trade/getting-started/delegated-accounts |
| [R32] | Jupiter Perps - Pool / Position Request docs | https://developers.jup.ag/docs/perps |
| [R33] | Phoenix - Accounts, position authority, isolated subaccounts (SDK) | https://docs.phoenix.trade/sdk/accounts |
| [R34] | Phoenix - Trader onboarding, off-curve (PDA) authorities | https://docs.phoenix.trade/sdk/register |
| [R35] | Phoenix - On-chain programs / typed CPI / Hawkeye | https://docs.phoenix.trade/sdk/on-chain-programs |
| [R36] | Phoenix - Documentation index (llms.txt) | https://docs.phoenix.trade/llms.txt |
| [R37] | Pacifica - Documentation index (llms.txt) | https://docs.pacifica.fi/llms.txt |
| [R38] | Jupiter - Documentation index (llms.txt) | https://dev.jup.ag/docs/llms.txt |
| [R39] | Drift - self-hosted gateway (delegate mode) | https://github.com/drift-labs/gateway |
| [R40] | Solana RPC `getAccountInfo` on api.mainnet-beta.solana.com and api.devnet.solana.com, 2026-09-10 (cluster verification) | see Appendix K8 |
| [R41] | Pacifica - AI Trading Agent (in-app) | https://app.pacifica.fi/agent |


---

# Appendix K - Venue Verification Register (Pass #1, 10 September 2026)

This register is the operational source of truth for venue assumptions in this blueprint. Status tags mean: **LIVE** = observed via API/on-chain/SDK package during the pass; **DOCS** = official documentation; **SECONDARY** = lead only; **UNVERIFIED** = release gate, not an assumption.

## K1. Corrections Locked Into v1.4

| Earlier assumption | Verified reality |
| --- | --- |
| Phoenix does not list DOGE | Phoenix lists DOGE plus FARTCOIN, PUMP, SUI, JUP, XRP; 82 active markets observed |
| Drift has merely "40+ active markets" | SDK package contained 85 mainnet perp constants including expired prediction markets; live status/count must be read on-chain |
| Jupiter Perps is interchangeable with CLOB venues | Jupiter Perps is a JLP oracle-pool with keeper-filled requests, three markets and an RPC-verified mainnet-only deployment |
| World's Fair router can require Phoenix devnet execution | **RPC verification confirms no Phoenix devnet exists.** Use Pacifica testnet + Drift devnet for free end-to-end execution and Phoenix LiteSVM for local protocol proof. |

DOGE is live on **Phoenix, Pacifica and Drift** mainnet surfaces in this verification pass; the router demo does not need a hypothetical third venue for DOGE.

## K2. Phoenix Perpetuals - Verified Facts

- **Type [DOCS]:** fully on-chain perp exchange with FIFO orderbook + spline liquidity, matched on-chain.
- **Network [LIVE/DOCS]:** RPC `getAccountInfo` on 2026-09-10 confirmed Phoenix prod, Phoenix beta and Hawkeye are executable on mainnet-beta and absent on devnet. The `phDEV...` program is a second mainnet deployment, not a devnet environment. Local testing uses `phoenix-rise-litesvm-test` (runs the real program bytes in-process).
- **Fees/leverage [LIVE, market list 2026-09-10]:** sampled markets report taker 3.5 bps / maker 0.5 bps; per-market `leverageTiers` (size-dependent max leverage) are exposed in `GET /v1/view/exchange/markets`.
- **Program IDs [LIVE, `@ellipsis-labs/rise@0.4.67`]:**
  - Phoenix prod: `EtrnLzgbS7nMMy5fbD42kXiUzGg8XQzJ972Xtk1cjWih`
  - Phoenix beta env: `phDEVv4w6BcfkLrLNeXr8HhhgQxnxziVGXpGPcaadMf`
  - Hawkeye: `RiSeVw3ZjNfsaXPRb4mgaqYaEEt41pNNJoDvVh7pgQj`
  - Ember: `EMBERpYNE6ehWmXymZZS2skiFmCa9V5dp14e1iduM5qy`
  - Flight: `F1ightu9cujFYo34k9CabifLrJT8qzfDVM2Q7BqhJn2W`
- **Exchange keys [LIVE, 2026-09-10]:** global config `2zskx2iyCvb6Stg7RBZkt1f6MrF4dpYtMG3yMvKwqtUZ`; perp asset map `2nHGAaEw3D5dd4hVueaUNoygkQFmoeKqRQWnSPqSMFUC`; canonical mint `PhUsd11YkbjSaWjFncfAAmatntsjx3MgDR9B6g1ks3A`; global vault `csZXgw2G58hbiWc9ndxaxrQVYVvqdXgQzYLuznEzHJu`. Dynamic trader-index arrays must be read at runtime and never hard-coded.
- **API [DOCS]:** REST `https://perp-api.phoenix.trade`; WS `wss://perp-api.phoenix.trade/v1/ws`; public market data requires no key; trader routes use JWT.
- **Account model [DOCS]:** trader account PDA from authority/pda_index/subaccount_index/program; cross-margin at subaccount 0, isolated accounts at >0; USDC-only collateral wrapped to canonical mint via Ember.
- **Delegation [DOCS]:** `DelegateTrader` sets position authority for orders; owner keeps withdrawals. Registration path can accept off-curve authorities such as PDAs. Typed CPI contexts include delegated market orders, registration, limit orders, cancel-all, deposit/withdraw, collateral transfer, conditional/stop paths and Hawkeye margin views.
- **Gating [UNVERIFIED]:** onboarding needs Phoenix onboarder co-sign; fresh-wallet builder/no-referral flow must be tested before assuming open access.
- **Markets [LIVE]:** 82 active. Includes DOGE/FARTCOIN/PUMP, numerous RWAs (NVDA, TSLA, AAPL, SPY, QQQ, GOLD, SILVER, COPPER, WTIOIL). Sample venue maxima: BTC 40x, ETH/SOL/GOLD 25x, NVDA 20x, DOGE/SUI/JUP/PUMP/FARTCOIN 10x. Phoenix does not list WIF/BONK/PAXG/XAU under those symbols.

## K3. Pacifica - Verified Facts

- **Type [DOCS]:** hybrid off-chain CLOB with on-chain Solana settlement/custody.
- **Network [LIVE]:** mainnet API `https://api.pacifica.fi/api/v1`; testnet `https://test-api.pacifica.fi/api/v1`, 88 markets observed on 2026-09-10.
- **Signing [DOCS]:** Ed25519 over deterministic JSON with account/signature/timestamp/expiry-window headers.
- **Agent keys [DOCS/UNVERIFIED]:** separate keypair bound by main-wallet signature can sign trading operations. Ability to request withdrawal remains unverified and is a custody classification gate.
- **Account model [DOCS]:** unified margin; per-market cross/isolated; leverage settings; subaccounts; vaults with per-symbol whitelist/blacklist/max-leverage controls.
- **Markets [LIVE mainnet]:** 77. Includes DOGE 20x, WIF 5x, kBONK 10x, FARTCOIN/PUMP/SUI/JUP/XAU/PAXG/NVDA 10x, BTC/ETH 50x, SOL 20x; also EURUSD, USDJPY, CL, NATGAS, COPPER, PLATINUM, SP500 and SOL-USDC spot. Minimum order observed: 10 USDC.
- **Enforcement:** Markov can gate before signing, but there is no on-chain Markov mandate enforcement inside Pacifica order execution; venue-native vault rules are the venue-side policy mechanism.

## K4. Drift Protocol v2 - Verified Facts

- **Type:** fully on-chain Anchor program using multiple liquidity mechanisms (DLOB/JIT/AMM); optional Swift signed-order path exists in SDK but scope remains unverified.
- **Network [LIVE/DOCS]:** RPC verification on 2026-09-10 confirmed `dRiftyHA...` is executable on both mainnet and devnet; devnet USDC faucet is available.
- **Program ID [LIVE SDK config]:** `dRiftyHA39MWEi3m9aunc5MzRF1JYuBsbn6VPcn33UH` (differs from commonly memorised strings; always read from SDK config).
- **SDK [LIVE]:** stable `@drift-labs/sdk` 2.156.0; latest observed beta 2.163.0-beta.13; Python SDK also exists; self-hosted gateway can run as delegate.
- **Delegation [DOCS]:** one delegate per subaccount; may deposit, swap, place/cancel orders; **cannot withdraw**. PDA-as-delegate and CPI-friendly Markov path remain unverified.
- **Markets [LIVE package snapshot]:** 85 mainnet constants including DOGE/WIF/1MBONK/1MPEPE/FARTCOIN/PUMP/1KPUMP/SUI/JUP/PAXG/HYPE plus expired `*-BET` markets; 29 devnet constants include SOL/BTC/ETH/DOGE/SUI. Live status/leverage/fees must be read on-chain.

## K5. Jupiter Perps - Verified Facts

- **Type [DOCS]:** oracle-priced pool model with JLP as counterparty. Pool/custody supports SOL, ETH, BTC long collateral and stablecoin short collateral.
- **Execution [DOCS]:** `PositionRequest` PDA is created and a keeper executes/rejects it; market and trigger request types. This is not an orderbook/maker venue.
- **Program ID [DOCS/LIVE]:** `PERPHjGBqRHArX4DySjwM6UJHiR3sWAatqfdBS2qQJu`; RPC verification on 2026-09-10 confirmed it executable on mainnet and non-executable/system-owned on devnet.
- **Developer surface [DOCS/LIVE]:** no devnet deployment exists at the verified program address; Perps API docs carry a work-in-progress warning; official docs point to a community Anchor-IDL parser; Jupiter CLI supports perp operations.
- **Delegation:** none documented. Whether a Markov PDA can own a position is unverified.
- **Markets:** SOL, ETH, BTC only in this verified pass.
- **Launch decision:** keep outside launch set; add later only with a dedicated asynchronous request adapter rather than forcing synchronous quote/fill assumptions.

## K6. Cross-Venue Matrix

| Capability | Phoenix | Pacifica | Drift | Jupiter Perps |
| --- | --- | --- | --- | --- |
| Execution | On-chain CLOB + spline | Off-chain CLOB, on-chain settlement | On-chain DLOB/JIT/AMM | Oracle pool, keeper-filled |
| Free test path | LiteSVM local (no devnet) | Testnet API | Devnet + faucet | None - RPC-verified no devnet |
| Trade-only delegation | Position authority | Agent key (withdraw scope unverified) | Delegate, no withdraw | None documented |
| PDA account authority | Yes documented | No (off-chain key) | Unverified | Unverified |
| Typed CPI helper | Yes | N/A | IDL/public program, path unverified | Community IDL only |
| Market count snapshot | 82 live | 77 mainnet / 88 testnet | ~85 SDK constants; verify live | 3 |
| DOGE | Yes, 10x | Yes, 20x | Yes; live leverage runtime-read | No |
| WIF / BONK | No / No | Yes / kBONK | Yes / 1MBONK | No |
| FARTCOIN / PUMP | Yes / Yes | Yes / Yes | Yes / Yes | No |
| Gold | GOLD | XAU + PAXG | PAXG | No |
| US equities | Many | Some | No | No |

## K7. Gates Before Adapter Claims

1. Phoenix fresh-wallet onboarding via `build-register-ixs` with no referral code.
2. Pacifica agent-key `Request Withdrawal` test on testnet.
3. Drift PDA-delegate + CPI/order path on devnet.
4. Resolve Pacifica fee tiers and Drift live fee parameters for comparable route scoring.
5. **CLOSED 2026-09-10:** Phoenix `phDEV...` is mainnet-beta, not staging/devnet. Do not describe it as a free test environment.

## K8. RPC Cluster Verification (2026-09-10)

Method: JSON-RPC `getAccountInfo` against `https://api.mainnet-beta.solana.com` and `https://api.devnet.solana.com`.

| Program | Address | Mainnet-beta | Devnet |
| --- | --- | --- | --- |
| Phoenix prod | `EtrnLzgbS7nMMy5fbD42kXiUzGg8XQzJ972Xtk1cjWih` | executable (BPF upgradeable) | **missing** |
| Phoenix beta | `phDEVv4w6BcfkLrLNeXr8HhhgQxnxziVGXpGPcaadMf` | executable (BPF upgradeable) | **missing** |
| Hawkeye | `RiSeVw3ZjNfsaXPRb4mgaqYaEEt41pNNJoDvVh7pgQj` | executable | **missing** |
| Drift v2 | `dRiftyHA39MWEi3m9aunc5MzRF1JYuBsbn6VPcn33UH` | executable | **executable** |
| Jupiter Perps | `PERPHjGBqRHArX4DySjwM6UJHiR3sWAatqfdBS2qQJu` | executable | system-owned, not executable |

Consequence: the only free, on-chain, end-to-end executable venue is Drift (devnet). Pacifica testnet is a venue-hosted environment with off-chain matching, not a Solana cluster. Phoenix execution cannot exist before mainnet.

**Launch-set conclusion:** the verified test-stage router should be **Pacifica + Drift**, while Phoenix is the core local-on-chain enforcement proof and closed-mainnet adapter. Phoenix has no devnet; Jupiter Perps is also RPC-verified mainnet-only and is not in the launch set. This is a capability decision, not a judgment about Jupiter's product quality.

---

# Final Decision Sheet

| Decision | Status |
| --- | --- |
| Product category: Programmable Perps | LOCKED |
| Core primitive: Programmable Perp Account | LOCKED |
| Human interface: Markov Terminal | LOCKED |
| AI interface: MCP, not a proprietary AI agent | LOCKED |
| API/SDK: same account/router primitives as Terminal/MCP | LOCKED |
| Execution model for current build | **MULTI-VENUE - LOCKED** |
| Phoenix | Core protocol-enforcement adapter; LiteSVM/live reads in test stage; mandatory capped-mainnet target |
| Test-stage executable venues | **Pacifica testnet + Drift devnet - both required for genuine multi-venue execution** |
| Private-beta venue target | Phoenix + eligible Pacifica + Drift; at least two production-ready before multi-venue mainnet claim |
| Multi-venue router | **CURRENT BUILD - LOCKED** |
| Canonical market registry | CURRENT BUILD - LOCKED |
| Asset scope | Dynamic venue-supported universe; DOGE is verified live on Phoenix/Pacifica/Drift mainnet; canonical mapping includes contract multipliers and risk gates |
| Portfolio-level cross-venue risk | PRIVATE-BETA BUILD - LOCKED |
| Funding-aware expected-cost routing | CURRENT BUILD - LOCKED |
| Cross-venue rebalance/migration | PRIVATE-BETA BUILD; owner-approved first |
| Dev/test / World's Fair target | Multi-venue executable product on Pacifica testnet + Drift devnet, plus Phoenix LiteSVM/live-read proof |
| Conditional capped closed-mainnet beta | **LOCKED RELEASE TARGET** |
| Private mainnet beta launch | **LOCKED PRODUCT TARGET** |
| Jupiter Perps | Later adapter; not launch set; async keeper/request semantics must be modeled explicitly |
| Solver/RFQ network | DEFERRED FROM INITIAL LAUNCH; PB-3 completion pilot |
| Native Markov perps | DEFERRED FROM INITIAL LAUNCH; PB-4 final private-beta pilot, milestone-gated |
| Native DEX required for initial PMF | REJECTED |
| Token | NO CURRENT NEED |
| Trade finance as core product | OUT OF SCOPE; possible future application client only |

> **Founder north star** Markov should ship as a product people can actually use before it tries to own liquidity: one programmable account, one canonical market/risk language and a real multi-venue router across venue capabilities that have actually been verified, accessible through Terminal, API/SDK and MCP. The free test-stage route set is Pacifica + Drift; Phoenix is the strongest protocol-enforcement adapter and enters capped mainnet after access/integration gates. The private-beta program can then conclude by testing solver/RFQ execution and a tightly capped Markov-native market only after the external-venue product has generated the flow and evidence to justify them.


---

# Appendix L - Public Claims Register (Landing Page, Pitch, Submission)

Every externally visible claim must map to a verified fact in Appendix K. This register is the checklist for the landing page, the World's Fair submission, the demo video and any pitch material. "Test stage" means the Sep-Oct 2026 World's Fair build.

## L1. Claims that are TRUE today (2026-09-10) and may be used

| Claim | Basis |
| --- | --- |
| Markov is a programmable perpetuals account on Solana; the account enforces mandates before any trade is created | Part III |
| Five hard rules enforced on every action: max leverage, max notional, min safety buffer, max daily loss, approved markets | §14 |
| Multi-venue router executes on two venues in the test stage: **Pacifica testnet** and **Drift devnet** | K3, K4, K8 |
| Phoenix is integrated as a core adapter: live mainnet market data, canonical-market discovery, and a local LiteSVM proof of the on-chain policy/CPI path | K2 |
| Phoenix joins real-money routing in the capped closed-mainnet beta (Phoenix + Pacifica + Drift) | §54, K8 |
| DOGE, FARTCOIN and PUMP are live on all three launch venues' mainnet surfaces; WIF and BONK on Pacifica and Drift; equities and gold on Phoenix (some on Pacifica) | K6 |
| Every decision produces a receipt recorded by the Markov program on Solana (route, rules evaluated, authorization, tx signature, pre/post state) | §17 |
| MCP clients can read, simulate and request; they cannot withdraw and cannot bypass the mandate | §23 |
| Owner signature is required for every risk-increasing action in the test stage and closed beta | §29 |
| Markov is not a source of trade ideas; venue agents, third-party models and humans are | §12 |
| No token; no proprietary AI trading agent | Decision Sheet |

## L2. Claims that are FALSE or UNVERIFIED and must not appear

| Prohibited claim | Why |
| --- | --- |
| "Executable on devnet" applied to Phoenix, or "Phoenix + one more venue on devnet" | Phoenix has no devnet deployment (K8) |
| "Routes across Phoenix, Drift, Jupiter and more" / "Jupiter Perps" in any launch route list | Not in the launch set (K5) |
| A router card showing Phoenix as the SELECTED executable route in the test stage | Phoenix cannot execute before mainnet; show it as "mainnet quote · read-only in test stage" |
| "Best execution", "cheapest venue guaranteed", "fee-optimised" | Fee comparability is an open gate (K7 #4) |
| "Never get liquidated", "protects you from liquidation" | §41; no guarantee by design |
| "Delegated / autonomous trading" as a live feature | Owner-signed is baseline; delegation per venue is gated (§29) |
| "On-chain enforced on every venue" | Documented only for Phoenix; Pacifica is off-chain by construction; Drift PDA path unverified (K6) |
| "AI-powered trading", "AI finds the best trade" | Markov does not generate trade ideas (§12); venues already ship that |
| Any bps / leverage / cost figure presented as live data | Landing-page numbers are illustrative and must be labelled |
| "Devnet" as a product badge | Use "Test stage"; the environments are Pacifica testnet + Drift devnet + Phoenix LiteSVM |

## L3. Approved short-form wording

- Environment badge: **TEST STAGE · Pacifica testnet + Drift devnet · Colosseum Crypto World's Fair**
- Venue line: **Executes on Pacifica and Drift today. Phoenix integrated; live in the capped mainnet beta. Jupiter Perps: later adapter.**
- Stats row: **5 hard rules** / enforced on every action · **2 venues** / executable in the test stage · **3 venues** / at capped mainnet beta · **1 receipt** / per decision, recorded on Solana
- Router card labels: `Pacifica · testnet · SELECTED`, `Drift · devnet`, `Phoenix · mainnet quote · read-only in test stage`
- Positioning line: **Venue agents suggest trades. Markov decides what the capital is allowed to do.**
- Footer: **Programmable perpetuals on Solana. Existing venues first; native markets later, if earned. No token. Software infrastructure, not investment advice. Phoenix, Pacifica and Drift are independent venues; availability follows each venue's access model. All figures on this page are illustrative.**

---

# Appendix M - Change Log

## v1.4 -> v1.5 (10 September 2026)

1. **Appendix K8 added:** RPC cluster verification table (Phoenix prod/beta/Hawkeye mainnet-only; Drift on both clusters; Jupiter Perps not deployed on devnet).
2. **Appendix L added:** public-claims register derived from the 10 September landing-page review, which carried five prohibited claims (Phoenix on devnet, Jupiter in the route list, Phoenix as selected test-stage route, unqualified "on-chain verifiable", "2+ venues executable on devnet").
3. **§10 / §12 venue-agent positioning:** Pacifica's in-app AI Trading Agent and Phoenix Vulcan establish that venues own "AI trade ideas" per venue; Markov's differentiation is stated as governing what those ideas may do with capital, not generating them.
4. **§54 blocker 6:** capped mainnet Phoenix smoke budget as the only pre-beta path to real Phoenix execution.
5. **Sources:** Pacifica references R18/R19/R28/R29 moved to docs.pacifica.fi; R33-R41 added.
6. **K2/K4 details:** Phoenix sampled fees and leverage-tier exposure; Drift program-ID caveat.
7. Terminology: residual "devnet release/build" wording replaced with "test-stage".

## v1.3 -> v1.4 (10 September 2026)

Phoenix beta-cluster hedges removed after RPC verification; Jupiter Perps devnet status closed; Drift devnet status confirmed by RPC; K1/K2/K5/K6/K7 and §28/§49/§51/§53/§54/Appendix G/H updated accordingly.

## v1.2 -> v1.3 (10 September 2026)

Launch set re-based on Verification Pass #1: Pacifica testnet + Drift devnet as executable test-stage venues; Phoenix as core adapter with LiteSVM proof; Jupiter Perps removed from launch set; canonical registry gained `contract_multiplier` and `execution_model`; §29 execution modes rewritten per venue; §34 populated; Appendix K added.
