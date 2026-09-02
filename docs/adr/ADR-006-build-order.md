# ADR-006 — Build order for 1–27 September (Session 0 proposal)

Status: **Accepted 2026-09-02** (Kunal: "for others decide on me"; decided per the Recommendation, see Decision at the end) · Date: 2026-09-01 · Seat: all

Today is 1 Sep. Gate B closes 27 Sep or is disclosed open. The pack's P01 window (31 Aug–2 Sep) is already half gone and its pre-flight is red on three counts (no faucet SOL, D0/D3 undecided, no program row that matches the spec). The order below keeps P08 first because paper days cannot be manufactured, and pulls every decision to the front so that no seat writes code against a `PENDING` row.

## Decisions before code (target: Kunal answers by 2 Sep)

| ID | Decision | ADR | Blocks |
|---|---|---|---|
| D0 | Anchor / Agave / LiteSVM pin (decision date 2 Sep; P01's first act is the SBF build) | ADR-001 | P01+ |
| D1 | one origin vs split for `markov.trade` | ADR-002 | P11, DNS |
| D2 | mark path (on-chain Pyth account / house `MarkAccount`) | ADR-003 | **P08 today**, P04 |
| D3 | successor program vs in-place upgrade; spec amended to chain names | ADR-004 | P02 |
| D4 | Rust workspace vs existing TS services | ADR-005 | P07, P09, P10, P13 |
| — | who holds `2fpQ…`, an RPC-provider key, a Pyth Pro or provider key if needed, three fresh keypairs | FACTS | P07, P09 |

## Calendar

| Dates | Protocol | Agents | Surfaces | Truth |
|---|---|---|---|---|
| **1–2 Sep** | ADR-001/003/004 accepted; P01 pre-flight re-run (faucet, `anchor build`, LiteSVM build, Pyth SDK build) | **P08 starts** with `MARK_SOURCE=onchain` the moment D2 is answered; `paper/YYYY-MM-DD.md` day 1 written by 2 Sep | — | `docs/10` §1/§2/§4 amended to chain names; copy-grep in CI |
| 2–4 Sep | **P01** repo bootstrap, FACTS rows filled by build output | P05 guard (pure; reasons = chain 0–10 + appended + off-chain three) | — | P14 skeleton: `rust.yml`, `truth.yml`, IDL append-only check against the chain's 11 |
| 4–10 Sep | **P02** program core (successor ID, enum 0–16, receipts via `emit_cpi`, withdraw-in-every-state tests first) | P06 book-core; paper runner switches to the real core when green | P12 may start any time (off critical path) | P14 `program.yml` |
| 8–13 Sep | **P03** adapter trait, **P04** `demo_perps` + `MarkAccount`/on-chain mark, `mark-poster` | P07 runtime scaffold against P02 devnet deploy | Codama client from the P02 IDL (`packages/sdk`) | runbooks: faucet-topup, key-rotation |
| 10–20 Sep | P09 indexer (Rust, parity job, finalizer) | **P07** hosted on Railway (new services, separate env scopes), redteam schedule → B3, B4, B5, B7 by 20 Sep | **P11** `/book` from 14 Sep against the hosted `data-api` | P14 alerts (guard divergence, refusal drought) |
| 15–21 Sep | **P10** data-api, `api.markov.trade` live | 24 h skip-dominant log for B3 | P11 e2e: withdraw-enabled-in-every-state | parity deliberate-mismatch demo; indexer rebuild runbook timed |
| 21–25 Sep | — | key-rotation drill (`Unauthorized` proof) | **P13** bot (new emergency key, own service) → B6 pair; P12 port + visual diff | P14 `gate-b-verify.sh` first full table |
| 25–27 Sep | — | — | tape recorded on production hosts, second-wallet replay | **P15** close ritual; `GATE_B_STATUS` |

**P08 start rule (one sentence, superseding the three phrasings above and in ADR-003):** P08 starts now, reading the devnet Pyth account `7UVimffx…` over RPC as its mark, labelled `source: pyth-devnet-account`; it does not wait for ADR-003 to be accepted, because any later D2 answer only changes the `MarkSource` implementation, not the daily file. P08's seventh consecutive file lands on **8 Sep** if it starts 2 Sep.

**What changes if Kunal picks the other option:** ADR-004 B (in-place upgrade) inserts a `Migration<From, To>` task into P02 and requires the `2fpQ…` key before 4 Sep. ADR-005 B or C (reuse TS services) removes P09/P10 from the Protocol column, adds "bring `apps/indexer` + `apps/data-api` to `docs/12`" to the Agents column from 10 Sep, and requires a superseding ADR for `docs/07` §4 first. ADR-001 A (Anchor 0.31.1) adds the lockfile-pinning work described in ADR-001 to P01 and removes `Migration` from P02's toolbox. The calendar above assumes A / A / B; it is a proposal, not a decision.

## Slip rule (from `docs/16`)

If on 27 Sep anything is red, Gate B is disclosed open in the order B4 → B5 → B8 → B9 → B14, and no real-venue promise is added that week.

## Decision (2026-09-02)

Accepted as the plan with ADR-001 B, ADR-004 A, ADR-005 A. P08 starts today with `MARK_SOURCE=onchain`; P01 runs today with the SBF build as its first act. The RPC provider key is supplied by Kunal later; until it arrives, submits and ingestion use the public devnet RPC with backoff and the calendar rows for P07/P09 carry that as a known risk.
