# ADR-011 — The control-plane program is a successor, built beside `markov_mandate`, deployed only from a networked box

Status: **Accepted 2026-09-15** · Date: 2026-09-15 · Seat: Protocol · Inputs: `01_Program_Build_Prompt.md`, blueprint v3.1 Part II and Annex P §31–33 · Relates to: ADR-004 (successor program, Gate B), ADR-008/009 (venue refusal as data; signer propagation)

## Context

Gate B's `markov_mandate` (`25CdYaZe…`) is one owner's vault plus a propose-only operator and a fourteen-gate ladder around a CPI into a mock venue. The blueprint's control plane needs what it does not have: an account object separate from any vault, versioned mandates with a hash, scoped and capped permissions for MCP clients, keepers and API keys, receipts keyed by request id with the checks the decision was made on, a daily ledger, an Invest mandate with a spend cap the keeper cannot exceed, and enforcement of venue adjacency through the instructions sysvar rather than a CPI the program signs.

Retrofitting that into `markov_mandate` would renumber nothing (its `BlockReason` is append-only) but would change every account layout and the meaning of its instructions. Its receipts on devnet are Gate B evidence and must stay decodable.

## Decision

1. **A new program, `programs/markov`**, to the `01` prompt: `GlobalConfig`, `MarkovAccount`, `Mandate`, `InvestMandate`, `Permission`, `ActionReceipt`, `DailyLedger`; the instruction set in `01` §3 plus the admin setters the prompt implies (`set_venue_program`, `set_invest_programs`) and `link_venue`, `pause_invest`. Reason codes are the prompt's u16 table; rule ids are in `state::rule`. `markov_mandate` is untouched and keeps serving the Book One desk.
2. **The program re-validates from the chain's limits, never the caller's.** `checks::evaluate_trade` and `evaluate_invest` take the caller's observed values, replace every enforced limit with the mandate's or the config's, recompute `pass`, and refuse an `Allow` that fails any enforced rule. Rules the program cannot know (slippage, single-asset cap) are recorded as supplied and may still reject.
3. **Adjacency is a hard requirement.** An `Allow` needs the configured venue program at *i+1* (and never this program); a refusal or skip may not be followed by any configured venue, collect or swap program; `invest_execute` needs collect at *i+1* and swap at *i+2*. The zero pubkey (the system program) means "unset" in the config, so the system program can never be a venue.
4. **Reserve proof is a token account, not a number.** `invest_execute` reads the owner's USDC account and checks the floor against its balance minus the pull.
5. **Deviations from the prompt, stated:** `InvestMandate.max_reference_deviation_bps` is added (Reference Safe mode needs a bound); `set_mandate` and `set_invest_mandate` emit the `MandateSet` receipt as an event without a receipt PDA (no request id, no rent); `record_decision` takes the `day_epoch` it charges and refuses one that is not the clock's day; `pause`/`unpause` do not emit receipts (they change `MarkovAccount.status`, which every later receipt records through reason 11).
6. **Not deployed from here.** The id in `declare_id!` is a placeholder. Deployment needs a funded deployer, the Squads multisig and a box that can reach devnet, none of which this session had. Until deployed, no surface reads this program; `packages/sdk` and the Terminal keep targeting `markov_mandate`.

## Alternatives rejected

- **Upgrade `markov_mandate` in place.** Every account changes shape; Gate B receipts and the desk would need a migration for no product gain.
- **Wait for the deploy path before writing the program.** The prompt asks for LiteSVM and property tests first; those run here, and they found real bugs (the zero-pubkey venue collision, the day-epoch check) before any deploy.

## Verification (2026-09-15)

- SBF build with platform-tools v1.52 (`cargo +solana build --target sbpf-solana-solana`, no Anchor CLI on the box), stripped with `llvm-objcopy`.
- `cargo test -p markov`: 8 host tests (unit + proptest over random mandates and observations: an `Allow` verdict never carries a violated enforced check) and 22 LiteSVM cases against the built binary — invariants 1–8 of `01` §4, including missing, misplaced, wrong-program and self adjacency, paused/global-paused behaviour for owner and actor, scope and cap refusals as receipts, daily cap roll, expiry, revocation, replay, finalize once, close after 30 days, Invest month roll and ceiling, reserve floor from the token account, skip cannot ride with collect.
- `cargo clippy -D warnings`, `cargo fmt --check`, `cargo deny check` (see FACTS) clean.
- IDL assembled from the `idl-build` harness at `docs/idl/markov-qKxcmWWh.json` (placeholder id).
- Not verified: anything on devnet.
