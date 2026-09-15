# `markov` — the control-plane program

Built to `01_Program_Build_Prompt.md`. One program holds the durable truth for every Markov product: the **account**, its **versioned mandate** (hard rules), its **permissions** (who may ask for what) and the **receipts** of every decision. Venue execution happens in the adjacent instruction; the program checks, through the instructions sysvar, that an `Allow` is followed by the configured venue program and that a refusal is followed by nothing.

Status: **built and tested in LiteSVM; not deployed.** The `declare_id!` is a placeholder; the deploying box generates the keypair (`anchor keys sync`), records the id in `docs/FACTS.md`, and deploys under the Squads multisig per `01` §8. Nothing in the Terminal reads this program yet (`packages/sdk` targets the deployed `markov_mandate`).

## Accounts (PDAs)

| Account | Seeds | Holds |
|---|---|---|
| `GlobalConfig` | `["config"]` | admin, `paused`, caps (per-account gross notional, per-position leverage, global daily Invest spend, per-execution bounds, freshness slots), venue program ids by `venue_id`, the Invest `collect` and swap program ids, the USDC mint |
| `MarkovAccount` | `["account", owner]` | owner, execution mode, active mandate and invest-mandate versions, status, replay domain, linked venues, reserved[64] |
| `Mandate` | `["mandate", account, version_le]` | hash, max leverage, max notional, min safety buffer, max daily loss, approved markets (≤32), tier caps, allowed venues bitmap |
| `InvestMandate` | `["invest", account, version_le]` | hash, allowlist (≤16), per-period budget, period, monthly ceiling, reserve floor, max cost, max reference deviation, single-asset cap, execution mode, window, days, paused, `spent_this_month_usd`, `month_epoch` |
| `Permission` | `["perm", account, actor]` | kind, scopes bitmap, per-action and daily caps, spent today, day epoch, expiry slot, revoked |
| `ActionReceipt` | `["receipt", account, request_id_le]` | actor, kind, decision, reason code, versions, data slot, venue, market, checks (≤12), route hash, tx hint, pre/post state hashes, finalized |
| `DailyLedger` | `["day", account, day_epoch_le]` | realized loss, gross notional, action count |

Money is micro-USD `u64`; observed values on checks are `i64`. Reason codes and rule ids are in `src/state.rs` and never renumber.

## Decision flow (`record_decision`, instruction *i*)

1. Actor gate: the owner passes; anyone else needs a live `Permission` with the scope for the kind and headroom on both caps (invariant 4). A scope or cap failure becomes a `Reject` with reason 8/9.
2. Pauses: global pause rejects actor flows (12); an account pause rejects risk-increasing kinds (11). Reduce and close still pass a paused account.
3. `checks::evaluate_trade` recomputes every enforced rule with the **chain's** limits (venue, market, freshness, leverage, notional, safety buffer, daily loss including today's ledger). A supplied `Allow` that fails any enforced rule is refused outright (invariant 1). Off-chain rules (slippage, …) are recorded as supplied and may still reject.
4. Adjacency: an `Allow` requires instruction *i+1* from the configured program for `venue_id` and never this program (invariant 2); anything else requires that no configured venue, collect or swap program follows.
5. Only an `Allow` changes the ledger and the actor's daily spend; every outcome writes the receipt PDA and emits `ReceiptEmitted`.

`invest_execute` is the keeper's Allow path: the reserve floor is read from the owner's USDC token account (never a supplied number), the month rolls once and only grows (invariant 5), and the transaction must carry the delegation `collect` at *i+1* and the swap at *i+2*. `record_skip` is the keeper's refusal path and may not ride with either.

## Tests

- `cargo test -p markov --lib` — pure checks: unit tests and a proptest that no `Allow` verdict ever contains a violated enforced rule.
- `cargo test -p markov --test markov` — 22 LiteSVM cases against the built `.so`, one or more per invariant, including the instruction-adjacency cases (missing, misplaced, wrong program, this program). The venue and `collect` legs are `programs/test-noop` (test only, never deployed); the swap leg is the compute-budget builtin.

Build without the Anchor CLI (what this repo's session did): link the platform-tools Rust as a rustup toolchain and run `cargo +solana build --release --target sbpf-solana-solana -p markov --lib`, then `llvm-objcopy --strip-all` into `target/deploy/markov.so`. With the CLI, `anchor build` does the same.
