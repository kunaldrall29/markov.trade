# 04 — Markov Terminal (Next.js) — Build Prompt

Inherit `00_Build_Conventions.md`. This supersedes the earlier Terminal prompt: **no mock mode, no fixtures, no illustrative numbers.** The Terminal is the reference client of the account API; it renders only what the control plane returns.

## 1. Stack

Next.js (App Router) + TypeScript + Tailwind; TanStack Query (server state) + Zustand (UI state); `@solana/wallet-adapter` (mainnet-beta; devnet only via an explicit `NEXT_PUBLIC_ENV=devnet` build for CI); `lightweight-charts` (candles/lines); Recharts (bars, gauges, heatmap, timeline); zod schemas imported from `packages/sdk`; OpenAPI client generated from the control plane spec. Deploy on Vercel; edge caching disabled for authenticated routes.

## 2. Brand

Use `brand/` exactly (Conventions §8). Nav: `markov-mark.svg` at 28px + wordmark in Bricolage Grotesque; on ink surfaces `markov-mark-white.svg`; favicon from `markov-mark-mono-black.svg`; OG `markov-og-image.png`. Tokens from `markov-colors.css`: cream ground, clay for pressable surfaces (inset light top-left, shade bottom-right, soft drop), glass for readable panels (46% white, blur 26px, 1px white edge), ink for the terminal and dark panels (blue shifts to `#8EA0FF`, verified green to `#7CF29A`), one blue action per view. Type: Bricolage Grotesque for headlines and large numerals; Instrument Sans for UI (13–15px, 500–600); JetBrains Mono for every number, id, hash and eyebrow. Signal colors only as status.

Stage labelling is mandatory in the top bar: `CAPPED MAINNET` when the account is in the capped stage, `REHEARSAL` for rehearsal wallets, plus per-venue `INTEGRATED · read-only` on Phoenix until execution is enabled for that account.

## 3. Rules

1. Real data only. Every panel reads from the API; empty states explain what is missing ("No linked Pacifica account", "Official price feed unavailable for AAPLx — Reference Safe disabled").
2. Freshness on every actionable panel; stale disables Execute with the reason chip.
3. Phoenix appears with live mainnet data and is not selectable for execution unless the API's `capabilities().executable` is true for this account.
4. Jupiter Perps never appears.
5. Outcome first, proof expandable: one-line decision, expandable checks table (rule / observed / limit / result).
6. Owner signs every risk-increasing action and every mandate change; there is no auto-execute toggle. Invest execution is keeper-run inside the on-chain cap and is shown as such.
7. No withdraw in the Terminal; venue links only.
8. Manual venue pin re-runs policy; a REJECT stays REJECT.
9. No forecast overlays; indicators limited to SMA, EMA, Bollinger, ATR, RSI, MACD, off by default.
10. Build-time claims check (`scripts/check-claims.ts`) fails on prohibited phrases.

## 4. Routes

`/overview` · `/markets` · `/markets/[canonical_market_id]` · `/portfolio` · `/positions` · `/invest` · `/invest/rules/[id]` · `/mandate` · `/risk` · `/receipts` · `/receipts/[request_id]` · `/approvals` · `/integrations` · `/venues` · `/settings`.
Shell: left rail, top bar (mark, stage label, account selector, mandate version chip, wallet). `⌘K` palette. Keyboard `L`/`S` in market workspace.

## 5. Screens (fields from the API; nothing computed client-side except formatting)

- **Overview:** equity, gross notional, effective leverage `x / limit`, daily PnL, daily-loss budget used, status pill; Chart A equity & PnL; Chart B mandate headroom gauges; active mandate summary; linked venues strip with env and health; recent receipts; alerts; pending approvals count.
- **Markets:** category tabs, search; table (market, mark from best executable venue, 24h, funding per venue on hover, OI, venue dots, tier chip with reasons, allowed badge, ceiling); sparklines from real 24h marks; unsupported market → explicit empty state.
- **Market workspace:** header (symbol, canonical id, tier, allowed, ceiling, venue dots); Chart C candles per venue with mark/index/entry/liquidation overlays and the mandate ceiling band when a size is typed; Chart D mirrored depth per executable venue; Chart E funding by venue with horizon shading; venue panel cards (mark, spread, depth ±10/±50 bps, funding, max leverage, margin modes, fees or `unknown`, health, freshness; Phoenix card read-only banner); trade ticket (side, notional/base, leverage bounded by ceiling, order type per capability, limit price, max slippage, horizon; venue selector AUTO/pin; live policy preview; Chart F route stacked bars; Simulate; Request; state machine footer); positions in this market with reduce/close.
- **Portfolio:** aggregate cards; Chart G exposure by venue × market; Chart H risk over time with mandate thresholds and receipts as markers; Chart I funding accrued; reconciliation banner.
- **Positions:** cross-venue table with venue env tags, liquidation estimate labelled `estimate`, freshness; reduce/close drawers with policy preview and signing.
- **Invest:** rule list (asset/basket, cadence, mode, window, next due, budget used, status); rule builder (pinned allowlist with mint + verified flag + issuer powers disclosure; amount; cadence; window; mode with Reference Safe availability per asset; budget; ceiling; reserve floor; max cost bps; single-asset cap) → diff → sign invest mandate + sign USDC recurring delegation (two wallet signatures, both shown with exact program ids); upcoming actions; history with executed/skipped/rejected receipts and reasons; pause/revoke; Chart M contributions vs holdings (scaled UI), Chart N cost per purchase vs limit, Chart O executed/skipped timeline; Buy / Sell now panel (owner-signed, same mandate, same adapter) when enabled by config.
- **Mandate:** current version (five rules, activation slot, hash copyable, status); soft preferences; new version flow (form → diff → sign); pause/unpause; history with receipts.
- **Risk:** current state block; Chart J stress heatmap labelled `simulated`; Chart K bullet charts current vs projected; scenario runner; reduce-only proposal panel.
- **Receipts:** list with filters; Chart L timeline; receipt view with full `ActionReceipt`, checks table, route snapshot, pre/post hashes, `Copy JSON`, `Verify on chain` (Markov program event/PDA in explorer, correct cluster).
- **Approvals:** pending MCP proposals with diff/checks; Sign (wallet) or Decline; both receipted.
- **Integrations:** MCP client registration/approval, scopes, expiry, revoke; tool catalog from `/mcp/tools`; activity feed.
- **Venues:** per venue: env, link status, account id, balances, health, capabilities, last reconciled slot, deposit/withdraw outbound links; link flows (Pacifica: sign binding message, exact bytes shown; Drift: initialize + deposit tx); Phoenix: `INTEGRATED · read-only` until enabled.
- **Settings:** RPC endpoint (from config, read-only display), units, notifications.

## 6. Charts (all from API; venue colours fixed: Pacifica `#2F3BE0`-tinted blue family, Drift signal amber, Phoenix graphite for read-only)

A equity/PnL line · B headroom gauges ×5 · C candles (per venue source) · D mirrored depth · E funding multi-line · F route stacked bars · G exposure stacked bars · H risk over time + receipt markers · I funding accrued bars · J stress heatmap · K bullet charts · L receipt timeline · M contributions vs holdings · N cost per purchase · O executed/skipped timeline. Every chart caption carries venue, env, freshness; axes labelled; tooltips mono; no animation > 200 ms; reduced-motion respected; text summary via `aria-label`.

## 7. Trade flow state machine

`IDLE → PREVIEW → ROUTED → SIMULATED → REQUESTED → AWAITING_SIGNATURE → SUBMITTED → PENDING → RECONCILING → RECEIPT | FAILED`. Stale data during PREVIEW…AWAITING_SIGNATURE returns to PREVIEW with `STALE_MARKET_DATA` and forces a re-quote. The Solana transaction the user signs includes `markov.record_decision` + venue instruction + `markov.finalize_decision`; the Terminal shows all instructions with program ids before the wallet prompt.

## 8. Wallet and signing

Mainnet-beta cluster; hard error on mismatch. Solana transactions built server-side, returned base64, signed by the wallet, submitted via the API. Pacifica operations signed with `signMessage` after displaying the human-readable operation and payload hash; refuse if unsupported. Invest: two signatures on rule creation (invest mandate tx; Subscriptions recurring delegation tx); revoke is one signature. No key material in the browser (audited: localStorage, IndexedDB, cookies).

## 9. States

Loading, empty (with next action), error (taxonomy code + retry), stale, degraded, paused, wallet disconnected (read-only browse), wrong cluster, API unreachable (banner; cached view marked `cached · <age>`).

## 10. Responsive and accessibility

Desktop-first (≥1280 three-column workspace); 1024–1279 stacked panels; <1024 ticket as bottom sheet; phones read-only for trading with a notice. Keyboard reachable; real tables; contrast ≥ 4.5:1 on cream and ink; chips carry text not just colour.

## 11. Acceptance

- [ ] `pnpm build` green; claims check zero violations; no string literals for prices/amounts in components.
- [ ] Rehearsal walkthrough on mainnet with a rehearsal wallet: link Pacifica and Drift; SOL-PERP $50 long via AUTO with route bars and receipt; reject on $5,000 request; reduce; Invest rule for $5/week NVDAx created with both signatures; keeper receipt appears; MCP proposal approved and one declined; all receipts open from the UI with explorer links. Evidence saved per `07`.
- [ ] Phoenix panels show live mainnet data with the read-only tag; no Execute control on Phoenix.
- [ ] Grep the bundle: zero "Jupiter Perps", zero "illustrative", zero "mock".
- [ ] Lighthouse accessibility ≥ 95; keyboard-only completion of the trade and invest flows.
