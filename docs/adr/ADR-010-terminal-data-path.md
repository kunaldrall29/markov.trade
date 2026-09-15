# ADR-010 — How the Terminal reads the chain until the Rust read plane is hosted

Status: **Accepted 2026-09-15** · Date: 2026-09-15 · Seat: Surfaces · Relates to: ADR-002 (one origin), ADR-005 (Rust read plane, parity first)

## Context

`apps/web` shipped as a Grok-built design with a static sample tape on every page. The build conventions forbid mock data anywhere a user can see, and P11 requires `/book` to read live state. ADR-005 decided the indexer and `data-api` are Rust, parity first; both are still scaffolds (`crates/indexer`, `crates/data-api` exit 2 on start), and neither is hosted. The Terminal cannot wait on them and must not invent numbers.

The session that made this decision could not reach any Solana RPC host (egress policy), so nothing below was exercised against devnet from that box; the tests that could run are described in §Verification.

## Decision

1. **The connected owner's state is read from the chain by the browser**, through `@solana/kit` and the generated client in `packages/sdk`: the owner's mandates (`getProgramAccounts` filtered on discriminator and owner), vault and token balances, the mock venue's position and mark, the Pyth `PriceUpdateV2`. No server sits between the owner and their account; the RPC URL is `VITE_RPC_URL` with the FACTS keyless endpoints as defaults and a fallback.
2. **The public feed and the house book's stats are served by same-origin routes** (`/api/v1/receipts`, `/api/v1/book/stats`, `/api/v1/mandates/:address`, `/api/health`) inside the TanStack Start app. They walk `getSignaturesForAddress` + `getTransaction`, decode `emit_cpi` receipts from inner-instruction data by IDL, and cache for a few seconds in memory. They are a **cache in front of the chain**, never a source of truth: every response carries `data_slot`, `source: "chain"` and `env`, money is `{raw, decimals, mint}`, and there is no `apy`, `apr` or `projected_*` field (docs/12 §3). A failed read returns an error; the page shows a degraded banner naming the failing term and greys the counters (docs/13 §6 rule 5).
3. **`chainReady` on `/api/health` is narrower than docs/12 §4** because there is no indexer: it means "the RPC answered `getSlot` within ten seconds". When the Rust read plane is hosted, the same paths move behind it and `chainReady` regains its `lag && ingest && parity` meaning; the wire shapes in `apps/web/src/lib/api-types.ts` are written to docs/12 so the swap is a host change, not a UI change.
4. **Shared types cross the language boundary by test, not by import.** `programs/markov-mandate/examples/ts_fixtures.rs` serialises accounts and receipts with the program crate's own codecs; `packages/sdk` tests decode those bytes, and derive the Gate B mandate, vault and venue position addresses FACTS recorded from devnet. A layout change in Rust fails the SDK tests.
5. **Owner verbs are wallet-signed only.** Wallet Standard wallets (Phantom, Solflare, Backpack, …) and Solana Mobile Wallet Adapter are the connectors; the browser stores the chosen wallet's name and the account address, never key material. `Withdraw` renders enabled in every state, including when the chain cannot be read.

## Alternatives rejected

- **Read everything in the browser, no server routes.** The feed needs one `getTransaction` per signature; keyless devnet endpoints throttle bursts (FACTS `RPC endpoints`), and every visitor would repeat the same walk. The cache is cheap and honest about its slot.
- **A TypeScript indexer in the web app.** Rejected in ADR-005 for the reasons that still hold: the parity job and finalizer are the proof surface for B9 and belong in the Rust workspace with the program's own types.
- **Keep the sample tape until the Rust API is hosted.** A page that shows numbers nobody can check is the thing this project exists not to ship.

## Consequences

- `/book` shows what the chain says, five seconds stale at most while the tab is visible, and says so when it cannot.
- The Rust `data-api` keeps its brief (docs/12) and, when hosted, replaces `apps/web/src/lib/server/chain.server.ts` behind the same wire shapes. `chainReady` then covers ingest and parity.
- The Playwright suite runs against a fixture RPC by default (this environment cannot reach devnet) and against a real endpoint when `E2E_RPC_URL` is set; the CI workflow does both.

## Verification (2026-09-15)

- `packages/sdk`: 20 tests, decoders against program-crate bytes, PDAs against FACTS addresses, FACTS drift check clean.
- `apps/web`: typecheck, lint, 28 Playwright cases on desktop and Pixel-7 profiles against the fixture RPC (withdraw enabled in Active/Paused/Revoked and with the chain down; circuit chip paused/global halt; feed filters; API shapes; wallet connect, silent reconnect, fund/withdraw/pause/revoke/amend/create instruction construction captured from the test wallet).
- Not verified here: any read or write against real devnet. See `docs/SESSION_LOG.md` for what Kunal's box must confirm first.
