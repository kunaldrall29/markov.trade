# 02 — Technical Decisions (ADR pack)
Markov Book · 30 August 2026 · v0.1  
Binding until an ADR is explicitly superseded. Tempted additions go to BACKLOG.md.

---

## ADR-01 — Product shape: house book first, marketplace later

**Decision:** Ship one house strategy (Book One) before any operator marketplace.

**Why:** A marketplace of policy controls has no demand. HLP proved the sequence: run the house book, publish the curve, then let others in.

**Consequence:** Float listings, third-party operators, strategy templates as a consumer surface — all Phase 3. The program may still stamp `strategy_id`. The UI does not pretend there is a market.

**Status:** Accepted.

---

## ADR-02 — Account model for MVP: SMA mandates, not a pooled NAV

**Decision:** Each depositor gets their own mandate account. Funds never commingle. `owner_withdraw` succeeds in Active, Paused, Revoked, and Expired.

**Why:** This is the property that survives an operator-key theft and an admin-key story like April 2026. A pooled vault reintroduces a single drain target and a fund-shaped regulatory object.

**Cost we accept:** N depositors = N transactions per book action. Fine at devnet and at a guarded mainnet cap of tens of owners. Batching is a named Phase-2 engineering item, not a sneak-in.

**Revisit when:** a scoped review is done and we want a $5k–$50k TVL pool under hard in-program ceilings.

**Status:** Accepted for MVP.

---

## ADR-03 — Venue: do not pretend Jupiter is an order book

**Decision:** Book One’s first *live* venue is chosen from what the venue actually is, not from the HLP metaphor.

Facts as of late August 2026:

- **Jupiter Perps** is trader-to-JLP. Execution is request → keeper fulfillment. You are a trader against a pool, not a quoter on a CLOB. Markets: SOL, ETH, wBTC. Max 6 positions. Oracles: Edge / Chainlink / Pyth.
- **JLP** is the LP token. Holding JLP is long-biased inventory plus fee yield. Hedging that inventory (short the basket on a perp) is a real strategy and already occupied by professional desks.
- **Velocity** (former Drift) is being rebuilt after the April 2026 authority exploit. Perps-only, USDT settlement planned, private beta then public relaunch. Order-book + JIT auction historically. Unstable integration target until public program IDs and docs freeze.
- **Pacifica** and other Solana books exist and carry real volume. Integration quality must be verified in Week 0, not assumed.

**MVP venue policy:**

| Phase | Venue | Role |
|---|---|---|
| Paper (Week 0–2) | Public Jupiter + Hyperliquid + Solana perp stats | Shadow book. No capital. |
| Devnet MVP | `demo_perps` adapter | Same interface as a real venue. Fills marked from live oracle prices. Receipts are real. PnL is marked, not promised. |
| First real venue (M2) | Jupiter Perps **or** a live Solana CLOB, whichever passes the Week-0 checklist | One venue only. |
| Second venue | Only after Book One is stable on venue one | Hedge or inventory transfer, not “more AI.” |

**Week-0 checklist (must pass before a venue is named in public copy):**

1. Documented programmatic open / close / cancel.
2. Known settlement mint and collateral.
3. Position and funding readable on-chain or via a stable API.
4. Devnet or paper environment that matches production semantics closely enough to test refusals.
5. License / ToS allows a vault-style agent.

Until the checklist passes, public copy says “Solana perps,” not a brand.

**Status:** Accepted.

---

## ADR-04 — What the agent is

**Decision:** Three processes, not one “AI.”

```
research-sidecar  →  proposes features / regime
book-core         →  deterministic quotes, hedges, sizes
risk-guard        →  veto; fail closed; writes BlockReason
     ↓
mandate program   →  last gate; CPI into allowlisted venue only
```

- `book-core` must be runnable with the sidecar killed.
- `risk-guard` cannot be an LLM. It is thresholds + invariants.
- An LLM may sit in the sidecar or as a proposer inside book-core. It may never sign. It may never bypass the guard.

**Invariants the guard always checks (mirrors on-chain gates where possible):**

- net delta within band
- gross exposure within cap
- per-venue notional within cap
- funding-regime allow (optional: sit flat if funding adverse beyond N hours)
- oracle freshness
- slippage bound
- daily loss circuit: halt new risk if mark-to-market drawdown hits X% on the day

On-chain gates remain the last word. Off-chain guard exists so we do not spam the chain with proposals we already know will refuse — but every on-chain proposal still goes through the program.

**Status:** Accepted.

---

## ADR-05 — Mandate program is the lock, not a rewrite

**Decision:** Reuse the live Markov program model.

Keep:

- fail-closed gate order
- ActionReceipt + RefusalReceipt + BlockReason
- owner-only unpause
- emergency key = pause/revoke only
- `strategy_id` on receipts
- spend caps for data / compute

Add for Book One (program delta, small):

- optional **net-delta** and **gross-exposure** fields on policy, *or* enforce those only off-chain in v0 and on-chain in v1
- a real venue adapter interface used by `demo_perps` and later Jupiter / CLOB
- `strategy_id = BOOK_ONE` constant for the house book

**v0 choice (MVP):** delta / gross enforced off-chain by risk-guard + recorded on receipts as metadata where the event schema already allows it. On-chain enforcement of delta is a Phase-1 program change because it needs a mark price the program trusts.

**Do not add in MVP:** share accounting, NAV oracle, performance-fee waterfall, staking, points.

**Status:** Accepted.

---

## ADR-06 — Proof of hedge is a product surface

**Decision:** The primary UI is not a marketplace. It is a dashboard that a stranger can audit.

Minimum fields, all sourced from chain + venue APIs, never from a private ledger file:

- TVL-in-strategy (sum of mandate balances)
- net delta (USD)
- gross exposure
- funding collected (session / 7d / 30d)
- realized + unrealized PnL (marked, labeled as marked)
- actions / refusals / refusal rate
- last N receipts with BlockReason
- hedge error (target delta vs actual)
- circuit state (live / paused / revoked / daily-loss halt)

Empty states are honest. A zero-refusal book is displayed as unremarkable, not as excellence.

**Status:** Accepted.

---

## ADR-07 — Paper before size

**Decision:** Two weeks of a shadow book against live prices is a gate for any public APY-shaped sentence.

Paper rules:

- Same risk-guard as production.
- Same BlockReason vocabulary.
- Daily public log: return, max DD, turnover, hedge error, refused actions, why.
- No depositors.
- If the book cannot be shown ugly weeks and all, do not invite capital.

**Status:** Accepted.

---

## ADR-08 — Devnet money is fake; claims stay labeled

**Decision:** Devnet USDC-d / DEMO mints are not yield. Marketing, grants, and hackathon copy must say “devnet, marked PnL, unaudited.”

A recorded demo may show: deposit → agent acts → cap refuses → owner withdraws in revoked state. It may not show an APY as if it were earned in the wild.

**Status:** Accepted.

---

## ADR-09 — Keys and authority

**Decision:**

| Key | Holds | Can do | Cannot do |
|---|---|---|---|
| Owner wallet | Depositor | fund, amend (tighten), pause, unpause, revoke, withdraw | operate the book |
| Operator key (house agent) | Railway process / HSM-like file, env only | propose actions inside policy | withdraw, unpause, amend caps up |
| Emergency key | Separate process / human | pause, revoke | unpause, withdraw, trade |
| Upgrade authority | Documented single key on devnet; mainnet plan = multisig then freeze | deploy | treated as the Drift lesson pointed at ourselves |

Devnet upgrade authority being a single key is an accepted risk and must be written in SECURITY.md. It is not hidden.

**Status:** Accepted.

---

## ADR-10 — Data plane

**Decision:** Indexer is chain-native (program logs → parse events → Postgres). The API never invents receipts. `data/ledger.json` is not evidence.

Public read model is narrow and append-only: receipt id, time, mandate, operator, strategy, venue, token, amount, result, block_reason, tx signature. No raw instruction payloads.

**Status:** Accepted.

---

## ADR-11 — x402 and payments

**Decision:** Spend budgets exist in-program in MVP (OverSpendCap refusals). Facilitator settlement is not required to call the product live on devnet. Pin a facilitator only when a real invoice is paid from a mandate.

**Status:** Accepted (deferred settlement).

---

## ADR-12 — Regulatory shape (engineering implication)

**Decision:** MVP SMA mandates are closer to “you kept your account and hired a bounded agent” than to “you bought shares in a fund.”

Still:

- no performance-fee collection in the program in MVP (record theoretical fee off-chain only)
- no solicitation of APY
- geo and ToS are a human task before any mainnet dollar
- pooled NAV in Phase 2 is explicitly a different legal object and gets its own review

**Status:** Accepted as a constraint on the fee engine. Not legal advice.

---

## Non-goals (repeat until it sticks)

Pooled NAV, token, points, multi-operator marketplace, copilot mode, prediction-market adapter, “reads all of X,” restyling the marketing site as the way we find PMF, raising before the paper book exists.
