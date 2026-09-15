# 05 — Markov Landing Page (markov.trade) — Build Prompt

Inherit `00_Build_Conventions.md`. Supersedes the earlier landing prompt. Stack: Next.js static export, self-hosted fonts, no analytics, no third-party scripts, Vercel. Brand assets from `brand/` only.

## 1. Brand application

- Nav: `markov-mark.svg` (28px) + "markov" in Bricolage Grotesque 800; on the ink CTA section use `markov-mark-white.svg`.
- Hero: `markov-wordmark-stipple.png` is the only place the stippled wordmark may appear (interactive on web per the kit).
- Colors from `markov-colors.css`; page ground cream `#ECEBE6`; clay cards; glass panels; ink terminal panels; Markov Blue `#2F3BE0` for the one phrase and the one button per view; Verified `#2E8B57` and Signal `#E8552B` for status only.
- Type per the kit: display Bricolage Grotesque 700–800, tracking −0.03 to −0.04em, line-height 0.96–1.0; body Instrument Sans 17–20px at 1.5; UI 13–15px at 500–600; JetBrains Mono for eyebrows (caps, 0.08–0.12em tracking), receipts and numbers.
- Surfaces: clay (inset light top-left, inset shade bottom-right, soft outer drop), glass (46% white, blur 26px, 1px white edge, top highlight), ink (`#1C1C1A`, thin top highlight; blue → `#8EA0FF`, green → `#7CF29A`). Radii 18–30px. Never a hard drop shadow alone.
- OG image: `brand/markov-og-image.png`. Favicon: mono mark.
- Voice: declarative, numeric, honest about stage. Prohibited phrases per the kit and the claims policy.

## 2. Content rule: everything shown is live or receipted

No illustrative numbers. The mandate-check card and the router card render **a real receipt from the rehearsal account**, fetched at build time from the control plane (`/receipts/{request_id}` for a pinned, public rehearsal receipt) and rebuilt on deploy. If the fetch fails the build fails. The card carries the receipt id and an explorer link. Stats are computed at build time from the same source: number of hard rules in the active mandate, number of venues executable for the rehearsal account, number of venues integrated, receipts recorded to date.

## 3. Sections and copy

**Nav badge:** stage label from config — `CAPPED MAINNET · Pacifica + Drift executable · Phoenix integrated` (text generated from `/venues/capabilities` of the rehearsal account at build time; never hand-typed).

**Hero**
- H1: `Perps that obey` / `your rules, not` / `just your orders.` ("your rules" in Markov Blue)
- Sub: `Markov is a programmable perpetuals account on Solana. Humans, software and AI express leveraged intent; the account enforces what the capital is allowed to do — across Pacifica, Drift and Phoenix.`
- Stats (build-time): `{n} hard rules` / `enforced on every action` · `{n} venues` / `executable today` · `{n} venues` / `integrated` · `{n} receipts` / `recorded on Solana`
- Mandate-check card: the pinned real receipt (market, side, notional, checks with observed/limit, decision, route, cost bps, receipt id, explorer link).
- Venue line generated from capabilities.

**01 — The problem** (unchanged copy): `A stop-loss is one trigger. A mandate is a policy.` + three cards (Orders, not outcomes / Bots hold too much power / Every venue, a new dialect).

**02 — The control layer:** `Own the account. Borrow the liquidity.` Cards: One programmable account (real `MarkovAccount` fields from the rehearsal account, owner truncated); Five hard rules (the rehearsal mandate's real values); Multi-venue router (a real `/routes/compare` snapshot stored with the pinned receipt; Phoenix row tagged `read-only` if not executable for that account); Every action leaves a receipt (the pinned receipt's reason chips); Risk-tiered universe (tier ceilings from the real mandate); Actor-aware permissions (scopes from the real permissions rows, withdraw struck through); Terminal · API · MCP.

**03 — Position lifecycle:** six steps + the no-guarantee notice (unchanged).

**04 — MCP, not a trading bot:** tool chips from `/mcp/tools` at build time; conversation panel replaced by two real receipts: one MCP `request_trade` approved by the owner, one rejected with `MAX_LEVERAGE_EXCEEDED`, both with receipt ids. Caption: `Real receipts from the rehearsal account.`

**05 — Invest (new):** eyebrow `05 — INVEST`; H2 `Tell software how your money may be invested.`; one real Invest rule from the rehearsal account (assets, cadence, mode, budget, reserve) and its last two receipts (one executed with tx link, one skipped with reason). Disclosures line: `Tokenized stocks are price exposure, not shares. Issuers keep pause and delegate powers. Availability follows the issuer's eligibility rules.`

**06 — Release path:** cards generated from `config/stages.json` so labels can never drift from reality: current stage, next stage, gated stages. No dates that aren't in the config.

**CTA:** `Open the terminal` → `NEXT_PUBLIC_APP_URL`; `Read the docs` → docs site.

**Footer:** `Programmable perpetuals on Solana. Existing venues first; native markets later, if earned. No token. Software infrastructure, not investment advice. Phoenix, Pacifica and Drift are independent venues; Markov is not affiliated with them and availability follows each venue's access model. © 2026 Markov.` + `Markov — v{version} · {build date}` + brand kit link `markov.trade/media`.

## 4. Interactions

Hover on a check row → rule definition; hover on route segments → bps; `Expand` on the receipt → raw JSON; stippled wordmark interaction per the kit; reduced-motion respected.

## 5. Build-time checks

- Receipt fetch succeeds and matches the pinned id; stats non-zero.
- Claims check: prohibited phrases; "Jupiter" appears zero times; "Phoenix" never adjacent to "devnet"/"testnet".
- Lighthouse ≥ 95 ×4 on mobile; no external runtime requests except self-hosted fonts and the two images.

## 6. Acceptance

- [ ] Deployed at markov.trade with the official assets; brand kit page untouched.
- [ ] Every number on the page traces to a receipt id or a capabilities response captured in the build log.
- [ ] Stage badge text equals the control plane's stage config for the rehearsal account.
