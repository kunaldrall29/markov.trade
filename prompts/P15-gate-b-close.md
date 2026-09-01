# P15 — Close Gate B
Seat: Truth (with Surfaces on the tape) · Window: 25–27 Sep · Inherits `P00-conventions.md`

## Goal
Turn a working system into a closed gate: eight signatures, one tape, one security file, and a status line that a stranger can check.

## Pre-flight (STOP and report if any fails)
1. `gate-b-verify.sh` prints green for every automatable item.
2. Paper folder has ≥7 consecutive dated files with `PAPER_START_DATE` in FACTS.
3. `SECURITY.md` is written from `docs/14-SECURITY-AND-KEYS.md` §6 with real pubkeys.
4. The key-rotation drill has been run: the old operator key's next proposal was refused with `NotOperator`, and both signatures are recorded.

## Deliverables
- **The 90-second tape**, recorded once against production hosts, following the beat sheet:

| t | Beat | Proof |
|---|---|---|
| 0:00 | `/book`, counters, devnet label | URL |
| 0:15 | Fund; tokens in the mandate PDA | `SIG-FUND` |
| 0:30 | Agent hedge; ActionReceipt | `SIG-ACT` |
| 0:45 | Lower `per_tx_cap` below the next clip | `SIG-AMEND` |
| 0:55 | Next clip refused `OverTxCap` | `SIG-CAP` |
| 1:05 | Revoke via UI or Telegram | `SIG-REV` |
| 1:15 | Next attempt refused `Revoked` | `SIG-REV2` |
| 1:25 | Withdraw while Revoked; owner balance up | `SIG-WD` |

- `docs/demo/GATE-B.md` with the eight signatures, explorer links, timestamps, and the recording.
- FACTS: all eight SIG keys, `BOOK_URL`, `HEALTH_URL`, `RECEIPTS_API_URL`, `UPGRADE_AUTHORITY`, `PAPER_START_DATE`.
- The close ritual, in order: open every URL logged-out → replay the tape from a **second wallet** → chain-query receipt count vs API count → write `GATE-B.md` → set `GATE_B_STATUS = CLOSED` with the date in FACTS.

## Hard constraints
- No localhost in the tape. No cropped browser chrome hiding a URL. No cuts inside a beat.
- The replay from a second wallet is not optional; it is the difference between a demo and a system.
- Until `GATE_B_STATUS = CLOSED` is committed, the answer everywhere — grants, threads, conversations — is **"Gate B open"**.
- If an item is still red on 27 Sep: say so, start Colosseum with Gate B open and disclosed, and do **not** add a real-venue promise in the same week.

## Acceptance
`gate-b-verify.sh` green, eight signatures resolvable on the explorer by someone who has never met us, and the tape watchable without narration.

## Evidence
The tape, `GATE-B.md`, the FACTS diff, and the second-wallet replay signatures.
