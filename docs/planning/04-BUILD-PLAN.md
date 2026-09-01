# 04 — Build Plan
Markov Book · 30 August 2026 · v0.1

Calendar context: today is 30 August 2026. Colosseum window is 28 September – 2 November 2026. Treat the window as the public build of Book One + the first real-venue adapter spike, not as a reason to reopen the marketplace.

## 0. Roles

| Seat | Owns |
|---|---|
| Protocol | program, adapters, receipts, tests |
| Agents | book-core, risk-guard, sidecar, paper runner |
| Surfaces | Book One page, receipts filter, bot wiring |
| Truth | FACTS, STATUS, daily paper log, no unverified sentences |

If a seat is empty, that work does not start. Do not “AI-code” a market maker nobody can explain.

## 1. Week 0 — Freeze and paper (1–7 Sep)

**Output:** this pack signed; venue checklist started; paper runner ticking.

- Confirm ADRs 01–12. Any disagreement is a written ADR change, not a Slack vibe.
- Venue checklist against Jupiter Perps API, Pacifica, Velocity public docs. Name nothing in copy until a venue passes.
- Paper runner v0:
  - pull mark, funding, OI for SOL-PERP (public sources)
  - apply inventory + delta bands + daily-loss halt
  - write `paper/YYYY-MM-DD.md` every day
- Do not touch product UI this week except a blank `/book` route.

**Exit:** three paper days exist, even if returns are ugly.

## 2. Week 1 — Venue adapter + agent skeleton (8–14 Sep)

**Output:** `demo_perps` + agent that can skip / act / get refused on devnet.

- Adapter trait: `mark`, `open`, `close`, `increase`, `reduce`, `positions`.
- `demo_perps` program or CPI target using existing mock style. Mark from a pulled price. Position state in the mock.
- Agent: one crate/service. Flags `SHADOW` vs `DEVNET`.
- Risk-guard as a pure function with unit tests. No network.
- Wire `propose → program → receipt` for one happy path and one OverTxCap.

**Exit:** two signatures on devnet, listed in FACTS.

## 3. Week 2 — Dashboard + withdraw path (15–21 Sep)

**Output:** hosted Book One page that a phone can use.

- `/book` reads `/v1/receipts?strategy=BOOK_ONE` and `/v1/book/stats`.
- Fund, revoke, withdraw via wallet adapter. Withdraw never disabled by state.
- Indexer recognizes demo_perps events.
- Health: `chainReady` real.

**Exit:** demo script beats 1–4 work on a preview URL.

## 4. Week 3 — Kill switch, redteam, paper continues (22–28 Sep)

**Output:** full 90s tape; refusals on a schedule.

- Telegram revoke against a live Book One mandate.
- Redteam ticks: OverTxCap, SlippageExceeded, Revoked, one spend cap.
- Paper log now has ≥14 days or an explicit “started late on DATE” note. Do not invent days.

**Exit:** eight demo signatures committed. STATUS can be re-run.

## 5. Window weeks — Colosseum (28 Sep – 2 Nov)

In-window work, disclosed as such if rules require in-window novelty:

1. First real-venue adapter spike (Jupiter or passing CLOB) on **devnet or paper**, not unguarded mainnet capital.
2. Proof-of-hedge fields that need venue APIs (funding, mark, positions).
3. Cold-start doc: stranger → funded mandate → first receipt in 15 minutes.
4. Optional: MCP read tools for Book One (`list_mandates`, `get_receipts`, `propose_action` still fail-closed).

Do not: restyle the marketing site as the week’s work; open Float as a marketplace; ship a token; quote an APY.

Submit with ≥48 hours buffer. Every link opened logged-out.

## 6. Engineering standards (every week)

```
NON-NEGOTIABLES
- Fail closed.
- Every allow and every block that hits the program emits a receipt
  with a canonical BlockReason.
- Unpause is owner-only.
- BlockReason codes are append-only once emitted.
- No scope outside the week’s prompt. Else BACKLOG.md.
- End of session: SESSION_LOG + FACTS.
```

Tests that must exist before calling a week done:

- risk-guard: one test per halt condition
- program: existing invariant suite still green
- adapter: mark freshness failure → no trade
- UI: withdraw button enabled in Revoked (component test)

## 7. What we stop building

| Stop | Why |
|---|---|
| Operator marketplace cards as the homepage | No book to sell |
| Generic MCP-first story | Tooling, not the product |
| “Eleven refusal types” as the hero | Mechanism, not the job-to-be-done |
| Token / points / leaderboard | Violates proof-before-raise |
| Reading X as an order source | Sidecar only, after the book works |

## 8. Tools

- Solana devnet, Anchor, existing monorepo layout
- Railway for agent + indexer + api
- Vercel for `/book`
- Paper runner can be the same agent binary
- No new chain, no new token, no new database vendor

## 9. Risk register (build)

| Risk | Tell | Mitigation |
|---|---|---|
| Venue APIs move under us | Integration dies mid-window | Adapter trait + demo_perps always shippable |
| Agent overtrades | Dashboard looks like a slot machine | Default action is skip; interval 60s |
| Paper book is noise | Temptation to hide days | Publish ugly days or do not invite capital |
| Scope leak into marketplace | Familiar code paths | ADR-01; PR label `out-of-scope` |
| Single upgrade key | Drift-shaped criticism | Document in SECURITY.md; mainnet plan written |
