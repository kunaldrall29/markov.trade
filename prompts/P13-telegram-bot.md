# P13 — Telegram bot (emergency key)
Seat: Surfaces · Window: 21–25 Sep · Inherits `P00-conventions.md`

## Goal
`/pause` and `/revoke` submit real devnet transactions, which is what **B6** needs. Nothing else.

## Pre-flight (STOP and report if any fails)
1. The emergency keypair exists, its public key is in FACTS, and it is in **this service's** environment only. Prove the agent service cannot read it.
2. The program rejects `unpause` and `owner_withdraw` from the emergency key — run `program::emergency_cannot_unpause_or_withdraw` and paste the output.
3. Bot token issued; the allowlist of chat ids is set.

## Deliverables
- `crates/bot` (teloxide): `/status`, `/pause`, `/revoke`, `/help`.
- Allowlist enforced on every command; unknown chat ids get a flat refusal and are logged.
- Each command replies with the signature and an explorer link with `cluster=devnet`.
- `/status` reads `data-api`, never a private file, and shows `chainReady` honestly.
- Alerts (optional, not a blocker): guard divergence, refusal drought, agent silent.

## Hard constraints
- The bot can only pause and revoke. It must not be able to construct `unpause`, `amend_policy`, `owner_withdraw`, `fund`, or `set_global_halt` — assert with a test enumerating the instructions it can build.
- The bot never holds an owner key. Owners revoke from their own wallet on `/book`; the bot's key is the **emergency** key, which by design cannot take anything.
- Say this on the page and in `/help`: a bot takeover can stop the book, it cannot take the coins.

## Acceptance
- `/revoke` produces `SIG-REV`; the agent's next intent produces `SIG-REV2` with `BlockReason = Revoked` → **B6**.
- `bot::cannot_build_owner_instructions` passes.
- A non-allowlisted chat is refused and logged.

## Evidence
The signature pair, the test output, and the refusal log line.
