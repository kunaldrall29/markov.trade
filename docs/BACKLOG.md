# BACKLOG

Things noticed and deliberately not done, with one line of why. Each entry says what would have to be true to start. Items move out only via a prompt that owns them.

## Noticed in Session 0 (2026-09-01)

| Item | Why not now | To start |
|---|---|---|
| `docs/10-PROGRAM-SPEC.md` §1/§2/§4 and `docs/11` §4, `docs/12` §2 use `NotOperator` / `VenueNotAllowed` and number `Paused`=0; the chain says `Unauthorized` / `ProgramNotAllowed` and `OverTxCap`=0 | Session 0 is no-code and the spec edit is ADR-004's consequence | ADR-004 accepted |
| `docs/08` §1 and §3 quote stale versions (Anchor 1.0.x; Pyth SDK "up to 0.31.1") and an unauthenticated Hermes | same; recorded in FACTS with today's values | ADR-001 / ADR-003 accepted |
| Existing Railway `api` service holds `EMERGENCY_KEY_JSON` and signs house-operator actions from `keys/*.json` on disk; violates `docs/14` §2 for the old stack | old stack is not Gate B's; must not be inherited | P07/P13 create fresh keys in separate services |
| Existing TS indexer stalled at lag ≈ 1.2M slots with `chainReady:false`; public feed shows stale counts | not Gate B code; restarting it changes a live property Kunal did not ask to touch | Kunal says so, or ADR-005 picks reuse |
| Vercel project `markov-float` is git-linked to `kunal-drall/markov` (the old Base product) | outside this repo; a wrong autodeploy source is a footgun for D1 | D1 accepted; unlink or delete the project |
| `landing/index.html` has no OG image, favicon, manifest (DESIGN-NOTES "not carried over") | landing is Kunal's; B1 first impression needs them before the tape | P12 or Kunal, before P15 |
| `landing/index.html` FAQ says "Marks are marked PnL" — fine, but the header's `DEVNET` pill and the `/book` link 404 until P11 | by design per DESIGN-NOTES | P11 |
| Faucet rate-limits this IP; P01 pre-flight #2 needs SOL | provider-side | retry later, web faucet, or a provider faucet |
| No RPC provider account (Helius/Triton/QuickNode class) with WebSocket `logsSubscribe` | account opening is Kunal's | before P07/P09 |
| LiteSVM 0.16 depends on `solana-*` v4 crates; anchor 0.31.1 on v2, anchor 1.1.2 on v3 — one workspace may not resolve | needs a build, which is P01 pre-flight | P01 |
| `@coral-xyz/anchor` npm is frozen at 0.32.1; Anchor v1's TS client is `@anchor-lang/core` | only matters if something other than Codama generates the client | P11 pre-flight |
| The prior `5o8E…` history (four-beat, wallet demo, 11-reason redteam) is real devnet evidence that `docs/16`'s proof chain could cite as prior art | not Gate B proofs (different spec, no `BOOK_ONE`); would blur B4–B8 | never as B-proofs; maybe a "history" note on `/receipts` after Gate B |
| `MarkovFyi/*` public repos (MIT) and `kunaldrall29/markov` (Apache-2.0) disagree on license and on which program is canonical | governance, not Gate B | Kunal |
| Session 0 scratch tools (`dump-idl.mjs`, `disc.py`, `hist.py`, `bin-enum.py`) live in the session scratchpad | not part of the repo tree per `docs/09`; useful for the IDL append-only check | P14 may port `disc.py` logic into `scripts/` |
| **`scripts/copy-grep.sh` is vacuous on single-line SSR HTML**: `sed 's\|<script[^>]*>.*</script>\|\|g'` is greedy, so on `apps/web`'s rendered pages it deletes everything between the head's theme-boot script and the trailing module script — one word survives, "clean" proves nothing. Fix: strip with `perl -0pe 's/<script\b[^>]*>.*?<\/script>//gs; s/<style\b[^>]*>.*?<\/style>//gs'` (or a Python stripper), then keep the existing rules; add a self-test that asserts a rendered page still has >100 visible words after stripping | script is a pack artifact; Session 0 is no-code | P14 (owns copy-grep), before `truth.yml` is trusted |
| `apps/web` carries Grok builder scaffolding (better-auth + PGLite + `migrations/auth`, `server/middleware/grok-pwa.ts` head injection, `public/__grok/`, preview-host bridge, multiplayer p2p, Radix UI set). Harmless with `VITE_AUTH_ENABLED=false`, but the Vercel deployer sets it `true` (per `scripts/with-app-env.mjs`), which boots better-auth on PGLite inside the function | Kunal's app, imported verbatim | P12 decides what to strip; P11 must not build `/book` on top of the auth/db layer |
| `apps/web/public/videos/keep-the-pile.mp4` is 30 MB in git | hero film, Kunal's content | LFS or CDN before the repo grows; P12 |
| `/book` in `apps/web` lacks the B1 literal `devnet · marked PnL, not a promised rate` above the fold (has "Sample tape, devnet. Marked PnL, not a promised rate." in the sub-copy) | copy is Kunal's; P11 owns `/book` | P11 |
| `apps/web` pins `lucide-react ^0.510.0` while npm latest is 1.39.0; `nitro` is a dated beta (`3.0.260610-beta`); no `@solana/*` deps yet | upgrades are P11/P12 work with visual diffs | P11/P12 |
| `kunaldrall29/markov-landing-new` is a private repo whose commits are authored "Grok" ("Export from Grok"); if Vercel autodeploys from it, the production landing source is outside this monorepo | hosting is D1 | ADR-002 accepted → choose one source of truth for `apps/web` |
