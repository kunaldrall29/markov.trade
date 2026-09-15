# 01 — Markov Program (Solana, Anchor) — Build Prompt

Inherit `00_Build_Conventions.md`. Target: Solana mainnet-beta, deployed under a Squads multisig upgrade authority, rehearsed on devnet and LiteSVM first.

## 1. Purpose

One program that holds the durable truth for both products: the **account**, its **mandate** (versioned hard rules), its **permissions** (who may ask for what), and the **receipts** of every decision. Venue execution happens elsewhere (adapters, keepers, venues); the program's job is to be the thing rules and receipts cannot be changed without.

Two enforcement modes are supported from day one:
- **Owner-signed:** the owner's wallet signs the venue transaction; the program records the policy decision and receipt in the same transaction (instruction `record_decision` preceding the venue instruction) so a receipt cannot exist without the decision and vice versa.
- **Keeper-within-cap (Invest):** the keeper signs; spend is bounded by the Solana Subscriptions & Allowances program; the Markov program validates the invest mandate and records the receipt in the same transaction as the pull + swap.

Phoenix on-chain gating (PDA as trader authority, typed CPI) is designed for but **not enabled** in this build; the account layout reserves the fields.

## 2. Accounts (PDAs; seeds shown)

| Account | Seeds | Fields |
| --- | --- | --- |
| `MarkovAccount` | `["account", owner]` | owner, bump, execution_mode (OwnerSigned \| KeeperWithinCap), active_mandate_version, active_invest_mandate_version, status (Active \| Paused \| Closed), replay_domain (u64 nonce), linked_venues bitmap, created_slot, reserved[64] |
| `Mandate` | `["mandate", account, version_le]` | version, hash (sha256 of canonical encoding), max_leverage_bps, max_notional_usd, min_safety_buffer_bps, max_daily_loss_usd, approved_markets (Vec<CanonicalMarketId>, ≤ 32), tier_caps [5] leverage_bps, allowed_venues bitmap, activated_slot, activated_by |
| `InvestMandate` | `["invest", account, version_le]` | version, hash, allowlist (Vec<Pubkey mint>, ≤ 16), per_period_budget_usd, period_seconds, monthly_ceiling_usd, reserve_floor_usd, max_cost_bps, single_asset_cap_bps, execution_mode (AlwaysOn \| ReferenceSafe \| MarketHoursOnly), window_start_utc, window_end_utc, days_mask, paused, spent_this_month_usd, month_epoch, activated_slot |
| `Permission` | `["perm", account, actor]` | actor pubkey, kind (Owner \| Keeper \| McpClient \| ApiKey), scopes bitmap (account:read, risk:simulate, trade:request, trade:reduce, invest:propose, invest:execute), per_action_cap_usd, daily_cap_usd, spent_today_usd, day_epoch, expires_slot, revoked |
| `ActionReceipt` | `["receipt", account, request_id]` | request_id (u128), actor, kind (TradeOpen \| TradeReduce \| TradeClose \| InvestExecute \| InvestSkip \| MandateSet \| MandatePause \| PermissionSet), decision (Allow \| Reject \| RequireApproval \| Skip), reason_code (u16), mandate_version, invest_mandate_version, data_slot, venue_id, market_id, checks (Vec<Check{rule:u8, observed:i64, limit:i64, pass:bool}>, ≤ 12), route_snapshot_hash, tx_signature_hint (first 32 bytes) , pre_state_hash, post_state_hash, created_slot |
| `DailyLedger` | `["day", account, day_epoch]` | realized_loss_usd, gross_notional_usd, actions_count |
| `GlobalConfig` | `["config"]` | admin (multisig), paused, caps (per-account gross notional, per-position leverage, global daily invest spend), fee_bps (0 in this build), version |

Receipts are also emitted as Anchor events (`ReceiptEmitted`) with the full payload so indexers do not need to read PDAs. Receipt PDAs may be closed by the owner after 30 days to reclaim rent; the event remains.

## 3. Instructions

| Instruction | Signer | Effect |
| --- | --- | --- |
| `initialize_config(caps)` | admin multisig | one-time |
| `set_global_caps(caps)` / `set_global_pause(bool)` | admin multisig | emergency stop for keeper flows only; owner-signed flows keep working |
| `create_account(execution_mode)` | owner | creates `MarkovAccount` |
| `set_mandate(version, fields)` | owner | creates `Mandate`, bumps `active_mandate_version`, emits receipt `MandateSet` |
| `set_invest_mandate(version, fields)` | owner | same for `InvestMandate` |
| `pause / unpause` | owner | flips status; pause blocks all risk-increasing decisions |
| `set_permission(actor, kind, scopes, caps, expiry)` / `revoke_permission(actor)` | owner | grants/revokes |
| `record_decision(request_id, kind, decision, reason, checks, data_slot, venue_id, market_id, route_hash, pre_hash)` | owner **or** permitted actor within scope | validates the decision against the active mandate **on-chain** (re-computes the checks from the supplied observed values; rejects if any observed value violates the mandate but `decision == Allow`), writes `ActionReceipt`, updates `DailyLedger`. Must be instruction *i* with the venue instruction at *i+1* in the same transaction; the program verifies via `instructions` sysvar that the next instruction's program id is in `allowed_venues` for `venue_id` (for owner-signed flows) or is the Jupiter/Subscriptions pair (for invest flows). |
| `finalize_decision(request_id, post_hash, tx_hint)` | same | writes post-state hash after the venue instruction (instruction *i+2*); optional when the venue leg fails. |
| `record_skip(request_id, reason, checks, data_slot)` | keeper (invest:execute) | writes an `InvestSkip` receipt with no venue leg |
| `invest_execute(request_id, mint, usdc_amount, quote_cost_bps, official_ref, checks…)` | keeper (invest:execute) | validates against `InvestMandate` (allowlist, budget, ceiling, reserve floor via passed balance proof, cost bps, execution mode vs supplied market status), increments `spent_this_month_usd`, writes receipt; requires the next instructions to be the Subscriptions `collect` and the Jupiter swap route (verified via instructions sysvar program ids) |
| `close_receipt(request_id)` | owner | after 30 days |

**Reason codes (u16, stable):** 1 MARKET_NOT_ALLOWED, 2 MAX_LEVERAGE_EXCEEDED, 3 MAX_NOTIONAL_EXCEEDED, 4 MIN_SAFETY_BUFFER, 5 DAILY_LOSS_BUDGET_EXCEEDED, 6 STALE_MARKET_DATA, 7 SLIPPAGE_LIMIT, 8 ACTOR_SCOPE_DENIED, 9 ACTOR_CAP_EXCEEDED, 10 VENUE_NOT_ALLOWED, 11 ACCOUNT_PAUSED, 12 GLOBAL_PAUSED, 20 ASSET_NOT_ALLOWLISTED, 21 BUDGET_EXHAUSTED, 22 MONTHLY_CEILING_EXCEEDED, 23 RESERVE_FLOOR, 24 COST_LIMIT, 25 MARKET_CLOSED, 26 OFF_HOURS_DEVIATION, 27 ASSET_PAUSED, 28 NO_ROUTE, 29 STALE_QUOTE, 30 DELEGATION_INSUFFICIENT, 31 SINGLE_ASSET_CAP, 100 EXECUTION_CONFIRMED, 101 EXECUTION_FAILED.

## 4. Invariants the program enforces (tests must cover each)

1. No `Allow` receipt can be written whose supplied observed values violate the active mandate.
2. No `Allow` receipt without an adjacent venue instruction from an allowed program (instructions sysvar check).
3. A paused account or global pause yields `Reject` with reason 11/12; no state changes except the receipt.
4. Permission scope and caps are checked before mandate checks; daily caps roll at `day_epoch`.
5. `InvestMandate.spent_this_month_usd` can only increase within the month and resets on `month_epoch` change; `monthly_ceiling_usd` is never exceeded by more than rounding.
6. Replay: `request_id` is unique per account; a second `record_decision` with the same id fails.
7. Owner-only instructions require the owner signer even if a permission with all scopes exists.
8. Receipts are append-only; `finalize_decision` may only add `post_state_hash` and `tx_hint`.

## 5. Security requirements

- Anchor account constraints on every account (`has_one = owner`, seeds, bumps, `close = owner`).
- Instructions-sysvar introspection guarded against wrapping: require the current index and that the adjacent instruction is not the Markov program itself.
- Checked arithmetic everywhere; u64 for lamports/micro-USD, i64 for signed observed values.
- No `remaining_accounts` in decision paths.
- Upgrade authority: Squads multisig; program deployed with `--final` only after the audit; until then upgradeable under multisig with a published upgrade log.
- External audit before removing per-account caps; the caps in `GlobalConfig` are the pre-audit control.

## 6. Tests

- **Anchor unit tests** for every instruction and every invariant, run against `solana-test-validator` and LiteSVM.
- **Property tests** (proptest) for mandate checks: random observed values vs random mandates; never an `Allow` that violates.
- **Instruction-adjacency tests**: transactions with the venue instruction missing, misplaced, or from a disallowed program must fail.
- **Devnet integration**: create account → set mandate → set permission → `record_decision` + a real Drift devnet instruction; `invest_execute` + real Subscriptions devnet `collect` + a Jupiter-shaped instruction placeholder is *not* acceptable — on devnet there is no Jupiter, so the invest path is tested with `record_skip` and with a locally deployed swap stand-in that is **only** used in devnet CI and is clearly named `devnet_swap_stub`; the mainnet rehearsal covers the real path.
- **Mainnet rehearsal** (from `07_Mainnet_Test_Plan.md`): real receipts for each instruction with explorer links.

## 7. SDK (`packages/sdk`, TypeScript)

Codama-generated client from the IDL; helpers: `deriveAccount`, `buildRecordDecisionIx`, `buildInvestExecuteIx`, `canonicalMandateHash`, `decodeReceipt`, `subscribeReceipts(connection, account)`. All amounts as bigint micro-USD; all pubkeys as `Address` from `@solana/kit`.

## 8. Deployment

1. `anchor build --verifiable`; publish the verifiable build hash.
2. Deploy to devnet; run integration suite; record program id in FACTS with date.
3. Deploy to mainnet via Squads; `initialize_config` with the caps in Conventions §7.
4. Register the program id, IDL and multisig address in FACTS; add the Markov program to the Terminal explorer links.
5. Nothing else in the stack may hard-code the program id; it flows from `packages/facts`.

## 9. Out of scope (this build)

Phoenix PDA-authority gating and CPI, Markov Markets, cross-account portfolio limits, fee collection, permissionless keepers.
