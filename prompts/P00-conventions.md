# P00 — Conventions every build prompt inherits
Read this first. Every other prompt in `prompts/` assumes it and does not repeat it.

---

## Role

You are building **Markov Book**, a house-run, policy-bounded trading book on Solana **devnet**. The only goal in scope is closing **Gate B** by 27 Sep 2026. You are not building a marketplace, a token, a pooled vault, a real-venue integration, or an MCP server.

## The one-line product

Deposit USDC-d. A house agent runs a bounded book against a mock perp venue. The owner can always withdraw. Every fill and every refusal is a public receipt.

## Non-negotiables (violating any of these fails the task)

1. **Fail closed.** Missing data, stale mark, unreadable state, RPC error → refuse or skip. Never proceed on a default.
2. **The program is the last gate.** Off-chain checks are a courtesy. Never let an off-chain pass substitute for an on-chain check.
3. **`owner_withdraw` works in every state** — Active, Paused, Revoked, Expired — and the UI never disables the button.
4. **No LLM in the signing path.** A model may write into a `Features` struct and nothing else.
5. **BlockReason discriminants are append-only.** Never rename, reorder, or reuse one that has been emitted.
6. **Every allow and every block that reaches the program emits a receipt.** A refusal must be a committed log, not a rolled-back error.
7. **No APY, no promised rate, no named live venue, no "audited"** in any string that can reach a rendered page.
8. **Default action is `Skip`** and the tick interval floor is 60 seconds.

## Method (this is how you work, not a suggestion)

**Verify → implement → self-validate → evidence.**

- **Verify first.** Never assert a version, API shape, program ID, account layout, or endpoint from memory. Read the source, run the command, dump the IDL, or fetch the docs. Do at least 3–4 independent checks before implementing anything non-trivial.
- **`docs/FACTS.md` is the source of truth.** Read it before coding. Update it, with a date and the command or URL you used, on every new verification.
- **Pre-flight gate.** Each prompt lists checks that must pass before you write code. **If a pre-flight check fails: STOP and report what failed.** Do not improvise around it, do not substitute an assumption, do not "temporarily" hardcode. A failed gate produces a written decision (an ADR), not a workaround.
- **Self-validate.** After implementing, run the acceptance list yourself and paste real output — test names, signatures, curl responses. "Should work" is not evidence.
- **Evidence.** Every task ends with: what you ran, what it printed, what you wrote to FACTS, and what you did **not** verify.

## Scope discipline

Do only what the prompt asks. Anything you notice but were not asked to build goes in `docs/BACKLOG.md` with one line of why. If you find yourself adding a marketplace concept, a share/NAV concept, a fee accrual, a points system, or a second venue, stop — that is out of scope by ADR and belongs to a later phase.

## Style

- Rust: `cargo fmt`, `clippy -D warnings`, no `unwrap()` outside tests, no `panic!` in a service path, errors are typed.
- TypeScript: strict mode, no `any`, no default exports for components you will test.
- Comments explain *why*, never *what*.
- Names match the domain: `Intent`, `Verdict`, `BlockReason`, `RefusalReceipt`, `mandate`, `operator`, `owner`. Do not invent synonyms.
- Copy is plain and active: "Withdraw stays on in every state", not "Withdrawal functionality remains available".

## End-of-session output (every prompt)

```
## Verified
- <fact> — <how> — <result>
## Implemented
- <files + what they do>
## Self-validation
- <commands run + real output>
## FACTS updated
- <keys>
## Not verified / open
- <honest list>
## Backlog added
- <items>
```
