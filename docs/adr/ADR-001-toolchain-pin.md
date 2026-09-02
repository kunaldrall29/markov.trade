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

Note on pack v0.2's "recommended Agave 3.1.10": that is the Solana release anchor-cli 1.1.2 installs by default (observed today), not the current Agave stable (4.2.2) and not what devnet runs (4.3.0-beta.3). Whether 3.1.10's `cargo-build-sbf` output runs on a 4.3 devnet is not in question (SBF programs are forward-compatible), but client crates (`solana-client`, LiteSVM) resolve against 4.x, so option B should pin Agave 4.2.2 explicitly rather than accept Anchor's default.

## Recommendation

**B**, coupled to D3 = successor program (ADR-004). If D3 chooses an in-place upgrade of `5o8E…`, then **A** for the program crate only, with LiteSVM compatibility proven by a build in the P01 pre-flight before anything else is written.

Either way P01 must prove, with output pasted into FACTS: `anchor build` of an empty program, `cargo test` with the chosen LiteSVM in the same workspace, and `pyth-solana-receiver-sdk` compiling against the pin (or the ADR-003 fallback recorded).

## Consequences

`Anchor.toml [toolchain]`, `rust-toolchain.toml` and CI agree on one set. A developer machine that differs is a bug report. `docs/FACTS.md` rows `ANCHOR_VERSION`, `LITESVM_VERSION`, `RUST_VERSION` move from PENDING to values only after the P01 build passes.

**Observed 2026-09-01, applies to both options:** anchor-cli manages the Solana release itself. Every `anchor` invocation re-initialises `~/.local/share/solana/install/active_release` to that Anchor version's default (0.31.1 → Solana 2.1.0, 1.1.2 → 3.1.10) unless `Anchor.toml` carries `[toolchain] solana_version = "<pin>"`. So the pin is not optional, `anchor` must never be run outside the project in CI, and `solana --version` in FACTS must be recorded from inside the project after the pin exists. Both 0.31.1 and 1.1.2 are installed here (`avm use` switches). The active Solana release was re-initialised to 4.2.2 at the end of Session 0 but is volatile until the pin exists (the verification pass found it at 2.1.0 after its own `anchor --version` calls).

## Verification pass (2026-09-01, three independent judges)

- **Host `cargo check` was run for both option graphs.** B (`anchor-lang 1.1.2` + `litesvm 0.16.0` + `pyth-solana-receiver-sdk 2.0.0`) resolves to one graph (single `solana-address 2.6.1`; `solana-pubkey 3.x` is a shim over `solana-address 2.x`, so no Pubkey clash) and **passes**. A (`anchor-lang 0.31.1` + `litesvm 0.6.1` + `pyth-solana-receiver-sdk 1.0.1`) resolves but **fails** (E0277 `PriceFeedMessage: BorshSerialize` — `pythnet-sdk 2.3.1`'s open `anchor-lang >=0.28` bound pulls 1.1.2/borsh 1 next to 0.31.1/borsh 0.10) unless the lockfile is forced with `cargo update --precise`, which any later `cargo update` undoes. "A for the program crate only" is therefore not a clean fallback.
- Anchor 1.1.2's own project template pins `litesvm = "0.10.0"`; `anchor-litesvm 0.4.0` pins `litesvm ^0.11`; Anchor 1.x CI builds with Solana 3.1.10. The LiteSVM pin for B is whichever of 0.10–0.16 the P01 SBF build proves, not "latest".
- `@codama/nodes-from-anchor` converts IDLs from both 0.31 and 1.x (`anchor-lang-idl-spec` 0.1.x on both), so the Codama/Kit argument does not differentiate B; the on-chain IDL (`anchor idl init`) matters more for the web seat than the pin.
- Truth seat: the chosen pin should produce a **verifiable build** (`anchor build --verifiable` / `solana-verify`) so the successor's on-chain hash can be published in FACTS and `/v1/facts`; today nothing ties the deployed bytecode to a source commit except `declare_id!`.
- Both branches still lack an SBF build + LiteSVM test run. **Decision date: 2 Sep**, so P01's first act is the build.
