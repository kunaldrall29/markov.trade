# 01 — Product Thesis
Markov Book · 31 August 2026 · v0.2

Planning intent. Not a live product claim. Not an APY. Not a token.

---

## 1. The idea in one paragraph

Markov Book is a house-run, policy-bounded book on Solana. A depositor puts USDC into an account they still control. A house agent runs one strategy at a time inside an on-chain allowlist: venues, names, caps, slippage, expiry. The agent cannot withdraw. Every attempted action, including the ones the policy blocks, writes a public receipt with a machine-readable BlockReason. People show up for the book. They stay because the operator cannot empty the account the way an unbounded admin key can.

The first book is crypto perps. Later books reuse the same lock on whatever Solana already trades in size — including tokenized-equity perps — when a venue for that name passes the same checklist. The asset list is a policy field. It is not a new company.

> I earn the book. They cannot take the pile.

## 2. Order that may not be reversed

**Yield is the product. Authority is the lock. Receipts are the proof. AI is a desk, not a brand.**

A grant, a Colosseum blurb, or a deck that flips this order is wrong.

## 3. Why the permission-layer storefront does not pull

“Give an agent capital, keep the keys, configure a policy” is a real mechanism and a weak product.

- Nobody wakes up wanting to configure a policy.
- Vaults and copy desks exist because people want someone good to make them money.
- Authority matters after a burn. April 2026: audits checked code at rest; authority failed at runtime; limits were raised; vaults drained. Caps that live in the owner’s policy are the lock, not the click.
- An empty marketplace of operators has no demand. Demand attaches to a book with a public curve.

The mechanism is kept. The storefront is the book.

## 4. What people buy

**First-order job:** put idle USDC to work on Solana markets without handing a human or a bot the keys.

**Second-order job:** do that in a shape that survives an operator-key story.

**Third-order job (later books):** the same job on names that are not SOL/ETH/BTC — tokenized equities and other listed perps — without a second custody model.

The third job is in the thesis because the lock is an allowlist. Solana already concentrates tokenized-equity flow. That is a future `tokens: [...]` line, not a Gate B page and not an “RWA startup” headline.

## 5. What “AI market maker” is allowed to mean

Not “an LLM reads X and trades.”

1. **Deterministic core.** Inventory bands, hedge ratio, max gross, max per-venue, funding-entry rules, kill conditions. Runs if every model is down.
2. **Model with a veto.** Regime: chop / trend / halt. A model may propose. The guard may refuse. A refusal is a first-class event.
3. **Research sidecar.** Funding, open interest, liquidations, basis. Public social flow last, as a feature, never as an order router. For an equity-named book, the sidecar adds hours and borrow/funding on that name — still features.
4. **Proof of hedge.** Net delta, inventory, venue, funding collected, turnover, drawdown, every refusal. If a number cannot be checked from chain or venue APIs, it does not appear.

If the guard cannot be written on one page without a model, there is no book. There is a chatbot with a wallet.

## 6. What we are not

- Not a permission-system company. Policy is the lock, not the homepage.
- Not Hyperliquid HLP. We do not quote Jupiter as a CLOB house MM. Jupiter Perps is trader-to-pool. First live venue is unnamed until the checklist passes.
- Not JLP. JLP is long-biased basket plus fee yield.
- Not an RWA issuer, transfer agent, or tokenized-stock venue. We never mint equities. We may, later, allowlist a perp or hedge that already trades a tokenized name.
- Not an open operator marketplace on day one. Phase 3.
- Not a token or points product.
- Not “AI analyses everything.” One strategy, one envelope, one dashboard per book.

## 7. Book One — the only object we ship first

Single house strategy. SMA: one mandate per depositor. Funds do not commingle.

| Field | Gate B value |
|---|---|
| strategy_id | BOOK_ONE |
| venue | `demo_perps` (same adapter trait as a future real venue) |
| tokens | USDC-d, SOL-d |
| shape | near delta-neutral *intent* — not a promised APY |
| earn | funding when shorts are paid, plus any spread the venue actually permits |
| cut | flatten or reduce when regime = trend |
| never | withdraw authority |

Owners may only tighten caps. `owner_withdraw` works in Active, Paused, Revoked, Expired.

Pooled NAV / shares = Phase 2 decision after review. Default remains SMA. A shared pile is the object that got drained when admin authority failed.

## 8. The asset surface is one program, several books

The product is not “crypto perps forever.” The product is a bounded book whose **allowlist** can point at any market the venue adapter can trade.

| Book | When it is allowed to exist | What the allowlist points at |
|---|---|---|
| **Book One** | Now → Phase 1 | Crypto perps (SOL first). `demo_perps`, then one real venue after checklist. |
| **Book Two** | After Book One has a public curve and a passing venue | A second *crypto* name or a second venue — still not equities unless that venue is the one that passed. |
| **Equity book** | Phase 2 at the earliest, only if | (a) a Solana perp or hedge venue lists a tokenized-equity name with programmatic open/close, readable funding/OI, settlement mint, test env, ToS that allows an agent; (b) the token mint is on the owner’s allowlist; (c) hours / halt rules live in the guard; (d) we do not invent custody of the underlying share. |
| **Other names** | Phase 3+ | Only as templates other operators publish under the same lock. |

Tokenized equities belong in the thesis for three reasons:

1. **Demand already sits next to us.** Reported Q2 2026 tokenized-equity volume was concentrated on Solana. That is a market we can *later* point a mandate at. It is not proof anyone wants *our* book.
2. **The lock is the compliance surface.** Venue allowlist + token allowlist + caps ≈ what a desk writes in a mandate letter. An equity-named perp is a tighter allowlist, not a new architecture.
3. **We refuse the issuer job.** We do not wrap stocks. We do not run a transfer agent. If the name cannot be traded through a passing venue adapter, it does not enter a policy.

Until (a)–(d) are true, copy says “Solana perps.” It does not say “we trade Apple.”

## 9. Who it is for

1. Solana-native depositors already in JLP, lending, or a delta-neutral wrapper who want the operator unable to withdraw.
2. Later: the same depositor, same lock, a book whose names include tokenized-equity perps.
3. Phase 3: agent / bot builders who run *their* book under the program.
4. Not first: people who have never used a perp. That is a different company.

## 10. The wedge

Crowded: yield vaults, hedged-JLP desks, funding harvesters, catalog “mm vaults.”

Uncrowded combination:

- a house book with a public curve
- in-program authority bounds
- public refusal receipts
- the same envelope reusable when the listed names change

The refusal is not why someone deposits $500. It is why they pick this $500 slot over the one where limits could be raised.

## 11. Complete product by phase

Phases are gated. A calendar does not start a phase.

### Phase 0 — Gate B (now → 27 Sep, then Colosseum as public build)

Hosted Book One on devnet. `demo_perps`. Paper log. 90s tape: fund, act, OverTxCap, revoke, Revoked, withdraw. No APY. No named live venue. No equities on the page.

**Done:** GATE-B.md B1–B15 green.

### Phase 1 — One real venue, still tiny or fake money

Book One talks to one passing Solana perp venue. Same SMA. Guarded mainnet only with per-mandate cap ($100–$500), ≤20 owners, $5k ceiling, kill switch, upgrade plan, zero out-of-policy fills.

**Not in Phase 1:** fees, open deposits, second venue, token, equity names.

### Phase 2 — A book a stranger can judge

30–90 days of public marks. On-chain delta/gross if a trusted mark exists. Batching design for SMA fan-out. Decision: stay SMA or open a reviewed pool.

**Equity book may be designed here, not marketed**, and only if a venue for that name has passed the checklist. Shipping it still waits for Book One’s curve not being embarrassing.

**Kill the company if:** hedge error stays outside band 14 days unexplained; one out-of-policy mainnet fill; the book is a paused museum; nobody outside the team deposits $100 after seeing the curve.

### Phase 3 — Platform (only after Phase 2 is not embarrassing)

External operators publish templates. Owners subscribe, tighten-only. Book One stays listed. An equity-named template is just another template with a tighter allowlist. SDK / MCP = how an external agent runs a book under the gates — tooling, not the storefront. Reputation from receipts including refusals.

### Phase 4 — Fees and credit

In-program take-rate on operator performance. Bonds only after refusal graphs exist. Token utility specified next to fee settlement, not before. No ticker as the product.

## 12. What the complete company is (one sentence)

**Book One is the whole product until strangers deposit. If they do, the company is more books — crypto first, listed names later — under the same lock: owner keeps the pile, every no is on chain.**

## 13. Copy rules (so the thesis does not leak into Gate B)

| Phrase | Allowed when |
|---|---|
| Solana perps | Now |
| Named live venue | Checklist passed + FACTS row |
| Tokenized equities / “equity book” | Phase 2 design docs and investor appendix. Not on `/book`. Not on Gate B tape. |
| “We trade [ticker]” | Never, until that mint is on a live allowlist and a receipt exists |
| APY | Never as a promise. Marked path only, labeled marked |
| Marketplace / Float grid | Phase 3 |
| Token ticker | Phase 4 spec, not a launch |

## 14. Signals this thesis is allowed to lean on

- Ecosystem: Foundation asked for onchain perps *and* the vaults / MM / structured layer around them. We are the complementary object, not a new venue.
- Capital / research: perps and agent-wallet infra keep getting paper; tokenized-equity volume on Solana is why an equity book is *plausible later*.
- Onchain economics: demand for *yield on Solana markets* is borrowed from existing vaults. Demand for *this lock* is unproven. Paper + 20 conversations remain the test.
- Prior art: “mm vaults” and “agent wallet” already sit on public idea shelves. Difference is SMA + refusals + one house book first.

If a sentence would not survive a chain check or a paper log, it does not belong in this file as a claim. It belongs in BACKLOG or it is deleted.
