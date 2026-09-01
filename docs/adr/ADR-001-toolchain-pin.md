# ADR-001 — Toolchain pin (decision D0)

Status: **Proposed — needs Kunal** · Date: 2026-09-01 · Seat: Protocol · Blocks: P01, P02, P04, P05

## Context

The pack assumes Anchor v1.0.x. On 2026-09-01 the facts are:

| Fact | Value | Source |
|---|---|---|
| Both deployed programs (`5o8E…`, `CT2n…`) | Anchor **0.31.1**, host rustc 1.85.0, Solana CLI 2.1.21 / 4.2.1 at build time | `Anchor.toml` in both repos; their FACTS |
| Anchor current stable | **1.1.2** (2026-06-26); 2.0.0-rc.1 (2026-08-12) | crates.io, GitHub releases |
| Agave stable | **4.2.2** (2026-08-28); devnet runs 4.3.0-beta.3 | GitHub releases; `getVersion` |
| Installed on this workstation today | anchor-cli 0.31.1 (avm 1.1.2), solana-cli 4.2.2, rustc 1.97.1 | `docs/FACTS.md` |
| `pyth-solana-receiver-sdk` | 2.0.0 needs anchor ^1.0.2; 1.0.1 needs ^0.31.1 | crates.io deps |
| LiteSVM 0.16.0 | depends on `solana-*` / `agave-*` 4.2–4.3 crates | crates.io deps |
| anchor-lang 0.31.1 / 1.1.2 | depend on `solana-program ^2` / `solana-*` ^3 respectively | crates.io deps |

`Migration<From, To>` (the pack's answer to a missing reserve, D3) exists only in Anchor v1.

## Options

**A. Pin Anchor 0.31.1 + Agave 4.2.2.** Matches the deployed bytecode and lets an in-place upgrade of `5o8E…` reuse its build. Costs: no `Migration`, no Anchor-v1 LiteSVM default, and a probable crate-major clash between anchor 0.31.1 (`solana-program` v2) and LiteSVM 0.16 (`solana-*` v4) in one test crate. Older LiteSVM (0.6.x, solana 2.x era) would be needed and is unverified.

**B. Pin Anchor 1.1.2 + Agave 4.2.2.** Current stable; `pyth-solana-receiver-sdk` 2.0.0 fits; `Migration` available; Codama/Kit tooling targets the v1 IDL. Costs: the Gate B program is a port of the 0.31.1 source, and reproducing `5o8E…`'s exact account layout is only needed if D3 chooses in-place upgrade.

**C. Anchor 2.0.0-rc.1.** Rejected: release candidate, 20 days old, no reason to carry that risk inside Gate B.

## Recommendation

**B**, coupled to D3 = successor program (ADR-004). If D3 chooses an in-place upgrade of `5o8E…`, then **A** for the program crate only, with LiteSVM compatibility proven by a build in the P01 pre-flight before anything else is written.

Either way P01 must prove, with output pasted into FACTS: `anchor build` of an empty program, `cargo test` with the chosen LiteSVM in the same workspace, and `pyth-solana-receiver-sdk` compiling against the pin (or the ADR-003 fallback recorded).

## Consequences

`Anchor.toml [toolchain]`, `rust-toolchain.toml` and CI agree on one set. A developer machine that differs is a bug report. `docs/FACTS.md` rows `ANCHOR_VERSION`, `LITESVM_VERSION`, `RUST_VERSION` move from PENDING to values only after the P01 build passes.

**Observed 2026-09-01, applies to both options:** anchor-cli manages the Solana release itself. Every `anchor` invocation re-initialises `~/.local/share/solana/install/active_release` to that Anchor version's default (0.31.1 → Solana 2.1.0, 1.1.2 → 3.1.10) unless `Anchor.toml` carries `[toolchain] solana_version = "<pin>"`. So the pin is not optional, `anchor` must never be run outside the project in CI, and `solana --version` in FACTS must be recorded from inside the project after the pin exists. Both 0.31.1 and 1.1.2 are installed here (`avm use` switches); 4.2.2 is the active Solana release again.
