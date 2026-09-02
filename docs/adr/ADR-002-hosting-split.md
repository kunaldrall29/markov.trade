# ADR-002 — Hosting split for `markov.trade` (decision D1)

Status: **Accepted 2026-09-02** (Kunal: "for others decide on me"; decided per the Recommendation, see Decision at the end) · Date: 2026-09-01 · Seat: Surfaces · Blocks: P11, P12, P14, P15, DNS

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

## Verification pass (2026-09-01)

- A CORS allowlist of exactly `https://markov.trade` blocks Vercel preview URLs, `www`, and local dev; P11 must develop against either a local `data-api` or an explicit preview-origin list. The existing Railway `data-api` cannot serve `/book` at all: it sends no `Access-Control-Allow-Origin` for `markov.trade`, is `chainReady:false` with a 1.2M-slot lag, indexes `5o8E…` not the successor, and returns 404 for `/v1/book/stats`, `/v1/mandates`, `/v1/paper`, `/v1/facts` — so P11's pre-flight #1 cannot pass until P10 is hosted.
- DNS mechanics: Cloudflare records must be DNS-only (proxy off) for Vercel and Railway TLS; apex A record, `www` CNAME with a `www → apex` redirect so the tape shows one hostname.
- Every Vercel project also answers on `<project>.vercel.app` and preview URLs, and Railway keeps `*.up.railway.app`; these are public origins the incognito pass never opens. Either add deployment protection / redirects on default hosts or list them in FACTS as public.
- The "visible URL bar, no localhost" rule is from `prompts/P15`, not `docs/16` B14 (which says "production hosts").

## Consequences

DNS is not touched until this ADR is Accepted. `BOOK_URL`, `RECEIPTS_API_URL`, `HEALTH_URL` stay PENDING in FACTS until the hosts answer logged-out over HTTPS.

## Decision (2026-09-02)

**Option A, one origin.** `apps/web` serves `/`, `/book`, `/receipts`, `/paper` from one Vercel project on `markov.trade` (+ `www` redirecting to the apex). `api.markov.trade` is a CNAME to the Rust `data-api` on Railway. CORS allowlist: `https://markov.trade`, `https://www.markov.trade`, plus an explicit env list for Vercel preview origins during P11; never `*`. Cloudflare records are DNS-only (proxy off) and are added by Kunal only when a host answers: the apex/`www` targets come from the Vercel project's Domains panel, the `api` target from `railway domain`. Default hosts (`*.vercel.app`, `*.up.railway.app`) get deployment protection or a redirect before the tape. Until then `BOOK_URL`, `RECEIPTS_API_URL`, `HEALTH_URL` stay PENDING.
