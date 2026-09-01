# ADR-002 — Hosting split for `markov.trade` (decision D1)

Status: **Proposed — needs Kunal** · Date: 2026-09-01 · Seat: Surfaces · Blocks: P11, P12, P14, P15, DNS

## Context

- `markov.trade` is registered and delegated to Cloudflare; it has **no DNS records** (verified 2026-09-01). Nothing is served at the apex, `www`, or `api`.
- The landing page is Kunal's (`landing/index.html`, a single static file) and is an input, not a task.
- The pack expects `markov.trade/book`, `/receipts`, `/paper` (TanStack Start, `docs/13` §5) and `api.markov.trade` (Rust `data-api` on Railway).
- The design source `markovhq.grok.me` is already a TanStack Start app whose `/book` route is a static sample. Porting the landing into the same app is P12's job and is off the critical path.
- The tape (B14) must be recorded against production hosts with no localhost and a visible URL bar.

## Options

**A. One origin.** One TanStack Start deploy on Vercel serves `/` (the ported landing, or the static file mounted as the index route until P12 lands), `/book`, `/receipts`, `/paper`. `api.markov.trade` is a CNAME to the Railway `data-api` service. CORS on the API allows exactly `https://markov.trade`.
- Pro: one header, one theme toggle, one `copy-grep` target, one Lighthouse run, relative links in the tape, matches `docs/13` §5 verbatim.
- Con: the landing is no longer "just a file"; Kunal's edits to `index.html` must be re-ported (or the file is served verbatim by the app as a static route, which keeps the port trivial).

**B. Split origin.** Apex serves the static landing (Cloudflare Pages or Vercel static); `/book` lives on a second host (for example `book.markov.trade` or a Vercel rewrite from `markov.trade/book` to the app). API as in A.
- Pro: Kunal ships the landing independently, zero coupling.
- Con: a rewrite at the apex still needs the apex to be a Vercel/Cloudflare project; the header/nav must be duplicated and kept identical; two `copy-grep` targets; cookie-less theme persistence differs per origin; the tape shows a hostname change unless the rewrite is used.

## New input (2026-09-01, after the ADR was drafted)

Kunal's `kunaldrall29/markov-landing-new` is not a static file; it is the TanStack Start app with `/`, `/book`, `/receipts` and `/paper` already sharing one shell, header and theme toggle, now imported at `apps/web`. The Grok project memory records the design lock: "`/` and `/book` should look like the same desk." Option B would mean splitting an app that already exists as one origin.

## Recommendation

**A**, using `apps/web` as-is for `/` (the port in P12 is now a diff against the imported app, not a rebuild from the static file). Concretely: Vercel project for `apps/web` with domains `markov.trade` and `www.markov.trade`; Railway custom domain `api.markov.trade` on `data-api`; Cloudflare DNS: apex + `www` → Vercel, `api` → Railway (DNS-only, no Cloudflare proxy, so Vercel/Railway TLS works). CORS allowlist `https://markov.trade` only.

## What changes if Kunal picks B

`apps/web` router gets a `basepath`; the header component is duplicated into `landing/index.html` by hand; `scripts/copy-grep.sh` runs on both builds; `gate-b-verify.sh` opens two origins; `docs/13` §5 "header on every page" is satisfied by copy, not by code.

## Consequences

DNS is not touched until this ADR is Accepted. `BOOK_URL`, `RECEIPTS_API_URL`, `HEALTH_URL` stay PENDING in FACTS until the hosts answer logged-out over HTTPS.
