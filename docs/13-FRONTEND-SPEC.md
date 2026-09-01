# 13 — Frontend Spec
Markov Book · 31 August 2026 · v0.1

The design already exists at `markovhq.grok.me`. This document records it exactly as fetched on 31 Aug 2026 so it can be rebuilt in the repo without re-deriving anything, then specifies `/book`, which is the page Gate B actually grades.

---

## 1. Design tokens (verbatim from the live stylesheet)

```css
:root {
  color-scheme: light;
  --mk-bg:          #f6f5f1;
  --mk-surface:     #ffffff;
  --mk-raised:      #eceae4;
  --mk-fg:          #161513;
  --mk-muted:       #5c5953;
  --mk-subtle:      #8a8680;
  --mk-line:        #1615131a;   /* fg @ 10% */
  --mk-line-strong: #16151338;   /* fg @ 22% */
  --mk-allow:       #2c6b48;
  --mk-refuse:      #a33b32;
  --mk-refuse-fg:   #ffffff;
  --mk-accent:      #161513;
  --mk-accent-fg:   #f6f5f1;
}
.dark {
  color-scheme: dark;
  --mk-bg:          #111110;
  --mk-surface:     #1a1a18;
  --mk-raised:      #242422;
  --mk-fg:          #f4f3ef;
  --mk-muted:       #a8a59e;
  --mk-subtle:      #6f6c66;
  --mk-line:        #f4f3ef1f;
  --mk-line-strong: #f4f3ef47;
  --mk-allow:       #6fba88;
  --mk-refuse:      #e07068;
  --mk-refuse-fg:   #161513;
  --mk-accent:      #f4f3ef;
  --mk-accent-fg:   #111110;
}
```

Semantic aliases (Tailwind v4 `@theme`): `bg`, `surface`, `raised`, `fg`, `muted`, `subtle`, `line`, `allow`, `refuse`, `refuse-fg`, `accent`, `accent-fg`.

**Colour discipline:** exactly two semantic colours carry meaning — `allow` (green) and `refuse` (red). Everything else is paper, ink, and two greys. Never use `allow`/`refuse` decoratively. A red badge always means the program said no.

## 2. Type

| Role | Family | Use |
|---|---|---|
| Display + body | **Outfit** (400/500/600/700) | headlines, prose, buttons |
| Data | **JetBrains Mono** (400/500) | every number, every receipt row, every label-eyebrow, every reason code |

Loaded from Google Fonts in one stylesheet link, with `preconnect` to `fonts.googleapis.com` and `fonts.gstatic.com`.

Scale in use: hero `text-5xl` → `sm:text-7xl`, `leading-[0.95]`, `tracking-tight`, `font-semibold`. Section heads `text-4xl` (`md:text-5xl`). Card heads `text-2xl` / `text-xl`. Body `text-sm`/`text-base` `leading-relaxed` in `muted`. Eyebrows and tiles: mono at `11px`/`10px`, `uppercase`, `tracking-wider`. Numbers always `tabular-nums`.

**The typographic rule that carries the product:** prose is sans, evidence is mono. If a number is on the page, it is in JetBrains Mono. That is why a receipt row reads as a receipt and not as marketing.

## 3. Surface language

| Token | Value |
|---|---|
| Radii | `--radius-xs 6px`, `sm 10px`, `md 16px`, `lg 24px` |
| Hairline | `box-shadow: 0 0 0 1px var(--mk-line)`; hover `--mk-line-strong` |
| Card | `rounded-md bg-surface hairline`, padding 5–6 |
| Grid seams | `gap-px bg-line` with `bg-raised` cells — the stat strip is drawn by the gaps, not by borders |
| Easing | `--ease-smooth: cubic-bezier(.22,1,.36,1)` |
| Max width | `max-w-6xl`, page padding `px-5` |
| Touch targets | `min-h-10` / `min-h-11` on every control |

Borders are hairlines, never 1px solid grey boxes. That single choice is most of why the page reads as a ledger.

## 4. Motion (all four, exactly as the reference defines them)

```css
@keyframes tape-in    { 0% {opacity:0; transform:translateX(-8px)} to {opacity:1; transform:translateX(0)} }
@keyframes pulse-live { 0%,to {opacity:1} 50% {opacity:.35} }
@keyframes marquee    { 0% {transform:translateX(0)} to {transform:translateX(-50%)} }
@keyframes scan       { 0% {transform:translateY(-100%)} to {transform:translateY(400%)} }

.tape-in     { animation: tape-in .22s var(--ease-smooth) both }
.marquee-track { animation: marquee 28s linear infinite }
.scan-bar    { animation: scan 3.2s linear infinite }
```

Reveal cadence: receipt rows appear one at a time on a 700ms interval, `tape-in` each. `prefers-reduced-motion: reduce` shows all rows immediately and stops the marquee, scan bar, and canvas — this is already handled in the reference and must be preserved.

**Hero canvas** (ambient, `opacity-70`, masked): 8 horizontal hairlines at 6% alpha, a mid-line at 12%, an 80-point sine-driven price trace at 45%, and 40 drifting dots at 55% coloured `fg` / `allow` / `refuse` with roughly 12% of them refusals. It reads as a tape without ever claiming to be data. It is decorative and must never be mistaken for the book — hence it never carries axis labels or numbers.

## 5. Page inventory

| Route | Job | Gate B |
|---|---|---|
| `/` | landing — sell the book, close with the lock | B1, B15 |
| `/book` | the desk: counters, circuit, receipts, owner verbs | B1 B2 B6 B8 B9 B10 |
| `/receipts` | full feed, filterable by reason | optional |
| `/paper` | the daily paper log, rendered from `/v1/paper` | B12 surface |

Header on every page: wordmark (M glyph + `ARKOV`, `tracking-[0.12em]`), nav `Home / Desk / Receipts / Paper`, a pulsing `allow` dot with `DEVNET` in mono, theme toggle persisted at `localStorage["markov-theme"]` with a no-flash inline script.

## 6. `/book` — the page Gate B grades

```
┌──────────────────────────────────────────────────────────┐
│ BOOK_ONE ●          devnet · marked PnL, not a promised rate│
├────────────┬────────────┬────────────┬────────────────────┤
│ net delta  │ gross      │ funding 7d │ refusals           │
│ $12.40 ±20 │ $61 cap100 │ 0  USDC-d  │ 3   24h            │
├────────────┴────────────┴────────────┴────────────────────┤
│ circuit live            skip = default      mark 2s old   │
├──────────────────────────────────────────────────────────┤
│ 14:02  hedge SOL-PERP    +12.0   allowed         ↗ sig    │
│ 14:04  increase SOL      +40.0   OverTxCap       ↗ sig    │
│ 14:11  skip               —      skip                     │
├──────────────────────────────────────────────────────────┤
│ [ Fund ]  [ Pause ]  [ Revoke ]  [ Withdraw ]             │
│ Withdraw stays on in every state.                         │
└──────────────────────────────────────────────────────────┘
```

**Non-negotiables on this page:**

1. `Withdraw` is never `disabled`. Not while loading, not while a tx is pending, not when revoked, not when the API is down. If state is unknown, the button still renders and still builds a withdraw transaction. There is a Playwright test named `withdraw-enabled-in-every-state` and it is B10.
2. Every receipt row links to the explorer with `?cluster=devnet`.
3. Refusal rows show the `BlockReason` verbatim in mono, in `refuse`. No friendly rewording — `OverTxCap` is the product.
4. Off-chain-enforced numbers (net delta, gross, daily-loss halt) carry a small `off-chain guard` marker. Chain-enforced ones do not.
5. Degraded state is loud, not hidden: if `chainReady` is false, a banner names the failing term and the counters grey out with their last-updated timestamp.
6. Empty states are honest: zero refusals renders as `no refusals in this window` in `subtle`, never as a green tick or a badge.
7. No APY. No projected return. No sparkline that implies a trend the data does not support.

**Owner verb flows.** Fund → amount input in USDC-d, shows the mandate PDA the coins land in, then the signature. Pause / Revoke → confirm dialog naming the consequence in plain words ("the agent's next intent will be refused; withdraw stays on"). Withdraw → max button, then signature, then the owner's new balance.

**Polling.** 5s while the tab is visible, paused when hidden, backing off to 30s after three consecutive errors. Ten seconds end-to-end is the B9 budget and a 5s poll leaves headroom.

## 7. Copy rules (B15 lives here)

Allowed vocabulary: *devnet, marked, unaudited, book, mandate, receipt, refusal, operator, owner, withdraw, bounded, skip*.
Banned in shipped copy: `APY`, `APR`, `annualized`, `yield` as a promise, `guaranteed`, `risk-free`, `audited`, `secure`, `mainnet`, any venue brand, `all eleven refusals`, and any sentence that implies the delta band is enforced on chain in v0.

Every page carries the label `devnet · marked PnL, not a promised rate` above the fold. The FAQ answer to "What's the APY?" stays exactly as written: there isn't one on this page.

## 8. Accessibility floor

Keyboard reachable owner verbs with visible focus rings; `aria-live="polite"` on the receipt list so a new row is announced; contrast checked for `muted` and `subtle` on both themes; `prefers-reduced-motion` honoured across canvas, marquee, scan bar, and tape reveal; the theme toggle is a real `<button>` with an `aria-label` that names the destination state; the receipt list is an `<ol>`, the stat strip a `<dl>`, and the FAQ native `<details>` so it works before hydration.

## 9. Performance floor

Two font families, one stylesheet, no icon library beyond the handful of lucide glyphs actually used, no chart library in Gate B. SSR the shell so the label and the receipts render without JS. Lighthouse ≥ 95 on performance and accessibility for `/` and `/book` on a mobile profile, checked in CI on the preview URL.
