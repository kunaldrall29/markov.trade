//! `markov` — the control-plane program (`01_Program_Build_Prompt`).
//!
//! One program holds the durable truth for every Markov product: the
//! account, its versioned mandate (hard rules), its permissions (who may ask
//! for what) and the receipts of every decision. Venue execution happens in
//! the adjacent instruction; this program is the thing a receipt cannot exist
//! without, and the thing an Allow cannot bypass.
//!
//! Invariants (`01` §4), each covered by a test in `tests/`:
//! 1. No `Allow` receipt whose observed values violate the active mandate.
//! 2. No `Allow` receipt without an adjacent venue instruction from the
//!    configured program for that venue (instructions-sysvar check).
//! 3. A paused account (risk-increasing kinds) or global pause (actor flows)
//!    yields `Reject` 11/12; only the receipt is written.
//! 4. Permission scope and caps are checked before mandate checks; daily caps
//!    roll on the UTC day.
//! 5. `InvestMandate.spent_this_month_usd` only grows within a month and
//!    resets when the month changes; the ceiling is never exceeded.
//! 6. `request_id` is unique per account: the receipt PDA's `init` refuses a
//!    second decision with the same id.
//! 7. Owner-only instructions require the owner's signature; no permission
//!    grants them.
//! 8. Receipts are append-only; `finalize_decision` adds `post_state_hash` and
//!    `tx_signature_hint` once and nothing else.
#![allow(unexpected_cfgs)]
#![forbid(unsafe_code)]

use anchor_lang::prelude::*;
use anchor_spl::token_interface::TokenAccount;

pub mod checks;
pub mod errors;
pub mod events;
pub mod introspection;
pub mod state;

use crate::checks::{
    evaluate_invest, evaluate_trade, month_epoch, weekday_bit, InvestFacts, TradeFacts,
};
use crate::errors::MarkovError;
use crate::events::{ReceiptEmitted, ReceiptFinalized};
use crate::state::*;

// Placeholder id for tests and the IDL. The deploy keypair is generated on the
// deploying box (`anchor keys sync`) and the id recorded in FACTS (`01` §8).
declare_id!("qKxcmWWh5zLcm9jWExhH4Cm5eGmS3w5f1BhkUb6NsXs");

fn canonical_hash<T: AnchorSerialize>(v: &T) -> Result<[u8; 32]> {
    let mut bytes = Vec::new();
    v.serialize(&mut bytes).map_err(|_| MarkovError::Math)?;
    Ok(solana_sha256_hasher::hash(&bytes).to_bytes())
}

fn day_of(ts: i64) -> i64 {
    ts.div_euclid(SECONDS_PER_DAY)
}

/// What `record_decision` is handed. Every observed value comes from the
/// caller; every limit is taken from the chain (see `checks`).
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct DecisionArgs {
    pub request_id: u128,
    pub kind: ReceiptKind,
    pub decision: Decision,
    pub reason_code: u16,
    pub checks: Vec<Check>,
    pub data_slot: u64,
    pub venue_id: u8,
    pub market_id: MarketId,
    pub route_snapshot_hash: [u8; 32],
    pub pre_state_hash: [u8; 32],
    /// Projected notional of this action, micro-USD; charged to actor and
    /// ledger caps on Allow.
    pub notional_usd: u64,
    /// The UTC day this decision is charged to; must equal the clock's day.
    pub day_epoch: i64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct InvestArgs {
    pub request_id: u128,
    pub mint: Pubkey,
    pub usdc_amount: u64,
    pub quote_cost_bps: u16,
    /// Official reference and quote prices, micro-USD per unit; 0 = none.
    pub official_price: u64,
    pub quote_price: u64,
    pub market_status: MarketStatus,
    pub checks: Vec<Check>,
    pub data_slot: u64,
    pub route_snapshot_hash: [u8; 32],
    pub pre_state_hash: [u8; 32],
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct SkipArgs {
    pub request_id: u128,
    pub reason_code: u16,
    pub checks: Vec<Check>,
    pub data_slot: u64,
}

fn scope_for(kind: ReceiptKind) -> Option<u32> {
    match kind {
        ReceiptKind::TradeOpen => Some(scopes::TRADE_REQUEST),
        ReceiptKind::TradeReduce | ReceiptKind::TradeClose => Some(scopes::TRADE_REDUCE),
        ReceiptKind::InvestExecute | ReceiptKind::InvestSkip => Some(scopes::INVEST_EXECUTE),
        ReceiptKind::MandateSet | ReceiptKind::MandatePause | ReceiptKind::PermissionSet => None,
    }
}

fn risk_increasing(kind: ReceiptKind) -> bool {
    matches!(kind, ReceiptKind::TradeOpen | ReceiptKind::InvestExecute)
}

/// Actor gate (invariant 4): owner passes; anyone else needs a live permission
/// with the scope for `kind` and headroom on both caps. Returns the reason
/// that refused, or `NONE`, and rolls the permission's day.
fn actor_gate(
    signer: &Pubkey,
    owner: &Pubkey,
    permission: &mut Option<Box<Account<'_, Permission>>>,
    kind: ReceiptKind,
    notional_usd: u64,
    now: i64,
    slot: u64,
) -> Result<u16> {
    if signer == owner {
        return Ok(reason::NONE);
    }
    let scope = scope_for(kind).ok_or(MarkovError::NotOwner)?;
    let perm = permission.as_mut().ok_or(MarkovError::NoPermission)?;
    require_keys_eq!(perm.actor, *signer, MarkovError::NoPermission);
    if perm.revoked {
        return Err(MarkovError::PermissionRevoked.into());
    }
    if perm.expires_slot != 0 && slot >= perm.expires_slot {
        return Err(MarkovError::PermissionExpired.into());
    }
    perm.roll_day(now);
    if perm.scopes & scope == 0 {
        return Ok(reason::ACTOR_SCOPE_DENIED);
    }
    if perm.per_action_cap_usd != 0 && notional_usd > perm.per_action_cap_usd {
        return Ok(reason::ACTOR_CAP_EXCEEDED);
    }
    let day_total = perm
        .spent_today_usd
        .checked_add(notional_usd)
        .ok_or(MarkovError::Math)?;
    if perm.daily_cap_usd != 0 && day_total > perm.daily_cap_usd {
        return Ok(reason::ACTOR_CAP_EXCEEDED);
    }
    Ok(reason::NONE)
}

#[program]
pub mod markov {
    use super::*;

    pub fn initialize_config(
        ctx: Context<InitializeConfig>,
        caps: GlobalCaps,
        usdc_mint: Pubkey,
    ) -> Result<()> {
        let c = &mut ctx.accounts.config;
        c.admin = ctx.accounts.admin.key();
        c.paused = false;
        c.caps = caps;
        c.fee_bps = 0;
        c.version = 1;
        c.venue_programs = [Pubkey::default(); MAX_VENUES];
        c.invest_collect_program = Pubkey::default();
        c.invest_swap_program = Pubkey::default();
        c.usdc_mint = usdc_mint;
        c.invest_day_epoch = day_of(Clock::get()?.unix_timestamp);
        c.invest_spent_today_usd = 0;
        c.bump = ctx.bumps.config;
        Ok(())
    }

    pub fn set_global_caps(ctx: Context<AdminOnly>, caps: GlobalCaps) -> Result<()> {
        ctx.accounts.config.caps = caps;
        ctx.accounts.config.version = ctx.accounts.config.version.saturating_add(1);
        Ok(())
    }

    /// Emergency stop for actor (keeper, MCP, API-key) flows. Owner-signed
    /// flows keep working (`01` §3).
    pub fn set_global_pause(ctx: Context<AdminOnly>, paused: bool) -> Result<()> {
        ctx.accounts.config.paused = paused;
        Ok(())
    }

    /// The zero pubkey (the system program's id) means "unset"; it can never
    /// be a venue, and neither can this program.
    pub fn set_venue_program(ctx: Context<AdminOnly>, venue_id: u8, program: Pubkey) -> Result<()> {
        require!((venue_id as usize) < MAX_VENUES, MarkovError::UnknownVenue);
        require_keys_neq!(program, crate::ID, MarkovError::AdjacentIsSelf);
        require_keys_neq!(program, Pubkey::default(), MarkovError::UnknownVenue);
        ctx.accounts.config.venue_programs[venue_id as usize] = program;
        Ok(())
    }

    pub fn set_invest_programs(
        ctx: Context<AdminOnly>,
        collect: Pubkey,
        swap: Pubkey,
    ) -> Result<()> {
        require_keys_neq!(collect, crate::ID, MarkovError::AdjacentIsSelf);
        require_keys_neq!(swap, crate::ID, MarkovError::AdjacentIsSelf);
        require_keys_neq!(collect, Pubkey::default(), MarkovError::UnknownVenue);
        require_keys_neq!(swap, Pubkey::default(), MarkovError::UnknownVenue);
        ctx.accounts.config.invest_collect_program = collect;
        ctx.accounts.config.invest_swap_program = swap;
        Ok(())
    }

    pub fn create_account(
        ctx: Context<CreateAccount>,
        execution_mode: ExecutionMode,
    ) -> Result<()> {
        let a = &mut ctx.accounts.account;
        a.owner = ctx.accounts.owner.key();
        a.bump = ctx.bumps.account;
        a.execution_mode = execution_mode;
        a.active_mandate_version = 0;
        a.active_invest_mandate_version = 0;
        a.status = AccountStatus::Active;
        a.replay_domain = 0;
        a.linked_venues = 0;
        a.created_slot = Clock::get()?.slot;
        a.reserved = [0u8; 64];
        Ok(())
    }

    pub fn link_venue(ctx: Context<OwnerOnly>, venue_id: u8) -> Result<()> {
        require!(venue_id < 64, MarkovError::UnknownVenue);
        ctx.accounts.account.linked_venues |= 1u64 << venue_id;
        Ok(())
    }

    pub fn set_mandate(
        ctx: Context<SetMandate>,
        version: u32,
        fields: MandateFields,
    ) -> Result<()> {
        let account = &mut ctx.accounts.account;
        require!(
            account.status != AccountStatus::Closed,
            MarkovError::AccountClosed
        );
        require!(
            version == account.active_mandate_version.saturating_add(1),
            MarkovError::BadMandateVersion
        );
        require!(fields.validate(), MarkovError::InvalidMandate);
        let clock = Clock::get()?;
        let m = &mut ctx.accounts.mandate;
        m.account = account.key();
        m.version = version;
        m.hash = canonical_hash(&fields)?;
        m.max_leverage_bps = fields.max_leverage_bps;
        m.max_notional_usd = fields.max_notional_usd;
        m.min_safety_buffer_bps = fields.min_safety_buffer_bps;
        m.max_daily_loss_usd = fields.max_daily_loss_usd;
        m.approved_markets = fields.approved_markets;
        m.tier_caps_bps = fields.tier_caps_bps;
        m.allowed_venues = fields.allowed_venues;
        m.activated_slot = clock.slot;
        m.activated_by = ctx.accounts.owner.key();
        m.bump = ctx.bumps.mandate;
        account.active_mandate_version = version;
        emit_cpi!(ReceiptEmitted {
            account: account.key(),
            request_id: u128::from(version),
            actor: ctx.accounts.owner.key(),
            kind: ReceiptKind::MandateSet,
            decision: Decision::Allow,
            reason_code: reason::NONE,
            mandate_version: version,
            invest_mandate_version: account.active_invest_mandate_version,
            data_slot: clock.slot,
            venue_id: 0,
            market_id: [0u8; 16],
            checks: vec![],
            route_snapshot_hash: [0u8; 32],
            pre_state_hash: m.hash,
            created_slot: clock.slot,
            created_ts: clock.unix_timestamp,
        });
        Ok(())
    }

    pub fn set_invest_mandate(
        ctx: Context<SetInvestMandate>,
        version: u32,
        fields: InvestMandateFields,
    ) -> Result<()> {
        let account = &mut ctx.accounts.account;
        require!(
            account.status != AccountStatus::Closed,
            MarkovError::AccountClosed
        );
        require!(
            version == account.active_invest_mandate_version.saturating_add(1),
            MarkovError::BadMandateVersion
        );
        require!(fields.validate(), MarkovError::InvalidMandate);
        let clock = Clock::get()?;
        let m = &mut ctx.accounts.invest_mandate;
        m.account = account.key();
        m.version = version;
        m.hash = canonical_hash(&fields)?;
        m.allowlist = fields.allowlist;
        m.per_period_budget_usd = fields.per_period_budget_usd;
        m.period_seconds = fields.period_seconds;
        m.monthly_ceiling_usd = fields.monthly_ceiling_usd;
        m.reserve_floor_usd = fields.reserve_floor_usd;
        m.max_cost_bps = fields.max_cost_bps;
        m.max_reference_deviation_bps = fields.max_reference_deviation_bps;
        m.single_asset_cap_bps = fields.single_asset_cap_bps;
        m.execution_mode = fields.execution_mode;
        m.window_start_utc = fields.window_start_utc;
        m.window_end_utc = fields.window_end_utc;
        m.days_mask = fields.days_mask;
        m.paused = false;
        m.spent_this_month_usd = 0;
        m.month_epoch = month_epoch(clock.unix_timestamp);
        m.activated_slot = clock.slot;
        m.bump = ctx.bumps.invest_mandate;
        account.active_invest_mandate_version = version;
        emit_cpi!(ReceiptEmitted {
            account: account.key(),
            request_id: u128::from(version) | (1u128 << 64),
            actor: ctx.accounts.owner.key(),
            kind: ReceiptKind::MandateSet,
            decision: Decision::Allow,
            reason_code: reason::NONE,
            mandate_version: account.active_mandate_version,
            invest_mandate_version: version,
            data_slot: clock.slot,
            venue_id: 0,
            market_id: [0u8; 16],
            checks: vec![],
            route_snapshot_hash: [0u8; 32],
            pre_state_hash: m.hash,
            created_slot: clock.slot,
            created_ts: clock.unix_timestamp,
        });
        Ok(())
    }

    pub fn pause(ctx: Context<OwnerOnly>) -> Result<()> {
        require!(
            ctx.accounts.account.status == AccountStatus::Active,
            MarkovError::AccountClosed
        );
        ctx.accounts.account.status = AccountStatus::Paused;
        Ok(())
    }

    pub fn unpause(ctx: Context<OwnerOnly>) -> Result<()> {
        require!(
            ctx.accounts.account.status == AccountStatus::Paused,
            MarkovError::AccountClosed
        );
        ctx.accounts.account.status = AccountStatus::Active;
        Ok(())
    }

    pub fn pause_invest(ctx: Context<OwnerInvest>, paused: bool) -> Result<()> {
        ctx.accounts.invest_mandate.paused = paused;
        Ok(())
    }

    pub fn set_permission(
        ctx: Context<SetPermission>,
        actor: Pubkey,
        kind: ActorKind,
        scopes: u32,
        per_action_cap_usd: u64,
        daily_cap_usd: u64,
        expires_slot: u64,
    ) -> Result<()> {
        require!(scopes & !scopes::ALL == 0, MarkovError::ScopeDenied);
        let now = Clock::get()?.unix_timestamp;
        let p = &mut ctx.accounts.permission;
        p.account = ctx.accounts.account.key();
        p.actor = actor;
        p.kind = kind;
        p.scopes = scopes;
        p.per_action_cap_usd = per_action_cap_usd;
        p.daily_cap_usd = daily_cap_usd;
        if p.day_epoch != day_of(now) {
            p.day_epoch = day_of(now);
            p.spent_today_usd = 0;
        }
        p.expires_slot = expires_slot;
        p.revoked = false;
        p.bump = ctx.bumps.permission;
        Ok(())
    }

    pub fn revoke_permission(ctx: Context<RevokePermission>) -> Result<()> {
        ctx.accounts.permission.revoked = true;
        Ok(())
    }

    /// Instruction *i*. Validates the decision against the chain, writes the
    /// receipt and the ledger, and requires the venue instruction at *i+1*
    /// for an Allow (or forbids one for anything else).
    pub fn record_decision(ctx: Context<RecordDecision>, args: DecisionArgs) -> Result<()> {
        require!(args.checks.len() <= MAX_CHECKS, MarkovError::TooManyChecks);
        require!(
            matches!(
                args.kind,
                ReceiptKind::TradeOpen | ReceiptKind::TradeReduce | ReceiptKind::TradeClose
            ),
            MarkovError::WrongDecisionKind
        );
        let clock = Clock::get()?;
        let now = clock.unix_timestamp;
        require!(args.day_epoch == day_of(now), MarkovError::WrongDayEpoch);
        let signer = ctx.accounts.signer.key();
        let account = &mut ctx.accounts.account;
        require!(
            account.status != AccountStatus::Closed,
            MarkovError::AccountClosed
        );
        require!(
            account.active_mandate_version > 0,
            MarkovError::NoActiveMandate
        );
        let is_owner = signer == account.owner;

        // Ledger for today (fresh accounts are zero; the seeds pin the day).
        let ledger = &mut ctx.accounts.ledger;
        if ledger.day_epoch != args.day_epoch {
            ledger.account = account.key();
            ledger.day_epoch = args.day_epoch;
            ledger.realized_loss_usd = 0;
            ledger.gross_notional_usd = 0;
            ledger.actions_count = 0;
            ledger.bump = ctx.bumps.ledger;
        }

        // Invariant 4: actor first.
        let actor_reason = actor_gate(
            &signer,
            &account.owner,
            &mut ctx.accounts.permission,
            args.kind,
            args.notional_usd,
            now,
            clock.slot,
        )?;

        // Invariant 3: pauses.
        let config = &ctx.accounts.config;
        let pause_reason = if !is_owner && config.paused {
            reason::GLOBAL_PAUSED
        } else if account.status == AccountStatus::Paused && risk_increasing(args.kind) {
            reason::ACCOUNT_PAUSED
        } else {
            reason::NONE
        };

        // Mandate checks, recomputed from the chain's limits.
        let mandate = &ctx.accounts.mandate;
        let facts = TradeFacts {
            venue_id: args.venue_id,
            market: args.market_id,
            data_slot: args.data_slot,
            now_slot: clock.slot,
            ledger_loss_usd: ledger.realized_loss_usd,
        };
        let verdict = evaluate_trade(mandate, &config.caps, &args.checks, &facts);

        let forced = if actor_reason != reason::NONE {
            actor_reason
        } else if pause_reason != reason::NONE {
            pause_reason
        } else {
            reason::NONE
        };
        let (decision, reason_code) = if forced != reason::NONE {
            (Decision::Reject, forced)
        } else if verdict.first_failure != reason::NONE {
            // Invariant 1: the caller may not call this an Allow.
            require!(
                args.decision != Decision::Allow,
                MarkovError::AllowViolatesMandate
            );
            let d = if args.decision == Decision::Skip {
                Decision::Reject
            } else {
                args.decision
            };
            (
                d,
                if args.reason_code == reason::NONE {
                    verdict.first_failure
                } else {
                    args.reason_code
                },
            )
        } else {
            (args.decision, args.reason_code)
        };

        // Invariant 2: adjacency.
        let sysvar = ctx.accounts.instructions.to_account_info();
        if decision == Decision::Allow {
            require!(
                mandate.venue_allowed(args.venue_id),
                MarkovError::UnknownVenue
            );
            let venue_program = config
                .venue_program(args.venue_id)
                .ok_or(MarkovError::UnknownVenue)?;
            introspection::require_next_is(&sysvar, &venue_program)?;
        } else {
            let mut forbidden: Vec<Pubkey> = config.venue_programs.to_vec();
            forbidden.push(config.invest_collect_program);
            forbidden.push(config.invest_swap_program);
            introspection::require_no_venue_next(&sysvar, &forbidden)?;
        }

        // Only an Allow changes anything but the receipt.
        if decision == Decision::Allow {
            ledger.actions_count = ledger.actions_count.saturating_add(1);
            ledger.gross_notional_usd = ledger
                .gross_notional_usd
                .checked_add(args.notional_usd)
                .ok_or(MarkovError::Math)?;
            if let Some(perm) = ctx.accounts.permission.as_mut() {
                if !is_owner {
                    perm.spent_today_usd = perm
                        .spent_today_usd
                        .checked_add(args.notional_usd)
                        .ok_or(MarkovError::Math)?;
                }
            }
        }

        let r = &mut ctx.accounts.receipt;
        r.account = account.key();
        r.request_id = args.request_id;
        r.actor = signer;
        r.kind = args.kind;
        r.decision = decision;
        r.reason_code = reason_code;
        r.mandate_version = account.active_mandate_version;
        r.invest_mandate_version = account.active_invest_mandate_version;
        r.data_slot = args.data_slot;
        r.venue_id = args.venue_id;
        r.market_id = args.market_id;
        r.checks = verdict.checks.clone();
        r.route_snapshot_hash = args.route_snapshot_hash;
        r.tx_signature_hint = [0u8; 32];
        r.pre_state_hash = args.pre_state_hash;
        r.post_state_hash = [0u8; 32];
        r.finalized = false;
        r.created_slot = clock.slot;
        r.created_ts = now;
        r.bump = ctx.bumps.receipt;

        emit_cpi!(ReceiptEmitted {
            account: account.key(),
            request_id: args.request_id,
            actor: signer,
            kind: args.kind,
            decision,
            reason_code,
            mandate_version: account.active_mandate_version,
            invest_mandate_version: account.active_invest_mandate_version,
            data_slot: args.data_slot,
            venue_id: args.venue_id,
            market_id: args.market_id,
            checks: verdict.checks,
            route_snapshot_hash: args.route_snapshot_hash,
            pre_state_hash: args.pre_state_hash,
            created_slot: clock.slot,
            created_ts: now,
        });
        Ok(())
    }

    /// Instruction *i+2*: the post-state hash after the venue leg. Adds two
    /// fields once and nothing else (invariant 8).
    pub fn finalize_decision(
        ctx: Context<FinalizeDecision>,
        post_state_hash: [u8; 32],
        tx_signature_hint: [u8; 32],
    ) -> Result<()> {
        let r = &mut ctx.accounts.receipt;
        require!(!r.finalized, MarkovError::AlreadyFinalized);
        let signer = ctx.accounts.signer.key();
        require!(
            signer == r.actor || signer == ctx.accounts.account.owner,
            MarkovError::NotOwner
        );
        introspection::require_prev_is_not_self(&ctx.accounts.instructions.to_account_info())?;
        r.post_state_hash = post_state_hash;
        r.tx_signature_hint = tx_signature_hint;
        r.finalized = true;
        emit_cpi!(ReceiptFinalized {
            account: ctx.accounts.account.key(),
            request_id: r.request_id,
            post_state_hash,
            tx_signature_hint,
            slot: Clock::get()?.slot,
        });
        Ok(())
    }

    /// An Invest cycle that did not execute: the receipt says why, and no
    /// venue instruction may follow.
    pub fn record_skip(ctx: Context<RecordSkip>, args: SkipArgs) -> Result<()> {
        require!(args.checks.len() <= MAX_CHECKS, MarkovError::TooManyChecks);
        let clock = Clock::get()?;
        let signer = ctx.accounts.signer.key();
        let account = &ctx.accounts.account;
        require!(
            account.status != AccountStatus::Closed,
            MarkovError::AccountClosed
        );
        let actor_reason = actor_gate(
            &signer,
            &account.owner,
            &mut ctx.accounts.permission,
            ReceiptKind::InvestSkip,
            0,
            clock.unix_timestamp,
            clock.slot,
        )?;
        let reason_code = if actor_reason != reason::NONE {
            actor_reason
        } else {
            args.reason_code
        };
        let config = &ctx.accounts.config;
        let mut forbidden: Vec<Pubkey> = config.venue_programs.to_vec();
        forbidden.push(config.invest_collect_program);
        forbidden.push(config.invest_swap_program);
        introspection::require_no_venue_next(
            &ctx.accounts.instructions.to_account_info(),
            &forbidden,
        )?;

        let r = &mut ctx.accounts.receipt;
        r.account = account.key();
        r.request_id = args.request_id;
        r.actor = signer;
        r.kind = ReceiptKind::InvestSkip;
        r.decision = Decision::Skip;
        r.reason_code = reason_code;
        r.mandate_version = account.active_mandate_version;
        r.invest_mandate_version = account.active_invest_mandate_version;
        r.data_slot = args.data_slot;
        r.venue_id = 0;
        r.market_id = [0u8; 16];
        r.checks = args.checks.clone();
        r.route_snapshot_hash = [0u8; 32];
        r.tx_signature_hint = [0u8; 32];
        r.pre_state_hash = [0u8; 32];
        r.post_state_hash = [0u8; 32];
        r.finalized = true;
        r.created_slot = clock.slot;
        r.created_ts = clock.unix_timestamp;
        r.bump = ctx.bumps.receipt;
        emit_cpi!(ReceiptEmitted {
            account: account.key(),
            request_id: args.request_id,
            actor: signer,
            kind: ReceiptKind::InvestSkip,
            decision: Decision::Skip,
            reason_code,
            mandate_version: account.active_mandate_version,
            invest_mandate_version: account.active_invest_mandate_version,
            data_slot: args.data_slot,
            venue_id: 0,
            market_id: [0u8; 16],
            checks: args.checks,
            route_snapshot_hash: [0u8; 32],
            pre_state_hash: [0u8; 32],
            created_slot: clock.slot,
            created_ts: clock.unix_timestamp,
        });
        Ok(())
    }

    /// The keeper's Allow path for Invest. Validates the invest mandate from
    /// the chain (the reserve floor from the owner's token account, not a
    /// supplied number), charges the month, writes the receipt, and requires
    /// the delegation `collect` at *i+1* and the swap at *i+2*.
    pub fn invest_execute(ctx: Context<InvestExecute>, args: InvestArgs) -> Result<()> {
        require!(args.checks.len() <= MAX_CHECKS, MarkovError::TooManyChecks);
        let clock = Clock::get()?;
        let now = clock.unix_timestamp;
        let signer = ctx.accounts.signer.key();
        let account = &ctx.accounts.account;
        require!(
            account.status == AccountStatus::Active,
            MarkovError::AccountClosed
        );
        require!(
            account.active_invest_mandate_version > 0,
            MarkovError::NoActiveInvestMandate
        );
        let config = &mut ctx.accounts.config;
        require!(!config.paused, MarkovError::AllowWhilePaused);
        config.roll_invest_day(now);

        let actor_reason = actor_gate(
            &signer,
            &account.owner,
            &mut ctx.accounts.permission,
            ReceiptKind::InvestExecute,
            args.usdc_amount,
            now,
            clock.slot,
        )?;
        require!(
            actor_reason == reason::NONE,
            MarkovError::AllowViolatesMandate
        );

        let m = &mut ctx.accounts.invest_mandate;
        require!(!m.paused, MarkovError::AllowWhilePaused);
        // Invariant 5: the month rolls once, then spend only grows.
        let epoch = month_epoch(now);
        if epoch != m.month_epoch {
            m.month_epoch = epoch;
            m.spent_this_month_usd = 0;
        }
        let reserve = &ctx.accounts.reserve_token_account;
        require_keys_eq!(
            reserve.owner,
            account.owner,
            MarkovError::WrongReserveAccount
        );
        require_keys_eq!(
            reserve.mint,
            config.usdc_mint,
            MarkovError::WrongReserveAccount
        );
        if m.execution_mode == InvestExecutionMode::ReferenceSafe {
            require!(args.official_price > 0, MarkovError::ReferenceRequired);
        }
        let facts = InvestFacts {
            mint_allowed: m.mint_allowed(&args.mint),
            usdc_amount: args.usdc_amount,
            reserve_balance_usd: reserve.amount,
            quote_cost_bps: args.quote_cost_bps,
            market_status: args.market_status,
            official_price: args.official_price,
            quote_price: args.quote_price,
            seconds_into_day: now.rem_euclid(SECONDS_PER_DAY) as u32,
            weekday_bit: weekday_bit(now),
            global_spent_today_usd: config.invest_spent_today_usd,
            spent_this_month_usd: m.spent_this_month_usd,
        };
        let verdict = evaluate_invest(m, &config.caps, &facts, &args.checks);
        require!(
            verdict.first_failure == reason::NONE,
            MarkovError::AllowViolatesMandate
        );

        let sysvar = ctx.accounts.instructions.to_account_info();
        require_keys_neq!(
            config.invest_collect_program,
            Pubkey::default(),
            MarkovError::UnknownVenue
        );
        require_keys_neq!(
            config.invest_swap_program,
            Pubkey::default(),
            MarkovError::UnknownVenue
        );
        introspection::require_next_is(&sysvar, &config.invest_collect_program)?;
        let second = introspection::adjacent_program(&sysvar, 2)?
            .ok_or(MarkovError::MissingAdjacentInstruction)?;
        require_keys_eq!(
            second,
            config.invest_swap_program,
            MarkovError::AdjacentProgramNotAllowed
        );

        m.spent_this_month_usd = m
            .spent_this_month_usd
            .checked_add(args.usdc_amount)
            .ok_or(MarkovError::Math)?;
        config.invest_spent_today_usd = config
            .invest_spent_today_usd
            .checked_add(args.usdc_amount)
            .ok_or(MarkovError::Math)?;
        if let Some(perm) = ctx.accounts.permission.as_mut() {
            if signer != account.owner {
                perm.spent_today_usd = perm
                    .spent_today_usd
                    .checked_add(args.usdc_amount)
                    .ok_or(MarkovError::Math)?;
            }
        }

        let r = &mut ctx.accounts.receipt;
        r.account = account.key();
        r.request_id = args.request_id;
        r.actor = signer;
        r.kind = ReceiptKind::InvestExecute;
        r.decision = Decision::Allow;
        r.reason_code = reason::NONE;
        r.mandate_version = account.active_mandate_version;
        r.invest_mandate_version = account.active_invest_mandate_version;
        r.data_slot = args.data_slot;
        r.venue_id = 0;
        r.market_id = [0u8; 16];
        r.checks = verdict.checks.clone();
        r.route_snapshot_hash = args.route_snapshot_hash;
        r.tx_signature_hint = [0u8; 32];
        r.pre_state_hash = args.pre_state_hash;
        r.post_state_hash = [0u8; 32];
        r.finalized = false;
        r.created_slot = clock.slot;
        r.created_ts = now;
        r.bump = ctx.bumps.receipt;
        emit_cpi!(ReceiptEmitted {
            account: account.key(),
            request_id: args.request_id,
            actor: signer,
            kind: ReceiptKind::InvestExecute,
            decision: Decision::Allow,
            reason_code: reason::NONE,
            mandate_version: account.active_mandate_version,
            invest_mandate_version: account.active_invest_mandate_version,
            data_slot: args.data_slot,
            venue_id: 0,
            market_id: [0u8; 16],
            checks: verdict.checks,
            route_snapshot_hash: args.route_snapshot_hash,
            pre_state_hash: args.pre_state_hash,
            created_slot: clock.slot,
            created_ts: now,
        });
        Ok(())
    }

    /// Rent back to the owner once a receipt is 30 days old; the event remains.
    pub fn close_receipt(ctx: Context<CloseReceipt>) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;
        require!(
            now >= ctx
                .accounts
                .receipt
                .created_ts
                .saturating_add(RECEIPT_CLOSE_AFTER_SECS),
            MarkovError::ReceiptTooYoung
        );
        Ok(())
    }
}

// ---- contexts ----------------------------------------------------------------

#[derive(Accounts)]
pub struct InitializeConfig<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(init, payer = admin, space = 8 + GlobalConfig::INIT_SPACE, seeds = [GlobalConfig::SEED], bump)]
    pub config: Box<Account<'info, GlobalConfig>>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AdminOnly<'info> {
    pub admin: Signer<'info>,
    #[account(mut, seeds = [GlobalConfig::SEED], bump = config.bump, has_one = admin @ MarkovError::NotAdmin)]
    pub config: Box<Account<'info, GlobalConfig>>,
}

#[derive(Accounts)]
pub struct CreateAccount<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(init, payer = owner, space = 8 + MarkovAccount::INIT_SPACE, seeds = [MarkovAccount::SEED, owner.key().as_ref()], bump)]
    pub account: Box<Account<'info, MarkovAccount>>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct OwnerOnly<'info> {
    pub owner: Signer<'info>,
    #[account(mut, seeds = [MarkovAccount::SEED, owner.key().as_ref()], bump = account.bump, has_one = owner @ MarkovError::NotOwner)]
    pub account: Box<Account<'info, MarkovAccount>>,
}

#[derive(Accounts)]
pub struct OwnerInvest<'info> {
    pub owner: Signer<'info>,
    #[account(seeds = [MarkovAccount::SEED, owner.key().as_ref()], bump = account.bump, has_one = owner @ MarkovError::NotOwner)]
    pub account: Box<Account<'info, MarkovAccount>>,
    #[account(
        mut,
        seeds = [InvestMandate::SEED, account.key().as_ref(), &account.active_invest_mandate_version.to_le_bytes()],
        bump = invest_mandate.bump,
        has_one = account @ MarkovError::NoActiveInvestMandate
    )]
    pub invest_mandate: Box<Account<'info, InvestMandate>>,
}

#[derive(Accounts)]
#[instruction(version: u32)]
#[event_cpi]
pub struct SetMandate<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(mut, seeds = [MarkovAccount::SEED, owner.key().as_ref()], bump = account.bump, has_one = owner @ MarkovError::NotOwner)]
    pub account: Box<Account<'info, MarkovAccount>>,
    #[account(
        init,
        payer = owner,
        space = 8 + Mandate::INIT_SPACE,
        seeds = [Mandate::SEED, account.key().as_ref(), &version.to_le_bytes()],
        bump
    )]
    pub mandate: Box<Account<'info, Mandate>>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(version: u32)]
#[event_cpi]
pub struct SetInvestMandate<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(mut, seeds = [MarkovAccount::SEED, owner.key().as_ref()], bump = account.bump, has_one = owner @ MarkovError::NotOwner)]
    pub account: Box<Account<'info, MarkovAccount>>,
    #[account(
        init,
        payer = owner,
        space = 8 + InvestMandate::INIT_SPACE,
        seeds = [InvestMandate::SEED, account.key().as_ref(), &version.to_le_bytes()],
        bump
    )]
    pub invest_mandate: Box<Account<'info, InvestMandate>>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(actor: Pubkey)]
pub struct SetPermission<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(seeds = [MarkovAccount::SEED, owner.key().as_ref()], bump = account.bump, has_one = owner @ MarkovError::NotOwner)]
    pub account: Box<Account<'info, MarkovAccount>>,
    #[account(
        init_if_needed,
        payer = owner,
        space = 8 + Permission::INIT_SPACE,
        seeds = [Permission::SEED, account.key().as_ref(), actor.as_ref()],
        bump
    )]
    pub permission: Box<Account<'info, Permission>>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RevokePermission<'info> {
    pub owner: Signer<'info>,
    #[account(seeds = [MarkovAccount::SEED, owner.key().as_ref()], bump = account.bump, has_one = owner @ MarkovError::NotOwner)]
    pub account: Box<Account<'info, MarkovAccount>>,
    #[account(
        mut,
        seeds = [Permission::SEED, account.key().as_ref(), permission.actor.as_ref()],
        bump = permission.bump,
        has_one = account @ MarkovError::NoPermission
    )]
    pub permission: Box<Account<'info, Permission>>,
}

#[derive(Accounts)]
#[instruction(args: DecisionArgs)]
#[event_cpi]
pub struct RecordDecision<'info> {
    /// The owner, or an actor with a permission on this account.
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(seeds = [GlobalConfig::SEED], bump = config.bump)]
    pub config: Box<Account<'info, GlobalConfig>>,
    #[account(mut, seeds = [MarkovAccount::SEED, account.owner.as_ref()], bump = account.bump)]
    pub account: Box<Account<'info, MarkovAccount>>,
    #[account(
        seeds = [Mandate::SEED, account.key().as_ref(), &account.active_mandate_version.to_le_bytes()],
        bump = mandate.bump,
        has_one = account @ MarkovError::NoActiveMandate
    )]
    pub mandate: Box<Account<'info, Mandate>>,
    /// Required for a non-owner signer; ignored for the owner.
    #[account(mut, seeds = [Permission::SEED, account.key().as_ref(), signer.key().as_ref()], bump = permission.bump)]
    pub permission: Option<Box<Account<'info, Permission>>>,
    #[account(
        init,
        payer = signer,
        space = 8 + ActionReceipt::INIT_SPACE,
        seeds = [ActionReceipt::SEED, account.key().as_ref(), &args.request_id.to_le_bytes()],
        bump
    )]
    pub receipt: Box<Account<'info, ActionReceipt>>,
    #[account(
        init_if_needed,
        payer = signer,
        space = 8 + DailyLedger::INIT_SPACE,
        seeds = [DailyLedger::SEED, account.key().as_ref(), &args.day_epoch.to_le_bytes()],
        bump
    )]
    pub ledger: Box<Account<'info, DailyLedger>>,
    /// CHECK: address-checked; read through `solana_instructions_sysvar`.
    #[account(address = solana_instructions_sysvar::ID @ MarkovError::BadInstructionIndex)]
    pub instructions: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[event_cpi]
pub struct FinalizeDecision<'info> {
    pub signer: Signer<'info>,
    #[account(seeds = [MarkovAccount::SEED, account.owner.as_ref()], bump = account.bump)]
    pub account: Box<Account<'info, MarkovAccount>>,
    #[account(
        mut,
        seeds = [ActionReceipt::SEED, account.key().as_ref(), &receipt.request_id.to_le_bytes()],
        bump = receipt.bump,
        has_one = account @ MarkovError::NotOwner
    )]
    pub receipt: Box<Account<'info, ActionReceipt>>,
    /// CHECK: address-checked; read through `solana_instructions_sysvar`.
    #[account(address = solana_instructions_sysvar::ID @ MarkovError::BadInstructionIndex)]
    pub instructions: UncheckedAccount<'info>,
}

#[derive(Accounts)]
#[instruction(args: SkipArgs)]
#[event_cpi]
pub struct RecordSkip<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(seeds = [GlobalConfig::SEED], bump = config.bump)]
    pub config: Box<Account<'info, GlobalConfig>>,
    #[account(seeds = [MarkovAccount::SEED, account.owner.as_ref()], bump = account.bump)]
    pub account: Box<Account<'info, MarkovAccount>>,
    #[account(mut, seeds = [Permission::SEED, account.key().as_ref(), signer.key().as_ref()], bump = permission.bump)]
    pub permission: Option<Box<Account<'info, Permission>>>,
    #[account(
        init,
        payer = signer,
        space = 8 + ActionReceipt::INIT_SPACE,
        seeds = [ActionReceipt::SEED, account.key().as_ref(), &args.request_id.to_le_bytes()],
        bump
    )]
    pub receipt: Box<Account<'info, ActionReceipt>>,
    /// CHECK: address-checked; read through `solana_instructions_sysvar`.
    #[account(address = solana_instructions_sysvar::ID @ MarkovError::BadInstructionIndex)]
    pub instructions: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(args: InvestArgs)]
#[event_cpi]
pub struct InvestExecute<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(mut, seeds = [GlobalConfig::SEED], bump = config.bump)]
    pub config: Box<Account<'info, GlobalConfig>>,
    #[account(seeds = [MarkovAccount::SEED, account.owner.as_ref()], bump = account.bump)]
    pub account: Box<Account<'info, MarkovAccount>>,
    #[account(
        mut,
        seeds = [InvestMandate::SEED, account.key().as_ref(), &account.active_invest_mandate_version.to_le_bytes()],
        bump = invest_mandate.bump,
        has_one = account @ MarkovError::NoActiveInvestMandate
    )]
    pub invest_mandate: Box<Account<'info, InvestMandate>>,
    #[account(mut, seeds = [Permission::SEED, account.key().as_ref(), signer.key().as_ref()], bump = permission.bump)]
    pub permission: Option<Box<Account<'info, Permission>>>,
    /// The owner's USDC account; its balance *is* the reserve proof.
    pub reserve_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        init,
        payer = signer,
        space = 8 + ActionReceipt::INIT_SPACE,
        seeds = [ActionReceipt::SEED, account.key().as_ref(), &args.request_id.to_le_bytes()],
        bump
    )]
    pub receipt: Box<Account<'info, ActionReceipt>>,
    /// CHECK: address-checked; read through `solana_instructions_sysvar`.
    #[account(address = solana_instructions_sysvar::ID @ MarkovError::BadInstructionIndex)]
    pub instructions: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CloseReceipt<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(seeds = [MarkovAccount::SEED, owner.key().as_ref()], bump = account.bump, has_one = owner @ MarkovError::NotOwner)]
    pub account: Box<Account<'info, MarkovAccount>>,
    #[account(
        mut,
        close = owner,
        seeds = [ActionReceipt::SEED, account.key().as_ref(), &receipt.request_id.to_le_bytes()],
        bump = receipt.bump,
        has_one = account @ MarkovError::NotOwner
    )]
    pub receipt: Box<Account<'info, ActionReceipt>>,
}
