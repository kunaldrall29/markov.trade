# 06 — PMF, Revenue, Strategy
Markov Book · 30 August 2026 · v0.1

## 1. Are we pre-PMF?

**Yes. Unambiguously.**

Product-market fit is repeatable demand from people who are not the team, at a price that pays for the cost of serving them. We have:

- a mechanism (mandates, receipts) that is live on devnet
- zero evidence that strangers will deposit because of that mechanism
- a new first-order offer (a house book) that does not yet have a public curve

What we have is a **hypothesis**, not fit.

### The hypothesis

> Solana-native depositors will put USDC into a near-delta-neutral perps book if (a) the marked path looks competitive with JLP-hedged and funding-harvest vaults they already understand, and (b) the operator cannot withdraw.

(a) is the product. (b) is the reason they pick us after April 2026. If (a) is missing, (b) does not matter. If (a) exists and (b) is missing, we are the same as every other vault and we lose on brand, liquidity, and desk quality.

### How we will know

| Signal | Pre-PMF (now) | Weak pull | Fit worth scaling |
|---|---|---|---|
| Conversations | “cool permissions” | “what’s the APY / who’s the book” | “here is my wallet, when can I deposit” |
| Deposits | team only | 10–20 external owners, small size, they come back after a red week | organic inflow without a points program |
| Retention | n/a | they do not all leave after the first flat month | 30-day keep after a drawdown |
| Pull vs push | we explain the product | they ask for a cap increase | they ask for a second strategy |

Until the middle column is true, treat every grant, hackathon, and thread as **distribution experiments**, not as proof.

## 2. Why this can have demand (without lying)

Demand for the *job* is proven adjacent:

- People already buy JLP (~high-hundreds of millions TVL). That is long-biased fee yield, not our book, but it proves “park capital next to Solana perps.”
- People already buy HLP-style community MM exposure (peaked near $600M, now much lower when returns flattened). That proves MM-vault demand is real and **fickle**.
- People already buy hedged-JLP products from professional desks. That proves the exact risk shape is occupied — we do not get a clean greenfield.

Demand for *our* combination is not proven. The honest edge is not “AI.” It is:

1. public receipts including refusals
2. operator cannot withdraw
3. house book first, so there is something to look at

That is a positioning edge. It is not a yield edge. Yield still has to come from the book.

## 3. Revenue potential — model, not a forecast

No number below is a projection of what we will earn. It is a model of **how** money could show up if the book works.

### Who pays

| Stream | When it turns on | Who pays | Notes |
|---|---|---|---|
| Performance fee on Book One | Phase 2, after a curve exists | Depositors | Typical crypto vault: 0/10 to 2/20. MVP charges **0**. Charging before a curve is how you look like a fund with no track record. |
| Management fee | Optional, later | Depositors | Only if the book is boring and sticky (HLP-like). Easy to kill demand. |
| Protocol fee on external operators | Phase 3 | Operators | Share of their performance at settlement. This is the actual company-shaped stream. |
| Marketplace fee | Phase 3 | Owners or operators | Thin. Do not build the business on it. |
| Data / Score access | Phase 3+ | Other agents, allocators | Only if refusal graphs exist. |

### Worked example (illustrative)

Assume, after fit, Book One has $2M TVL, 10% net before fee, 10% performance fee.

- Gross alpha ≈ $200k / year
- Performance take ≈ $20k / year
- That does not pay a team.

Assume Phase 3 works and external operators run $20M through mandates at the same 10/10.

- Operator alpha ≈ $2M
- Protocol 10% of operator take (if operators also charge 10%) ≈ $20k — still small
- Better structure: protocol takes 10–20% of *operator performance fees*, plus a tiny AUM fee. Still a $100k–$400k business at that scale.

**Venture-shaped revenue only appears if either:**

- Book One (or the platform) reaches HLP-like or JLP-adjacent TVL, or
- the mandate layer becomes default infrastructure for other agent apps (B2B take-rate on many books).

Both are post-PMF. Planning as if the $8k grant is a business is a category error. The grant buys time to run the paper book and the guarded window.

### Unit economics to watch

- Cost to run the agent + RPC + indexer vs fees.
- Capacity: SMA fan-out cost grows with owners until batching exists.
- Adverse selection: if only noisy depositors arrive, the book’s life gets harder — that is a risk, not a revenue line.

## 4. Strategy

### Position

Lead with the book. Close with the lock.

- Homepage / Book page: marked path, inventory, receipts.
- One line under it: operator cannot withdraw; every block is on chain.
- Never lead with MCP, “mandate layer,” or “AI that reads X.”

### Sequence (the whole company)

```
paper book → devnet Book One → guarded tiny mainnet
    → public curve → fees
        → other operators
            → B2B mandate layer
```

Any skip is how pre-PMF products raise and then stall.

### Distribution

| Channel | Use | Do not use for |
|---|---|---|
| Public paper log + receipts | Trust | Fake APY |
| Colosseum / Superteam | Forced ship + audience of builders | “We won so it works” |
| X | Daily book facts, refusal of the week | Token talk, dunking on victims of exploits |
| Conversations (n=20) | The actual PMF instrument | Script-reading |

The twenty conversations ask one question:

> Would you put $500–$5,000 USDC into this book at its marked path if the operator cannot withdraw and every trade plus every block is public?

Write the answers down. They outrank this document.

### Competitive stance

| Competitor class | They have | We have | We lose if |
|---|---|---|---|
| JLP | size, default button on Jupiter | tighter risk shape, withdraw lock | we pretend we are JLP |
| Hedged-JLP desks | actual desks | receipts + non-withdraw | our hedge is worse and we hide it |
| HLP | the category-defining MM vault | Solana + authority story | we claim we are HLP on a pool venue |
| Agent-wallet / spend limits | UX distribution | on-chain evidence | we stay a permission slideshow |
| Other AI vaults | narrative | a veto and a public no | we become another wrapper |

### What we will not do to “find” PMF

- Points for deposits
- Promised APY
- Marketplace liquidity that is just the team
- Rebranding as the work
- Raising on the AI label before the paper folder is public

## 5. Pre-mortem

**If this dies, it dies in one of four ways:**

1. The book is not real. An LLM with a wallet overtrades and the dashboard is a casino.
2. The book is real and still nobody cares, because JLP is one click and we are a science project.
3. We pool too early and inherit the object that got drained in April.
4. We stay in love with the permission layer and never ship a book.

(1) and (4) are in our control this month. (2) is the actual market test. (3) is a decision we already made not to take in MVP.

## 6. Bottom line

| Question | Answer |
|---|---|
| Pre-PMF? | Yes |
| Is there a market for the job? | Yes, adjacent and crowded |
| Is there a market for *this* product? | Unknown. Testable in weeks, not quarters |
| Revenue now? | No |
| Revenue later if the book works? | Fees on the house book (small), then take-rate on other operators (the company) |
| Venture-scale? | Only if the mandate layer becomes how other agents take deposits, or the book reaches serious TVL |
| What to do Monday | Paper runner + venue checklist + stop selling the lock as the store |

Treat Markov Book as a **pre-PMF yield product with an authority wedge**, not as an AI company and not as a protocol company. Protocol companies without a first buyer stall. AI companies without a book are costumes. Yield products without a lock are plentiful. The bet is the three together — and the bet stays a bet until strangers deposit.
