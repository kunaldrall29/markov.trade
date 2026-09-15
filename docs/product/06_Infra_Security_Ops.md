# 06 — Infrastructure, Security and Operations — Build Prompt

Inherit `00_Build_Conventions.md`. Everything here runs before the first mainnet rehearsal.

## 1. Topology

| Component | Where | Notes |
| --- | --- | --- |
| `services/api`, `services/keeper`, `services/mcp` | containers on a managed runtime (Cloud Run / Fly.io / ECS) in one region, two replicas for `api`, one leader-elected `keeper` | keeper leader lock in Redis with fencing token; a second keeper never runs a cycle without the lock |
| Postgres | managed (Supabase or RDS), PITR enabled, daily snapshots retained 30 days | migrations via `drizzle` or `prisma migrate`, reviewed |
| Redis | managed | queues, locks, rate limits |
| RPC | two providers (e.g. Helius + QuickNode/Triton) with health-checked failover; dedicated endpoints for keeper sends; `sendTransaction` with `skipPreflight=false` and simulation first | never public RPC in production |
| Signing service | isolated container with access to KMS; exposes `POST /sign` with allowlisted program ids, max lamports, per-minute limits, audit log | no other service can reach KMS |
| Terminal, landing | Vercel | env-specific builds; preview builds cannot reach mainnet keeper endpoints |
| Object storage | evidence artefacts, receipts export | versioned, immutable |

## 2. Secrets and keys

- KMS-backed Ed25519 keys for keeper and reconciliation signers; keys never exported. Rotation runbook: create new key → `set_permission` new actor on-chain → drain old actor → `revoke_permission` → destroy old key. Rehearsed on devnet before mainnet.
- Squads multisig for program upgrade authority and `GlobalConfig.admin`; 2-of-3 with hardware wallets; transaction proposals reviewed against the verifiable build hash.
- Secrets manager for RPC keys, DB URLs, OAuth secrets; injected at deploy; scanned in CI (gitleaks).
- Encrypted-at-rest columns for any owner-bound Pacifica agent key material; envelope encryption with KMS; decrypt only inside the adapter process.

## 3. Deployment and CI

- GitHub Actions: lint → typecheck → unit → program tests (Anchor + LiteSVM) → adapter recorded-response tests → live read smoke against mainnet → build containers → deploy to `rehearsal` → nightly devnet/testnet integration → manual gate → deploy to `mainnet`.
- `packages/facts` drift check: CI fails if a constant used in code is not present in `docs/FACTS.md` or differs.
- Verifiable program builds; the deployed program hash is checked against the repo hash on every CI run (`solana-verify`).
- Database migrations run in a pre-deploy job with a backup taken first; rollback documented.

## 4. Monitoring and alerting (page = wakes someone; warn = ticket)

| Signal | Threshold | Level |
| --- | --- | --- |
| Keeper cycle without receipt | 1 missed cycle | page |
| Canary rule missed | 1 | page |
| `RECONCILE_MISMATCH` | any | page |
| Any `Allow` receipt whose on-chain re-validation failed (should be impossible) | any | page + global pause |
| Adapter freshness breach | > 3× window for 2 min | warn; > 10 min page |
| RPC error rate | > 5% for 5 min | warn; failover auto |
| Cap utilisation | > 80% of any global cap | warn |
| Policy decision latency p95 | > 200 ms | warn |
| Signing service refusals | any | warn; repeated → page |
| Certificate/OAuth failures | > 1% | warn |

Dashboards: decisions by outcome and reason; per-venue freshness; keeper runs; cap utilisation; receipts reconciled vs pending; Invest executed/skipped by reason; MCP calls by client.

## 5. Security checklist (sign-off required per release)

- [ ] Program: Anchor constraints reviewed; instruction-adjacency tests green; upgrade authority = multisig; verifiable build hash published.
- [ ] Signing service: allowlist of program ids equals Markov, Subscriptions, Jupiter, Token programs, compute budget; per-tx lamport cap; audit log retained.
- [ ] No private key in any container image, repo, log line or client bundle (automated scan).
- [ ] All writes idempotent; replay protection on `request_id` on-chain and off-chain.
- [ ] Rate limits and per-actor caps enforced before policy evaluation.
- [ ] Dependencies pinned; `pnpm audit` and `cargo audit` clean or waived with reason.
- [ ] Threat model updated: keeper key compromise (bounded by on-chain caps + global pause), RPC compromise (dual providers, simulation, receipt reconciliation), oracle staleness (fail closed), issuer actions (pause handling), API auth bypass (scoped tokens, tests).
- [ ] Incident runbooks rehearsed (§6).
- [ ] External audit scheduled before any cap increase above Conventions §7.

## 6. Incident runbooks (in `infra/runbooks/`, each with exact commands)

1. **Global pause** — multisig `set_global_pause(true)`; verify keeper halts; communicate stage.
2. **Keeper key compromise** — pause; revoke permission on-chain; rotate KMS key; re-issue; reconcile all invest rules; report.
3. **Reconcile mismatch** — freeze the account's automation; diff chain vs DB; correct DB only from chain; receipt correction event; root cause.
4. **RPC degradation** — failover; verify keeper sends resume; backfill missed cycles inside their windows only (never late-execute).
5. **Issuer pause on a token** — mark asset `paused`; keeper skips with `ASSET_PAUSED`; Terminal banner; no automatic resumption until manual clear.
6. **Venue outage** — mark venue `degraded`; router excludes; owner-signed reduce/close still offered where possible.
7. **Data breach** — rotate all tokens and secrets; audit log export; notify affected owners.

## 7. Backups and data retention

Postgres PITR 30 days; receipts exported daily to object storage as JSONL (immutable); audit logs 1 year; evidence artefacts retained indefinitely.

## 8. Cost and quotas

Two RPC plans sized for keeper + WS fan-out; KMS operations budget; Vercel Pro; Postgres small instance with autoscaling storage. Alert at 80% of any quota.

## 9. Acceptance

- [ ] Rehearsal environment deployed from CI with all alerts firing in a drill (simulate a missed keeper cycle, a stale adapter, a reconcile mismatch).
- [ ] Global pause drill executed via multisig on mainnet and reverted, with receipts.
- [ ] Key rotation drill executed on devnet.
- [ ] Security checklist signed for release 0.1.
