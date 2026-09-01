# P01 — Repo bootstrap
Seat: Protocol · Window: 31 Aug – 2 Sep · Inherits `P00-conventions.md`

## Goal
A monorepo that builds, lints, tests, and deploys nothing yet — but where every later prompt has a place to put its code and a CI job that will catch it failing.

## Pre-flight (STOP and report if any fails)
1. `anchor --version`, `solana --version`, `rustc --version`, `node -v`, `pnpm -v` all print, and the Anchor and Agave versions are compatible with each other. Record all five in FACTS.
2. `solana config get` shows devnet, and `solana balance` on a fresh keypair funds from the faucet.
3. The **existing deployed `markov-mandate` program** can be found: `solana program show <id>` returns, and you can dump its IDL. If the program ID is unknown, STOP — that ID is a Week-0 input, not something to invent.
4. `anchor init` scaffolding builds an empty program with the pinned toolchain.

## Deliverables
- Tree exactly as in `docs/09-REPO-STRUCTURE.md` §1, with empty crates that compile (`lib.rs` with a doc comment is fine).
- `Anchor.toml` with a `[toolchain]` block pinning anchor and solana versions.
- `rust-toolchain.toml`, `Cargo.toml` workspace, `pnpm-workspace.yaml`, `turbo.json`.
- `deny.toml` banning, for `markov-guard` specifically: `tokio`, `reqwest`, `solana-client`, and any crate that reads a clock.
- `.env.example` with every variable from `docs/09-REPO-STRUCTURE.md` §4, values blank.
- `docs/FACTS.md` created from `docs/17-FACTS-TEMPLATE.md`, with the toolchain and program rows filled and everything else `PENDING`.
- `docs/{SECURITY.md,STATUS.md,SESSION_LOG.md,BACKLOG.md}` stubs; `docs/adr/` with ADR-001 recording the Rust-services decision from `07-TECH-ARCHITECTURE` §4.
- CI: `rust.yml`, `web.yml`, `truth.yml` as described in `docs/15-TESTING-CI-OBSERVABILITY.md` §3. They may have almost nothing to run; they must be green.
- `scripts/copy-grep.sh` implemented now (it costs ten minutes and it enforces B15 from day one).

## Acceptance
- `cargo build --workspace`, `cargo clippy --workspace -- -D warnings`, `cargo deny check` all pass locally and in CI.
- `pnpm -r build` passes.
- `bash scripts/copy-grep.sh` exits 0 on an empty build and exits 1 when you temporarily add the string `12% APY` to a test fixture. **Prove both.**
- `docs/FACTS.md` has no fabricated values; anything unverified says `PENDING`.

## Evidence to record
Toolchain versions, program ID, IDL sha256, and the copy-grep pass/fail demonstration.
