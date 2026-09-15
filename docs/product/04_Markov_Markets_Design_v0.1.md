# Markov Markets — Deliverable Equity Perps

**Design document v0.1 — 15 September 2026**
Native Markov perpetual markets on tokenized stocks: physically backed by the tokens themselves, deliverable, session-aware, with mandates enforced at the venue. This is the first native market in the Markov family and the only one justified by design rather than by routed demand. Status: design and counsel stage; nothing here is scheduled before the World's Fair build ends (12 Oct 2026). Verified facts are marked; everything else is a gate.

---

## 1. One sentence

**A perp on NVDA that is backed by real NVDAx in a pool, can be settled by taking or delivering NVDAx, prices off the exchange when Nasdaq is open and off the on-chain spot market when it isn't, and lets the people who already hold NVDAx earn by supplying it.**

## 2. Why this market, why native

**The gap (verified).** Phoenix lists NVDA, TSLA, SPY, QQQ and others as cash-settled perps; Pacifica lists some. Both settle against an oracle; when the exchange is closed the index is stale or synthetic while the spot token trades on Raydium and Jupiter with billions a quarter behind it (Q2 2026: ~$5.8B tokenized-equity DEX volume on Solana; June alone > $2B; 200k+ xStocks holders). No Solana perp venue takes tokenized stocks as margin (Drift's spot-collateral list has none; Pacifica unverified), and none links the perp to the spot token.

**The demand (verified, secondary).** Jupiter Lend added SPYx, QQQx, NVDAx and TSLAx as collateral for leveraged positions; SHIFT's leveraged tokenized equities trade on Jupiter; xStocks endorsed NestUSD, a protocol letting holders borrow against tokenized equities; SpaceX's tokenized listing did $7.6M in two hours across three issuers. People want leverage and hedges on these tokens, and they want yield on holding them.

**Why native is justified here and not for crypto perps.** For BTC/SOL the existing venues are mechanically sound and deep; a native market would need routed demand to justify itself (Perps PB-4 rule). For equities the mechanism at existing venues is structurally wrong off-hours and the spot market that could fix it already exists on-chain. The signal is the spot market, not Markov's router.

## 3. Market structure

### 3.1 Pool model (physically backed)
Modelled on Jupiter Perps' pool/custody design (verified: single Pool, per-asset Custody accounts, keeper-fulfilled requests), with tokenized stocks as the pool assets:

- One **Custody** per listed equity token (NVDAx, TSLAx, SPYx, QQQx, AAPLx…) and one per stable (USDC first).
- **Longs borrow the equity token from the pool** — a $10k NVDA long at 2x reserves $10k of NVDAx from the custody; max long open interest per market = pool holdings of that token.
- **Shorts borrow USDC from the pool** and are collateralised by USDC or by the equity token itself.
- LPs deposit any listed asset and receive **MLP** (Markov LP) shares; the pool is the counterparty to every position and is hedged *by construction* on the long side because it holds the actual token.
- Fees: borrow fee (utilisation-based, per asset, per hour), open/close fee in bps, price-impact fee scaled by trade size vs custody depth, delivery fee. No funding rate in the CLOB sense; borrow fees play that role, as in Jupiter Perps.

### 3.2 Deliverable settlement
- **Long delivery:** a long can close by paying remaining notional in USDC and receiving the underlying NVDAx from the custody (Token-2022 transfer to the trader's ATA).
- **Short delivery:** a short can close by delivering NVDAx to the custody and receiving USDC.
- **Why it matters:** delivery gives arbitrageurs a hard pin between perp and spot; it lets an Invest holder convert exposure to ownership and back without leaving the account; and it makes the market's off-hours price *the* price rather than a synthetic.
- Delivery follows the token's own rules (transfer hook accounts, scaled UI amounts, issuer pause); a paused token disables delivery and flags the market (§6).

### 3.3 Session-aware index
- **Market open:** Chainlink Data Streams US Equities (regular-hours feeds; multi-sourced; `marketStatus`, bid/ask, staleness fields) or Pyth equity feeds with market-hours status. Verified: Chainlink streams are live for SPY, QQQ, NVDA, AAPL, MSFT and more, consumed on Solana by GMX-Solana and Kamino; Pyth sponsors equity feeds on Solana mainnet and devnet.
- **Extended / overnight sessions:** Chainlink 24/5 streams exist but are single-sourced for extended and overnight hours (Chainlink's own risk note) — used only as a sanity band, never as the sole index.
- **Market closed (nights, weekends, holidays):** index = on-chain spot TWAP of the equity token across the deepest pools (Raydium CLMM, Byreal, Jupiter route price), bounded by a deviation band around the last official close (e.g. ±3%); outside the band the market enters **guarded mode** (§6). Off-hours is where the market is unique and where it is most dangerous; both are true.
- **Open gap:** at the first regular-hours print after a closed session the index steps to the exchange price; positions are marked continuously through the step; liquidations during the first N minutes use a widened buffer.

### 3.4 Collateral
- USDC (day one).
- The listed equity tokens themselves at a haircut (e.g. 70–80% initial, lower off-hours), enabling a covered short on a holding and equity-financed leverage. Haircuts widen automatically while `marketStatus ≠ open`.
- Cross-margin per Markov account across Markov Markets positions; isolated mode available per position.

### 3.5 Leverage and tiers
Per-market caps set by liquidity of the spot token and session: e.g. NVDA 5x open / 2x closed; a thin name 2x / 1x. Caps are Markov risk tiers applied at the venue, so a user's mandate, the tier, and the venue cap resolve to one ceiling exactly as in the router.

### 3.6 Listing criteria
A token is listable when it is on the Invest registry (pinned mint, issuer metadata, liquidity floor met over 30 days, corporate-action feed available, official price stream available). New issuers' tokens arrive through the Invest Listings module when it exists. No market lists on hype alone; SpaceX-style day-one listings need a "pilot market" mode with hard OI caps.

## 4. Participants

| Participant | Role | Where they come from |
| --- | --- | --- |
| LPs | supply equity tokens and USDC, earn borrow/open/close/impact fees, bear pool risk | Invest holders (idle stocks earn), yield-seeking USDC |
| Traders | leverage long/short, hedges on holdings, deliverable exposure | Invest users, agents via MCP, router users |
| Keepers | execute position requests, liquidations, index updates, session transitions | Markov-run first, third-party set later (bonded) |
| Markov control plane | mandates, permissions, receipts at the venue; router treats Markov Markets as one more adapter | existing |

## 5. Position lifecycle

`request (owner-signed, mandate-checked) → keeper executes at index ± impact → position (borrowed asset, collateral, entry, session at entry) → marked continuously → close by cash settlement or delivery → receipt`

Requests, not orders: the trader submits a `PositionRequest` (market, side, size, collateral, max index, delivery preference); a keeper fills or rejects it against the live index within a TTL. This is Jupiter Perps' pattern and it keeps matching off the critical path of the program. A CLOB is explicitly not built (Decision D10).

## 6. Risk model

| Risk | Treatment |
| --- | --- |
| Pool directional exposure (net short by construction when longs dominate) | LP-side awareness in the MLP price; utilisation caps per asset; borrow fee curve steepens at high utilisation; optional pool hedging via existing venues is **not** in v0 |
| Off-hours manipulation of the spot TWAP | multi-pool TWAP with depth weighting; deviation band vs last close; guarded mode: no new risk-increasing positions, wider haircuts, liquidations only on sustained breaches |
| Open-gap liquidations | widened buffer window after open; partial liquidation first |
| Issuer actions (pause, permanent delegate, delisting, pausable transfers) | market flags `ISSUER_PAUSED`; delivery disabled; cash settlement only; new positions disabled; LP withdrawals of that token disabled while paused; disclosed everywhere |
| Corporate actions (splits, dividends via scaled-UI rebase, symbol changes) | positions and custody accounted in raw amounts; a rebase is a multiplier change, never a PnL event; splits handled by index and size adjustment on the event slot |
| Oracle failure or staleness | stale fails closed: no new positions, no liquidations on stale data; fall back to last valid index with widened buffers |
| Regulatory | derivatives on securities plus pooled lending of securities; non-US only; eligibility mirrors the issuer's lists; counsel before any testnet with real tokens |
| Liquidity bootstrapping | Invest LP supply and a seed pool; no token, no points; fee share only |

## 7. Program design (sketch)

Accounts: `MarketsConfig`, `Pool`, `Custody{asset}`, `MlpMint`, `Market{equity}` (index config, session state, caps, tier), `Position`, `PositionRequest`, `SessionOracle` (last official close, market status, TWAP state), `IssuerFlags{asset}`.
Instructions: `add_liquidity / remove_liquidity`, `create_position_request / execute / reject`, `add_collateral / remove_collateral`, `close_cash / close_delivery`, `liquidate`, `update_index` (keeper, with oracle proofs), `set_session_state`, `flag_issuer_event`, admin listing/caps.
Markov integration: every request CPIs into the Markov program's policy check (mandate, permissions, tier) and emits an `ActionReceipt`; the router exposes Markov Markets through the same `PerpVenueAdapter` interface, with `execution_model = pool_request`, `delegation_model = markov_native`, `onchain_enforceable = true`.
Token-2022: all equity custody and delivery paths use the Token-2022 program with transfer-hook account resolution and scaled-UI-aware accounting.

## 8. Relationship to Invest and the router

- **Invest → Markets:** idle holdings can be supplied to the pool under a rule (an Invest Stage 2 "yield" capability with a venue that is Markov's own); a hedge rule ("protect if NVDA drops 10%") becomes a covered short on the same account.
- **Router → Markets:** Markov Markets is one venue among Phoenix, Pacifica and Drift for equity exposure; the instrument-aware router scores spot vs Markov perp vs external perp by lifecycle cost and session.
- **Markets → Invest:** delivery turns a perp into an Invest holding; receipts are shared.

## 9. Alternatives considered

| Alternative | Why not (now) |
| --- | --- |
| Native CLOB with market makers | needs MMs and a matching engine; contradicts D10; pool model needs no counterparty to launch |
| Cash-settled oracle perps like the existing venues | duplicates them without fixing off-hours; no reason to exist |
| Synthetic 1x exposure tokens | issuer-of-derivative role with no backing; worse regulatory profile than a backed pool |
| Using Jupiter Perps' program as-is | its custodies are SOL/ETH/BTC only and its docs are work-in-progress; the model is reused, the program is not |

## 10. Stages and gates

| Stage | Window | Contents | Gate to enter |
| --- | --- | --- | --- |
| M0 — Design and counsel | after 12 Oct 2026 | this document to v1.0; legal memo on derivatives on securities, pooled lending, eligibility; issuer conversation (Backed) on pooling/lending terms; oracle contract (Chainlink Data Streams or Pyth) | Fair submitted |
| M1 — Devnet prototype | Invest Stage 2 window | pool, custody, position request/execute, cash close, delivery with mock Token-2022 mints carrying the same extensions; session oracle with Pyth devnet equity feeds; Markov policy CPI; receipts | M0 memo says proceed |
| M2 — Mainnet pilot, one market | with Perps PB-4 | NVDA only; OI cap low six figures; LP allowlist; guarded off-hours mode on by default; delivery live; external audit | M1 complete; audit; counsel sign-off; eligibility gating live |
| M3 — Expand | after M2 metrics | SPY/QQQ/TSLA/AAPL; equity collateral; third-party keepers; instrument-aware routing across venues | pilot metrics (§11) |

**Blocking gates (open):**
1. Issuer terms — whether Backed's prospectus/terms permit pooling and lending of xStocks and what the permanent delegate and pause powers mean for a pool. Signals are positive (issuer-endorsed borrowing protocol, DeFi portal, "can be used as collateral, pooled in an AMM" in Solana's case study) but nothing is verified against the legal documentation.
2. Regulatory posture per jurisdiction for a derivative on a security offered non-US; entity and licensing; counsel.
3. Oracle: Chainlink Data Streams on Solana for the listed names — access terms, on-chain verification cost, and the exact `marketStatus` semantics; Pyth as fallback. Verified to exist; not verified as integrated.
4. Off-hours TWAP source set and manipulation resistance measured against actual pool depths.
5. Pacifica spot-collateral list (whether any venue already takes xStocks as margin).
6. Audit scope and budget.

## 11. Pilot metrics

Perp-to-spot basis during closed sessions (target: inside the deviation band > 95% of the time); share of closes settled by delivery; LP share of pool supplied from Invest holdings; utilisation and borrow-fee revenue per asset; liquidation count at open vs closed; zero mandate violations; zero positions opened on stale data.

## 12. Claims (extends the registers)

**May say (once M2 is live):** physically backed by the tokens in the pool; deliverable; session-aware index with disclosed off-hours method; mandates enforced at the venue; non-US only.
**Must not say:** "Nasdaq price 24/7", "no liquidation risk", "backed by shares" (backed by *tokens* that are backed by shares — say the chain), "yield guaranteed", "available in the US".
