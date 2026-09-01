# Landing page — what was lifted, what changed
Source: `markovhq.grok.me`, fetched 31 August 2026 (HTML + `assets/styles-*.css` + `assets/routes-*.js`).

## Lifted verbatim
- **Colour tokens**, both themes, every value (`--mk-*` in `index.html` matches the live stylesheet byte for byte).
- **Type**: Outfit 400/500/600/700 + JetBrains Mono 400/500, same Google Fonts request, same roles (prose sans, evidence mono).
- **Surface language**: hairline via `box-shadow: 0 0 0 1px`, radii 6/10/16/24, `gap-px` seams on the stat strip, `--ease-smooth`.
- **Motion**: all four keyframes at the original durations — `tape-in .22s`, `pulse-live 1.8s`, `marquee 28s`, `scan 3.2s` — plus the 700ms row-reveal cadence.
- **Hero canvas**: 8 hairlines at 6%, mid-line at 12%, 80-point sine trace at 45%, 40 drifting dots at 55% with ~12% refusal-coloured, same masks (vertical on mobile, horizontal from `lg`).
- **Structure and copy**: header card, hero, blotter, marquee, Job/Wedge/Proof, "An LLM never signs", "A book. Not a costume.", "Always able to leave.", FAQ, "Keep the pile." close, footer.
- **Theme persistence**: `localStorage["markov-theme"]` with the same no-flash inline boot script.

## Changed, and why
1. **`Jupiter` → `a pool venue`** in the HLP row. The original reads "We don't pretend Jupiter is that book". It is a disclaimer, not a claim — but B15 bans a named venue in shipped copy, and `copy-grep` cannot tell a disclaimer from a claim. One-word revert if you'd rather keep the original and add a grep exception.
2. **Framework removed.** The live page is TanStack Start + Tailwind v4. This is one static file with hand-written CSS using the same tokens, so it deploys anywhere with no build step. `prompts/P12` ports it back into `apps/web` as components; the CSS variable names are identical, so Tailwind v4 `@theme` picks them up unchanged.
3. **`Watch a refusal` now does something.** In the original it is a button with no handler in the SSR payload. Here it replays the tape and holds on the `OverTxCap` row for 1.2s. It is the page's one claim, so it should be demonstrable in one click.
4. **Blotter labelled as a sample.** Added `sample tape, devnet` under the owner verbs. If the landing blotter is ever wired to `/v1/book/stats`, it must carry the same `source` labelling as `/book` — an unlabelled live-looking number on a marketing page is exactly the failure mode the pack is built to prevent.
5. **Light-mode sun icon added** so the toggle shows the destination state in both themes.
6. **Accessibility floor made explicit**: `aria-live` on the tape, `aria-label` on the toggle that names the destination, native `<details>` FAQ, visible focus rings, reduced-motion killing canvas + marquee + scan + reveal.

## Not carried over
`og.jpg`, favicon, manifest, apple touch icons, and the Grok builder script. Add real assets before the tape is recorded — an OG card is part of B1's first impression.

## Deploy
Static. `vercel deploy landing/` or any host. Nav links point at `/book`, `/receipts`, `/paper`, which exist once `P11` ships; until then they 404 and that is fine on a preview, not on the recorded tape.

## v0.2 note (31 Aug 2026)
Blotter third tile aligned to `13-FRONTEND-SPEC` (`funding 7d`, not `funded`). Sample numbers stay illustrative. `HLP` / `JLP` comparison labels stay; venue brands stay out of shipped copy.
