# P12 — Landing page
Seat: Surfaces · Window: 21–26 Sep (or any spare slot; it is not on the critical path) · Inherits `P00-conventions.md`

## Goal
Port the existing design at `markovhq.grok.me` into the repo, unchanged in look and feel, with the copy discipline that makes **B15** pass by construction.

## Pre-flight
1. Read `docs/13-FRONTEND-SPEC.md` §§1–4. The tokens, type, and four keyframes are the design contract; do not re-derive or "improve" them.
2. Confirm Tailwind v4 is the version in `apps/web` (the reference uses v4 syntax such as `bg-linear-to-b` and `--color-*` theme vars). If the repo is on v3, STOP and report — porting classes across a major is a decision, not a detail.
3. A working reference build is committed at `landing/index.html` in this pack. Use it as the visual source of truth.

## Deliverables
`apps/web/app/routes/index.tsx` plus components, reproducing, in order:

1. Sticky header card: `bg-surface/80 backdrop-blur-md hairline rounded-lg`, M-glyph + `ARKOV` wordmark, nav, pulsing `DEVNET` dot, theme toggle.
2. Hero: eyebrow pill `House book · Solana perps · unaudited`; `Park USDC. / The book works. / You leave.` at `text-5xl → sm:text-7xl leading-[0.95]`; the sub-paragraph; two CTAs (`Open the desk` primary, `Watch a refusal` ghost); ambient canvas at `opacity-70` with the mask, drawn per `13-FRONTEND-SPEC` §4.
3. Live blotter card: scan bar, `BOOK_ONE` + `marked · not a rate`, four stat tiles, circuit line, receipts revealing one per 700ms with `tape-in`, and the four owner-verb buttons.
4. Marquee strip: the tape, tripled, `28s linear infinite`, bordered `border-y border-line bg-raised`.
5. Three cards: Job / Wedge / Proof.
6. `How the desk is allowed to work` → `An LLM never signs.` with 01 Core / 02 Veto / 03 Sidecar / 04 Proof.
7. `Not that` → HLP / JLP / Marketplace / Token comparison.
8. `Owner verbs` → Fund / Pause / Revoke / Withdraw.
9. FAQ as native `<details>` with the `+` that rotates 45° on open.
10. Closing panel: inverted `bg-fg text-bg`, `Keep the pile.`
11. Footer: wordmark, `Keep the pile.`, the three-line principle, nav.

## Hard constraints
- Copy is exactly the reference copy. Any new sentence must pass `copy-grep` and must not name a venue, promise a rate, or claim an audit.
- The numbered markers 01–04 stay only because that section is a real sequence (propose → veto → features → proof). Do not add numbering anywhere else.
- `prefers-reduced-motion: reduce` stops the canvas, marquee, scan bar, and tape reveal, and reveals all rows immediately.
- The blotter on the landing page is **illustrative**. If it ever shows live numbers, it must carry the same `source` labelling as `/book` — otherwise it stays static and reads as a sample.
- Two font families, no icon library beyond the glyphs actually used, no chart library.

## Acceptance
- Visual diff against `landing/index.html` at 390px, 768px, 1440px, both themes.
- `copy-grep` clean on the built HTML → **B15**.
- Lighthouse ≥95 performance and accessibility, mobile profile.
- Keyboard tour: every control reachable, focus visible, FAQ operable before hydration.

## Evidence
Screenshots at three widths in both themes, copy-grep output, Lighthouse scores.
