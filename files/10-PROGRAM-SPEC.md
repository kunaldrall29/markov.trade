# 10 — Program Spec
Markov Book · 31 August 2026 · v0.1
Covers `markov-mandate` and `demo-perps`. Anchor. Devnet only.

> **Reconciliation notice.** A `markov-mandate` program already exists on devnet with a receipt model and a `BlockReason` set that Gate B requires to remain intact ("all eleven historical BlockReasons still exist"). This spec is written as a **delta**. Before writing code, dump the deployed IDL, extract the exact enum discriminants and account layouts, and record them in `docs/FACTS.md`. If anything below conflicts with the deployed program, the deployed program wins and this file gets an ADR amendment. Never renumber a discriminant that has been emitted.

---

## 1. Accounts

### 1.1 `Mandate` (PDA)

```
seeds = [b"mandate", owner.key(), strategy_id, &nonce.to_le_bytes()]
```

| Field | Type | Notes |
|---|---|---|
| `owner` | `Pubkey` | only key that can withdraw, unpause, amend |
| `operator` | `Pubkey` | house agent; propose-only |
| `emergency` | `Pubkey` | pause/revoke only |
| `strategy_id` | `[u8; 16]` | `BOOK_ONE` constant for the house book |
| `state` | `MandateState` | `Active \| Paused \| Revoked \| Expired` |
| `policy` | `Policy` | inline, see 1.2 |
| `vault` | `Pubkey` | token account owned by the mandate PDA |
| `mint` | `Pubkey` | settlement mint (USDC-d) |
| `day_epoch` | `i64` | UTC day index for the rolling daily counters |
| `day_notional_used` | `u64` | reset when `day_epoch` rolls |
| `day_spend_used` | `u64` | data/compute spend |
| `action_seq` | `u64` | monotonic; goes on every receipt |
| `created_at`, `expiry_ts` | `i64` | |
| `bump`, `vault_bump` | `u8` | |

Reserve trailing padding so Phase-1 fields (`max_net_delta_usd`, `max_gross_usd` on-chain) can be added without a migration. If the deployed account has no reserve, use Anchor v1's `Migration<From, To>` and record the migration in an ADR.

### 1.2 `Policy` (inline struct)

| Field | Type | Gate B value |
|---|---|---|
| `venues` | `[Pubkey; 4]` + `venues_len` | `[demo_perps]` |
| `tokens` | `[Pubkey; 4]` + `tokens_len` | `[USDC-d, SOL-d]` |
| `allowed_actions` | `u16` bitmask | `open \| increase \| reduce \| close \| flatten` |
| `per_tx_cap` | `u64` | 50 (in mint base units) |
| `daily_cap` | `u64` | 200 |
| `max_slippage_bps` | `u16` | 50 |
| `spend_per_call`, `spend_daily` | `u64` | small, non-zero |
| `max_mark_age_slots` | `u64` | e.g. 150 |
| `expiry_ts` | `i64` | now + 14 days |

**Tighten-only diff.** `amend_policy` accepts a full `Policy` and asserts, field by field:
numeric caps `new <= old`; `max_mark_age_slots` `new <= old`; allowlists `new ⊆ old`; `expiry_ts` `new <= old`; `allowed_actions` `new & !old == 0`. Any violation → hard error `PolicyNotTightened`. This is a hard `Err` (the amend must not land), unlike a gate refusal.

### 1.3 `Registry` (single PDA, upgrade-authority controlled)

| Field | Notes |
|---|---|
| `admin` | documented single key on devnet; the accepted risk in `SECURITY.md` |
| `global_halt` | `bool`; when true every `execute_venue_action` refuses with `GlobalHalt` |
| `adapters` | allowlisted venue program IDs the mandate program will CPI into |

The registry can only ever *stop* things or shrink the adapter set in Gate B. It cannot move funds, cannot unpause a mandate, cannot widen a mandate policy.

## 2. Instructions

| Instruction | Signer | Legal in | Effect |
|---|---|---|---|
| `create_mandate(policy, strategy_id, nonce)` | owner | — | creates PDA + vault |
| `fund(amount)` | owner | Active, Paused | transfer to vault |
| `amend_policy(policy)` | owner | Active, Paused | tighten-only |
| `pause()` | owner or emergency | Active | → Paused |
| `unpause()` | **owner only** | Paused | → Active |
| `revoke()` | owner or emergency | Active, Paused | → Revoked, terminal |
| `execute_venue_action(intent)` | operator | any (gates decide) | the ladder, then CPI |
| `owner_withdraw(amount)` | **owner only** | **every state** | vault → owner ATA |
| `close_mandate()` | owner | vault empty | rent back to owner |
| `set_global_halt(bool)` | registry admin | — | circuit |

`execute_venue_action` argument:

```rust
pub struct Intent {
    pub intent_id: [u8; 32],     // blake3(mandate, slot_bucket, action, amount, nonce)
    pub action: ActionKind,      // Open|Increase|Reduce|Close|Flatten
    pub market: [u8; 16],        // e.g. SOL-PERP
    pub notional: u64,           // in mint base units
    pub side: Side,              // Long|Short
    pub limit_price: u64,        // for the slippage gate
    pub max_slippage_bps: u16,   // <= policy
    pub spend: u64,              // data/compute charged to this action
    pub forced: bool,            // redteam marker; NEVER skips a gate, only recorded
}
```

`intent_id` gives idempotency: a resubmission of the same id in the same `day_epoch` is refused with `DuplicateIntent` rather than double-filling.

## 3. Gate order

Implemented in `gates.rs` as one function per gate, called in this exact sequence. First failure emits a `RefusalReceipt` and returns `Ok(())`.

| # | Gate | BlockReason |
|---|---|---|
| 1 | registry global halt | `GlobalHalt` |
| 2 | mandate state | `Paused` / `Revoked` / `Expired` |
| 3 | signer is `mandate.operator` | `NotOperator` |
| 4 | duplicate `intent_id` | `DuplicateIntent` |
| 5 | venue program in `policy.venues` **and** in `registry.adapters` | `VenueNotAllowed` |
| 6 | mint in `policy.tokens` | `TokenNotAllowed` |
| 7 | action in `policy.allowed_actions` | `ActionNotAllowed` |
| 8 | `notional <= per_tx_cap` | `OverTxCap` |
| 9 | `day_notional_used + notional <= daily_cap` | `OverDailyCap` |
| 10 | `spend <= spend_per_call` / `day_spend_used + spend <= spend_daily` | `OverSpendCap` / `OverSpendDailyCap` |
| 11 | `intent.max_slippage_bps <= policy` and quote within bound | `SlippageExceeded` |
| 12 | `slot - mark.slot <= policy.max_mark_age_slots` | `StaleOracle` |
| 13 | CPI returns error | `VenueRejected` |
| 14 | post-checks (vault delta, position delta, no authority change) | `PostCheckFailed` |

**Post-checks, explicitly.** After the CPI returns: the vault's token balance changed by no more than the intent's notional plus fees; the vault's `owner` and `delegate` are unchanged; the mandate's `owner`, `operator`, and `policy` bytes are unchanged. Any drift → `PostCheckFailed`, and because the CPI already happened, the whole transaction reverts with a hard error so nothing lands. This is the one place where a refusal is allowed to be an `Err`: a state we cannot describe is a state we do not keep.

## 4. `BlockReason`

**Append-only.** Discriminants come from the deployed program; the names below are the working set. Week 0 dumps the IDL and fills the real numbers into FACTS. Never edit a row that has been emitted; add new rows at the end.

| # | Name | Meaning |
|---|---|---|
| 0 | `Paused` | mandate paused |
| 1 | `Revoked` | mandate revoked |
| 2 | `Expired` | past expiry |
| 3 | `NotOperator` | signer is not the mandate operator |
| 4 | `VenueNotAllowed` | venue not in policy/registry |
| 5 | `TokenNotAllowed` | mint not in policy |
| 6 | `OverTxCap` | notional above per-tx cap |
| 7 | `OverDailyCap` | notional above rolling daily cap |
| 8 | `OverSpendCap` | spend above per-call budget |
| 9 | `OverSpendDailyCap` | spend above daily budget |
| 10 | `SlippageExceeded` | quote outside bound |
| — | `StaleOracle` | *new in Gate B* — mark older than policy allows |
| — | `ActionNotAllowed` | *new* — action kind not permitted |
| — | `DuplicateIntent` | *new* — replay of an intent id |
| — | `GlobalHalt` | *new* — registry circuit open |
| — | `VenueRejected` | *new* — venue CPI returned an error |
| — | `PostCheckFailed` | *new* — invariant broken after CPI |

Gate B requires the original eleven to still exist **and** requires Book One to have actually triggered `OverTxCap`, `Revoked`, and one of the spend/slippage reasons.

## 5. Events (the receipts)

Emitted with Anchor's CPI-event mechanism so the payload survives log truncation and can be parsed from the IDL rather than by string matching.

```rust
#[event] pub struct ActionReceipt {
    pub seq: u64, pub intent_id: [u8;32], pub mandate: Pubkey,
    pub owner: Pubkey, pub operator: Pubkey, pub strategy_id: [u8;16],
    pub venue: Pubkey, pub market: [u8;16], pub action: u8, pub side: u8,
    pub notional: u64, pub fill_price: u64, pub mark_price: u64, pub mark_slot: u64,
    pub spend: u64, pub forced: bool, pub ts: i64, pub slot: u64,
    // metadata, off-chain enforced in v0 (ADR-05)
    pub net_delta_usd_e6: i64, pub gross_usd_e6: u64,
}

#[event] pub struct RefusalReceipt {
    pub seq: u64, pub intent_id: [u8;32], pub mandate: Pubkey,
    pub operator: Pubkey, pub strategy_id: [u8;16], pub venue: Pubkey,
    pub action: u8, pub notional: u64, pub reason: u8,   // BlockReason
    pub gate_index: u8, pub forced: bool, pub ts: i64, pub slot: u64,
}
```

Also emit `OwnerAction { kind: Fund|Amend|Pause|Unpause|Revoke|Withdraw, .. }` so the feed can show the owner's own moves next to the agent's. B6's proof is a *pair* — revoke, then the next attempt refused — and both halves need to be indexable.

**Never in an event:** raw instruction payloads, keypairs, private URLs, or any field the API would have to redact later.

## 6. Venue adapter ABI

One trait, two consumers: the program CPIs into it, and the off-chain `markov-venue` crate mirrors it for quoting.

| Method | Direction | Purpose |
|---|---|---|
| `mark(market) -> (price, expo, slot)` | read | current mark and its freshness |
| `positions(mandate) -> [Position]` | read | size, side, entry, funding accrued |
| `open(mandate, market, side, notional, limit)` | write, CPI | new position |
| `increase(mandate, market, notional, limit)` | write, CPI | add to position |
| `reduce(mandate, market, notional, limit)` | write, CPI | trim |
| `close(mandate, market, limit)` | write, CPI | flat |

**Rules that make this a real seam, not a stub:**
- Every write takes the **mandate PDA as the authority signer** — the operator key never signs to the venue directly.
- Every write returns a `Fill { price, notional, fee }` the program can post-check.
- Errors are a fixed set: `MarketUnknown`, `StaleMark`, `SlippageExceeded`, `InsufficientCollateral`, `PositionLimit`, `VenuePaused`. A real venue's error space maps onto these; anything unmapped becomes `VenueRejected`.
- No method may take a `&str` price feed name or a client-supplied price. Marks come from an account.

## 7. `demo-perps`

A mock venue, not a toy: it exists so the interface, the freshness gate, and the receipt shape are exercised for real.

| Account | Purpose |
|---|---|
| `Market` | `market_id`, `base_decimals`, `mark_account`, `fee_bps`, `paused` |
| `MarkAccount` | `price`, `expo`, `slot`, `source` (`pyth` \| `house`), `poster` |
| `Position` PDA `[b"pos", mandate, market_id]` | `side`, `notional`, `entry_price`, `funding_accrued`, `updated_slot` |

Behaviour:
- Fills at `mark ± fee_bps`, deterministic — no random slippage, because a demo that flatters itself is worse than no demo.
- `funding_accrued` advances by a fixed, published devnet rate per elapsed slot so the "funding 7d" tile has an honest, clearly-labelled devnet source.
- Rejects with `StaleMark` if `slot - mark.slot > market.max_age`. The mandate program also checks this; two independent checks is the point.
- **Cannot move tokens on its own.** Collateral movement happens in the mandate program's CPI, authorised by the mandate PDA, or the mock holds no custody at all and only tracks notional. Prefer the second: for Gate B, `demo_perps` is an accounting mock with **zero token custody**, so a bug in the mock cannot touch a vault. Record this in `SECURITY.md`.

`mark-poster` is a small job that pulls the price off-chain and writes `MarkAccount`. It signs with its own key, is allowlisted per market, and can only write price/slot/source. It cannot pause markets, cannot open positions, and holds no tokens.

## 8. Test obligations (program)

One LiteSVM test per row. These are the acceptance tests referenced by `P02`–`P04`.

| Test | Asserts |
|---|---|
| `withdraw_active/paused/revoked/expired` | `owner_withdraw` succeeds in all four states |
| `withdraw_not_owner` | any non-owner signer fails |
| `operator_cannot_withdraw` | explicit, named, and permanent |
| `operator_cannot_unpause` | only owner unpauses |
| `emergency_can_pause_revoke_only` | unpause and withdraw fail for emergency |
| `amend_tighten_ok` / `amend_widen_rejected` | tighten-only |
| `gate_order_*` (one per BlockReason) | correct reason **and** correct `gate_index` |
| `refusal_emits_and_commits` | a refusal produces a durable log, not a rollback |
| `duplicate_intent_refused` | idempotency |
| `stale_mark_refused` | freshness enforced on chain |
| `post_check_reverts` | tampering after CPI reverts everything |
| `venue_not_in_registry` | policy allow is not enough; registry must allow too |
| `daily_counter_rolls` | `day_epoch` rollover resets used amounts |

Green suite is a merge requirement, not a milestone.
