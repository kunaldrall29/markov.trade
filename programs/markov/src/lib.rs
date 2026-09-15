//! Markov program v0 — account, mandate, permissions, receipts.
//!
//! Pacifica (`venue_id = 1`) is off-chain matching: an Allow receipt does not
//! require an adjacent venue instruction. Drift / Phoenix / Jupiter do.

#![allow(unexpected_cfgs)]
#![forbid(unsafe_code)]

use anchor_lang::prelude::*;
use solana_instructions_sysvar::{
    load_current_index_checked, load_instruction_at_checked, ID as INSTRUCTIONS_ID,
};
use solana_sha256_hasher::hashv;

pub mod checks;
pub mod errors;
pub mod events;
pub mod ids;
pub mod state;

use crate::checks::{evaluate_mandate, Observed};
use crate::errors::MarkovError;
use crate::events::ReceiptEmitted;
use crate::ids::*;
use crate::state::*;

declare_id!("37rW4ETzh8o7ebWFnrJRKt7iCUvWdxsv37vPENjAYz1J");

#[program]
pub mod markov {
    use super::*;

    pub fn initialize_config(
        ctx: Context<InitializeConfig>,
        caps: Caps,
        venue_programs: [Pubkey; MAX_VENUES],
    ) -> Result<()> {
        require!(caps.per_position_leverage_bps > 0, MarkovError::InvalidCaps);
        let c = &mut ctx.accounts.config;
        c.admin = ctx.accounts.admin.key();
        c.paused = false;
        c.caps = caps;
        c.fee_bps = 0;
        c.version = 1;
        c.bump = ctx.bumps.config;
        c.venue_programs = venue_programs;
        c.reserved = [0; 32];
        Ok(())
    }

    pub fn set_global_caps(ctx: Context<AdminOnly>, caps: Caps) -> Result<()> {
        require!(caps.per_position_leverage_bps > 0, MarkovError::InvalidCaps);
        ctx.accounts.config.caps = caps;
        Ok(())
    }

    pub fn set_venue_programs(
        ctx: Context<AdminOnly>,
        venue_programs: [Pubkey; MAX_VENUES],
    ) -> Result<()> {
        ctx.accounts.config.venue_programs = venue_programs;
        Ok(())
    }

    pub fn set_global_pause(ctx: Context<AdminOnly>, paused: bool) -> Result<()> {
        ctx.accounts.config.paused = paused;
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
        a.reserved = [0; 64];
        Ok(())
    }

    pub fn set_mandate(
        ctx: Context<SetMandate>,
        version: u32,
        max_leverage_bps: u32,
        max_notional_usd: u64,
        min_safety_buffer_bps: u32,
        max_daily_loss_usd: u64,
        approved_markets: Vec<[u8; 16]>,
        tier_caps: [u32; 5],
        allowed_venues: u32,
    ) -> Result<()> {
        require!(
            approved_markets.len() <= MAX_MARKETS,
            MarkovError::TooManyMarkets
        );
        require!(
            version == ctx.accounts.account.active_mandate_version.saturating_add(1),
            MarkovError::MandateVersion
        );
        require!(max_leverage_bps > 0, MarkovError::InvalidCaps);

        let hash = hash_mandate(
            version,
            max_leverage_bps,
            max_notional_usd,
            min_safety_buffer_bps,
            max_daily_loss_usd,
            &approved_markets,
            &tier_caps,
            allowed_venues,
        );

        let m = &mut ctx.accounts.mandate;
        m.account = ctx.accounts.account.key();
        m.version = version;
        m.hash = hash;
        m.max_leverage_bps = max_leverage_bps;
        m.max_notional_usd = max_notional_usd;
        m.min_safety_buffer_bps = min_safety_buffer_bps;
        m.max_daily_loss_usd = max_daily_loss_usd;
        m.approved_markets = approved_markets;
        m.tier_caps = tier_caps;
        m.allowed_venues = allowed_venues;
        m.activated_slot = Clock::get()?.slot;
        m.activated_by = ctx.accounts.owner.key();
        m.bump = ctx.bumps.mandate;

        ctx.accounts.account.active_mandate_version = version;
        Ok(())
    }

    pub fn set_invest_mandate(
        ctx: Context<SetInvestMandate>,
        version: u32,
        allowlist: Vec<Pubkey>,
        per_period_budget_usd: u64,
        period_seconds: u32,
        monthly_ceiling_usd: u64,
        reserve_floor_usd: u64,
        max_cost_bps: u32,
        single_asset_cap_bps: u32,
        execution_mode: InvestExecMode,
        window_start_utc: u16,
        window_end_utc: u16,
        days_mask: u8,
    ) -> Result<()> {
        require!(allowlist.len() <= MAX_MINTS, MarkovError::TooManyMints);
        require!(
            version
                == ctx
                    .accounts
                    .account
                    .active_invest_mandate_version
                    .saturating_add(1),
            MarkovError::MandateVersion
        );

        let hash = hash_invest(
            version,
            &allowlist,
            per_period_budget_usd,
            period_seconds,
            monthly_ceiling_usd,
            reserve_floor_usd,
            max_cost_bps,
            single_asset_cap_bps,
            execution_mode as u8,
            window_start_utc,
            window_end_utc,
            days_mask,
        );

        let m = &mut ctx.accounts.invest;
        m.account = ctx.accounts.account.key();
        m.version = version;
        m.hash = hash;
        m.allowlist = allowlist;
        m.per_period_budget_usd = per_period_budget_usd;
        m.period_seconds = period_seconds;
        m.monthly_ceiling_usd = monthly_ceiling_usd;
        m.reserve_floor_usd = reserve_floor_usd;
        m.max_cost_bps = max_cost_bps;
        m.single_asset_cap_bps = single_asset_cap_bps;
        m.execution_mode = execution_mode;
        m.window_start_utc = window_start_utc;
        m.window_end_utc = window_end_utc;
        m.days_mask = days_mask;
        m.paused = false;
        m.spent_this_month_usd = 0;
        m.month_epoch = month_epoch(Clock::get()?.unix_timestamp);
        m.activated_slot = Clock::get()?.slot;
        m.bump = ctx.bumps.invest;

        ctx.accounts.account.active_invest_mandate_version = version;
        Ok(())
    }

    pub fn pause(ctx: Context<OwnerAccount>) -> Result<()> {
        ctx.accounts.account.status = AccountStatus::Paused;
        Ok(())
    }

    pub fn unpause(ctx: Context<OwnerAccount>) -> Result<()> {
        ctx.accounts.account.status = AccountStatus::Active;
        Ok(())
    }

    pub fn set_permission(
        ctx: Context<SetPermission>,
        kind: ActorKind,
        scopes: u32,
        per_action_cap_usd: u64,
        daily_cap_usd: u64,
        expires_slot: u64,
    ) -> Result<()> {
        // MCP clients cannot hold invest:execute.
        if matches!(kind, ActorKind::McpClient) {
            require!(
                scopes & SCOPE_INVEST_EXECUTE == 0,
                MarkovError::PermissionDenied
            );
        }
        let p = &mut ctx.accounts.permission;
        p.account = ctx.accounts.account.key();
        p.actor = ctx.accounts.actor.key();
        p.kind = kind;
        p.scopes = scopes;
        p.per_action_cap_usd = per_action_cap_usd;
        p.daily_cap_usd = daily_cap_usd;
        p.spent_today_usd = 0;
        p.day_epoch = day_epoch(Clock::get()?.unix_timestamp);
        p.expires_slot = expires_slot;
        p.revoked = false;
        p.bump = ctx.bumps.permission;
        Ok(())
    }

    pub fn revoke_permission(ctx: Context<RevokePermission>) -> Result<()> {
        ctx.accounts.permission.revoked = true;
        Ok(())
    }

    pub fn record_decision(
        ctx: Context<RecordDecision>,
        request_id: u128,
        day: u32,
        kind: ActionKind,
        decision: Decision,
        reason_code: u16,
        data_slot: u64,
        venue_id: u8,
        market_id: [u8; 16],
        observed: ObservedIx,
        route_hash: [u8; 32],
        pre_hash: [u8; 32],
    ) -> Result<()> {
        let clock = Clock::get()?;
        require!(day == day_epoch(clock.unix_timestamp), MarkovError::BadAccounts);
        let cfg = &ctx.accounts.config;
        let account = &ctx.accounts.account;
        let mandate = &ctx.accounts.mandate;

        require!(
            mandate.version == account.active_mandate_version,
            MarkovError::MandateVersion
        );

        let actor = ctx.accounts.actor.key();
        let is_owner = actor == account.owner;
        if !is_owner {
            let perm = ctx
                .accounts
                .permission
                .as_ref()
                .ok_or(MarkovError::PermissionDenied)?;
            require!(!perm.revoked, MarkovError::PermissionDenied);
            require!(
                perm.expires_slot == 0 || clock.slot <= perm.expires_slot,
                MarkovError::PermissionDenied
            );
            let need = match kind {
                ActionKind::TradeReduce | ActionKind::TradeClose => SCOPE_TRADE_REDUCE,
                _ => SCOPE_TRADE_REQUEST,
            };
            require!(perm.scopes & need != 0, MarkovError::PermissionDenied);
        }

        let obs = Observed {
            venue_id,
            market_id,
            data_age_ms: observed.data_age_ms,
            freshness_limit_ms: observed.freshness_limit_ms,
            projected_leverage_bps: observed.projected_leverage_bps,
            projected_notional_usd: observed.projected_notional_usd,
            safety_buffer_bps: observed.safety_buffer_bps,
            daily_loss_usd: observed.daily_loss_usd,
            slippage_bps: observed.slippage_bps,
            slippage_limit_bps: observed.slippage_limit_bps,
        };
        let (computed_reason, checks, mandate_ok) = evaluate_mandate(mandate, &obs);

        let mut decision = decision;
        let mut reason = reason_code;

        if account.status != AccountStatus::Active {
            decision = Decision::Reject;
            reason = REASON_ACCOUNT_PAUSED;
        } else if cfg.paused && matches!(kind, ActionKind::TradeOpen | ActionKind::InvestExecute)
        {
            // Global pause blocks keeper / risk-increasing automation; owner-signed
            // reduce/close still works. TradeOpen is risk-increasing.
            if !is_owner || matches!(kind, ActionKind::InvestExecute) {
                decision = Decision::Reject;
                reason = REASON_GLOBAL_PAUSED;
            }
        }

        if matches!(decision, Decision::Allow) && !mandate_ok {
            return err!(MarkovError::AllowWouldViolate);
        }
        if matches!(decision, Decision::Allow) && mandate_ok && reason == 0 {
            reason = REASON_CONFIRMED;
        }
        if matches!(decision, Decision::Reject) && reason == 0 {
            reason = if computed_reason == 0 {
                REASON_MARKET_NOT_ALLOWED
            } else {
                computed_reason
            };
        }

        if matches!(decision, Decision::Allow) {
            require_adjacent_venue(
                &ctx.accounts.instructions,
                venue_id,
                &cfg.venue_programs,
            )?;
        }

        let receipt = &mut ctx.accounts.receipt;
        receipt.account = account.key();
        receipt.request_id = request_id;
        receipt.actor = actor;
        receipt.kind = kind;
        receipt.decision = decision;
        receipt.reason_code = reason;
        receipt.mandate_version = mandate.version;
        receipt.invest_mandate_version = account.active_invest_mandate_version;
        receipt.data_slot = data_slot;
        receipt.venue_id = venue_id;
        receipt.market_id = market_id;
        receipt.checks = checks.clone();
        receipt.route_snapshot_hash = route_hash;
        receipt.tx_signature_hint = [0; 32];
        receipt.pre_state_hash = pre_hash;
        receipt.post_state_hash = [0; 32];
        receipt.created_slot = clock.slot;
        receipt.finalized = false;
        receipt.bump = ctx.bumps.receipt;

        let day = day_epoch(clock.unix_timestamp);
        let ledger = &mut ctx.accounts.daily_ledger;
        if ledger.actions_count == 0 && ledger.day_epoch == 0 {
            ledger.account = account.key();
            ledger.day_epoch = day;
            ledger.bump = ctx.bumps.daily_ledger;
        }
        if ledger.day_epoch != day {
            ledger.day_epoch = day;
            ledger.realized_loss_usd = 0;
            ledger.gross_notional_usd = 0;
            ledger.actions_count = 0;
        }
        ledger.actions_count = ledger.actions_count.saturating_add(1);
        if matches!(decision, Decision::Allow) {
            ledger.gross_notional_usd = ledger
                .gross_notional_usd
                .saturating_add(observed.projected_notional_usd.max(0) as u64);
        }

        emit!(ReceiptEmitted {
            account: account.key(),
            request_id,
            actor,
            kind,
            decision,
            reason_code: reason,
            mandate_version: mandate.version,
            invest_mandate_version: account.active_invest_mandate_version,
            data_slot,
            venue_id,
            market_id,
            checks,
            route_snapshot_hash: route_hash,
            created_slot: clock.slot,
        });
        Ok(())
    }

    pub fn finalize_decision(
        ctx: Context<FinalizeDecision>,
        post_hash: [u8; 32],
        tx_hint: [u8; 32],
    ) -> Result<()> {
        let r = &mut ctx.accounts.receipt;
        require!(!r.finalized, MarkovError::ReceiptImmutable);
        r.post_state_hash = post_hash;
        r.tx_signature_hint = tx_hint;
        r.finalized = true;
        Ok(())
    }

    pub fn record_skip(
        ctx: Context<RecordSkip>,
        request_id: u128,
        reason_code: u16,
        data_slot: u64,
        mint: Pubkey,
        checks: Vec<Check>,
    ) -> Result<()> {
        require_keeper_invest(&ctx)?;
        let clock = Clock::get()?;
        let account = &ctx.accounts.account;
        let r = &mut ctx.accounts.receipt;
        r.account = account.key();
        r.request_id = request_id;
        r.actor = ctx.accounts.actor.key();
        r.kind = ActionKind::InvestSkip;
        r.decision = Decision::Skip;
        r.reason_code = reason_code;
        r.mandate_version = account.active_mandate_version;
        r.invest_mandate_version = account.active_invest_mandate_version;
        r.data_slot = data_slot;
        r.venue_id = VENUE_JUPITER;
        r.market_id = pubkey_as_market(&mint);
        r.checks = checks.clone();
        r.route_snapshot_hash = [0; 32];
        r.tx_signature_hint = [0; 32];
        r.pre_state_hash = [0; 32];
        r.post_state_hash = [0; 32];
        r.created_slot = clock.slot;
        r.finalized = true;
        r.bump = ctx.bumps.receipt;

        emit!(ReceiptEmitted {
            account: account.key(),
            request_id,
            actor: ctx.accounts.actor.key(),
            kind: ActionKind::InvestSkip,
            decision: Decision::Skip,
            reason_code,
            mandate_version: account.active_mandate_version,
            invest_mandate_version: account.active_invest_mandate_version,
            data_slot,
            venue_id: VENUE_JUPITER,
            market_id: pubkey_as_market(&mint),
            checks,
            route_snapshot_hash: [0; 32],
            created_slot: clock.slot,
        });
        Ok(())
    }

    pub fn invest_execute(
        ctx: Context<InvestExecute>,
        request_id: u128,
        mint: Pubkey,
        usdc_amount: u64,
        quote_cost_bps: u32,
        data_slot: u64,
        market_status_open: bool,
        reserve_usd: u64,
        checks: Vec<Check>,
    ) -> Result<()> {
        require_keeper_invest_exec(&ctx)?;
        let clock = Clock::get()?;
        let account = &ctx.accounts.account;
        require!(
            account.status == AccountStatus::Active,
            MarkovError::AccountPaused
        );
        require!(!ctx.accounts.config.paused, MarkovError::GlobalPaused);

        let invest = &mut ctx.accounts.invest;
        require!(!invest.paused, MarkovError::InvestPaused);
        require!(
            invest.version == account.active_invest_mandate_version,
            MarkovError::MandateVersion
        );
        require!(
            invest.allowlist.iter().any(|m| *m == mint),
            MarkovError::AssetNotAllowlisted
        );
        require!(
            quote_cost_bps <= invest.max_cost_bps,
            MarkovError::CostLimit
        );

        match invest.execution_mode {
            InvestExecMode::MarketHoursOnly => {
                require!(market_status_open, MarkovError::InvestBudget);
            }
            InvestExecMode::AlwaysOn | InvestExecMode::ReferenceSafe => {}
        }
        require!(
            reserve_usd >= invest.reserve_floor_usd,
            MarkovError::InvestBudget
        );

        let me = month_epoch(clock.unix_timestamp);
        if invest.month_epoch != me {
            invest.month_epoch = me;
            invest.spent_this_month_usd = 0;
        }
        let next = invest
            .spent_this_month_usd
            .checked_add(usdc_amount)
            .ok_or(MarkovError::Overflow)?;
        require!(next <= invest.monthly_ceiling_usd, MarkovError::InvestBudget);
        invest.spent_this_month_usd = next;

        require_adjacent_invest(&ctx.accounts.instructions, &ctx.accounts.config)?;

        let r = &mut ctx.accounts.receipt;
        r.account = account.key();
        r.request_id = request_id;
        r.actor = ctx.accounts.actor.key();
        r.kind = ActionKind::InvestExecute;
        r.decision = Decision::Allow;
        r.reason_code = REASON_CONFIRMED;
        r.mandate_version = account.active_mandate_version;
        r.invest_mandate_version = invest.version;
        r.data_slot = data_slot;
        r.venue_id = VENUE_JUPITER;
        r.market_id = pubkey_as_market(&mint);
        r.checks = checks.clone();
        r.route_snapshot_hash = [0; 32];
        r.tx_signature_hint = [0; 32];
        r.pre_state_hash = [0; 32];
        r.post_state_hash = [0; 32];
        r.created_slot = clock.slot;
        r.finalized = false;
        r.bump = ctx.bumps.receipt;

        emit!(ReceiptEmitted {
            account: account.key(),
            request_id,
            actor: ctx.accounts.actor.key(),
            kind: ActionKind::InvestExecute,
            decision: Decision::Allow,
            reason_code: REASON_CONFIRMED,
            mandate_version: account.active_mandate_version,
            invest_mandate_version: invest.version,
            data_slot,
            venue_id: VENUE_JUPITER,
            market_id: pubkey_as_market(&mint),
            checks,
            route_snapshot_hash: [0; 32],
            created_slot: clock.slot,
        });
        Ok(())
    }

    pub fn close_receipt(ctx: Context<CloseReceipt>) -> Result<()> {
        let clock = Clock::get()?;
        let created = ctx.accounts.receipt.created_slot;
        require!(
            clock.slot.saturating_sub(created) >= RECEIPT_CLOSE_SLOTS,
            MarkovError::ReceiptTooYoung
        );
        Ok(())
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy)]
pub struct ObservedIx {
    pub data_age_ms: i64,
    pub freshness_limit_ms: i64,
    pub projected_leverage_bps: i64,
    pub projected_notional_usd: i64,
    pub safety_buffer_bps: i64,
    pub daily_loss_usd: i64,
    pub slippage_bps: i64,
    pub slippage_limit_bps: i64,
}

fn hash_mandate(
    version: u32,
    max_leverage_bps: u32,
    max_notional_usd: u64,
    min_safety_buffer_bps: u32,
    max_daily_loss_usd: u64,
    markets: &[[u8; 16]],
    tier_caps: &[u32; 5],
    allowed_venues: u32,
) -> [u8; 32] {
    let mut buf = Vec::new();
    buf.extend_from_slice(&version.to_le_bytes());
    buf.extend_from_slice(&max_leverage_bps.to_le_bytes());
    buf.extend_from_slice(&max_notional_usd.to_le_bytes());
    buf.extend_from_slice(&min_safety_buffer_bps.to_le_bytes());
    buf.extend_from_slice(&max_daily_loss_usd.to_le_bytes());
    buf.extend_from_slice(&(markets.len() as u32).to_le_bytes());
    for m in markets {
        buf.extend_from_slice(m);
    }
    for t in tier_caps {
        buf.extend_from_slice(&t.to_le_bytes());
    }
    buf.extend_from_slice(&allowed_venues.to_le_bytes());
    hashv(&[&buf]).to_bytes()
}

fn hash_invest(
    version: u32,
    allowlist: &[Pubkey],
    per_period_budget_usd: u64,
    period_seconds: u32,
    monthly_ceiling_usd: u64,
    reserve_floor_usd: u64,
    max_cost_bps: u32,
    single_asset_cap_bps: u32,
    mode: u8,
    window_start_utc: u16,
    window_end_utc: u16,
    days_mask: u8,
) -> [u8; 32] {
    let mut buf = Vec::new();
    buf.extend_from_slice(&version.to_le_bytes());
    buf.extend_from_slice(&(allowlist.len() as u32).to_le_bytes());
    for m in allowlist {
        buf.extend_from_slice(m.as_ref());
    }
    buf.extend_from_slice(&per_period_budget_usd.to_le_bytes());
    buf.extend_from_slice(&period_seconds.to_le_bytes());
    buf.extend_from_slice(&monthly_ceiling_usd.to_le_bytes());
    buf.extend_from_slice(&reserve_floor_usd.to_le_bytes());
    buf.extend_from_slice(&max_cost_bps.to_le_bytes());
    buf.extend_from_slice(&single_asset_cap_bps.to_le_bytes());
    buf.push(mode);
    buf.extend_from_slice(&window_start_utc.to_le_bytes());
    buf.extend_from_slice(&window_end_utc.to_le_bytes());
    buf.push(days_mask);
    hashv(&[&buf]).to_bytes()
}

fn pubkey_as_market(pk: &Pubkey) -> [u8; 16] {
    let mut out = [0u8; 16];
    out.copy_from_slice(&pk.to_bytes()[..16]);
    out
}

/// Pacifica is off-chain: no adjacent program. Other venues must be next ix
/// and must not be this program (anti-wrap).
fn require_adjacent_venue(
    instructions: &AccountInfo,
    venue_id: u8,
    venue_programs: &[Pubkey; MAX_VENUES],
) -> Result<()> {
    if venue_id == VENUE_PACIFICA {
        return Ok(());
    }
    let idx = load_current_index_checked(instructions)?;
    let next_index = (idx as usize)
        .checked_add(1)
        .ok_or(MarkovError::AdjacentVenue)?;
    let next = load_instruction_at_checked(next_index, instructions)
        .map_err(|_| MarkovError::AdjacentVenue)?;
    require!(next.program_id != crate::ID, MarkovError::AdjacentVenue);
    let expected = venue_programs
        .get(venue_id as usize)
        .copied()
        .unwrap_or_default();
    require!(expected != Pubkey::default(), MarkovError::AdjacentVenue);
    require!(next.program_id == expected, MarkovError::AdjacentVenue);
    Ok(())
}

fn require_adjacent_invest(instructions: &AccountInfo, config: &GlobalConfig) -> Result<()> {
    let idx = load_current_index_checked(instructions)?;
    let next_index = (idx as usize)
        .checked_add(1)
        .ok_or(MarkovError::AdjacentVenue)?;
    let next = load_instruction_at_checked(next_index, instructions)
        .map_err(|_| MarkovError::AdjacentVenue)?;
    require!(next.program_id != crate::ID, MarkovError::AdjacentVenue);
    let subs = config.venue_programs[VENUE_SUBSCRIPTIONS as usize];
    let jup = config.venue_programs[VENUE_JUPITER as usize];
    require!(
        next.program_id == subs || next.program_id == jup,
        MarkovError::AdjacentVenue
    );
    Ok(())
}

fn require_keeper_invest(ctx: &Context<RecordSkip>) -> Result<()> {
    let perm = &ctx.accounts.permission;
    require!(!perm.revoked, MarkovError::PermissionDenied);
    require!(
        perm.scopes & SCOPE_INVEST_EXECUTE != 0,
        MarkovError::PermissionDenied
    );
    require!(
        perm.actor == ctx.accounts.actor.key(),
        MarkovError::PermissionDenied
    );
    Ok(())
}

fn require_keeper_invest_exec(ctx: &Context<InvestExecute>) -> Result<()> {
    let perm = &ctx.accounts.permission;
    require!(!perm.revoked, MarkovError::PermissionDenied);
    require!(
        perm.scopes & SCOPE_INVEST_EXECUTE != 0,
        MarkovError::PermissionDenied
    );
    require!(
        perm.actor == ctx.accounts.actor.key(),
        MarkovError::PermissionDenied
    );
    Ok(())
}

#[derive(Accounts)]
pub struct InitializeConfig<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        init,
        payer = admin,
        space = 8 + GlobalConfig::INIT_SPACE,
        seeds = [SEED_CONFIG],
        bump
    )]
    pub config: Account<'info, GlobalConfig>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AdminOnly<'info> {
    pub admin: Signer<'info>,
    #[account(mut, seeds = [SEED_CONFIG], bump = config.bump, has_one = admin)]
    pub config: Account<'info, GlobalConfig>,
}

#[derive(Accounts)]
pub struct CreateAccount<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(
        init,
        payer = owner,
        space = 8 + MarkovAccount::INIT_SPACE,
        seeds = [SEED_ACCOUNT, owner.key().as_ref()],
        bump
    )]
    pub account: Account<'info, MarkovAccount>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(version: u32)]
pub struct SetMandate<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(
        mut,
        seeds = [SEED_ACCOUNT, owner.key().as_ref()],
        bump = account.bump,
        has_one = owner
    )]
    pub account: Account<'info, MarkovAccount>,
    #[account(
        init,
        payer = owner,
        space = 8 + Mandate::INIT_SPACE,
        seeds = [SEED_MANDATE, account.key().as_ref(), &version.to_le_bytes()],
        bump
    )]
    pub mandate: Account<'info, Mandate>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(version: u32)]
pub struct SetInvestMandate<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(
        mut,
        seeds = [SEED_ACCOUNT, owner.key().as_ref()],
        bump = account.bump,
        has_one = owner
    )]
    pub account: Account<'info, MarkovAccount>,
    #[account(
        init,
        payer = owner,
        space = 8 + InvestMandate::INIT_SPACE,
        seeds = [SEED_INVEST, account.key().as_ref(), &version.to_le_bytes()],
        bump
    )]
    pub invest: Account<'info, InvestMandate>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct OwnerAccount<'info> {
    pub owner: Signer<'info>,
    #[account(
        mut,
        seeds = [SEED_ACCOUNT, owner.key().as_ref()],
        bump = account.bump,
        has_one = owner
    )]
    pub account: Account<'info, MarkovAccount>,
}

#[derive(Accounts)]
pub struct SetPermission<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    /// CHECK: actor pubkey is the permission seed; they do not sign grants.
    pub actor: UncheckedAccount<'info>,
    #[account(
        seeds = [SEED_ACCOUNT, owner.key().as_ref()],
        bump = account.bump,
        has_one = owner
    )]
    pub account: Account<'info, MarkovAccount>,
    #[account(
        init_if_needed,
        payer = owner,
        space = 8 + Permission::INIT_SPACE,
        seeds = [SEED_PERM, account.key().as_ref(), actor.key().as_ref()],
        bump
    )]
    pub permission: Account<'info, Permission>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RevokePermission<'info> {
    pub owner: Signer<'info>,
    #[account(
        seeds = [SEED_ACCOUNT, owner.key().as_ref()],
        bump = account.bump,
        has_one = owner
    )]
    pub account: Account<'info, MarkovAccount>,
    #[account(
        mut,
        seeds = [SEED_PERM, account.key().as_ref(), permission.actor.as_ref()],
        bump = permission.bump
    )]
    pub permission: Account<'info, Permission>,
}

#[derive(Accounts)]
#[instruction(request_id: u128, day: u32)]
pub struct RecordDecision<'info> {
    #[account(mut)]
    pub actor: Signer<'info>,
    #[account(seeds = [SEED_ACCOUNT, account.owner.as_ref()], bump = account.bump)]
    pub account: Account<'info, MarkovAccount>,
    #[account(
        seeds = [SEED_MANDATE, account.key().as_ref(), &account.active_mandate_version.to_le_bytes()],
        bump = mandate.bump
    )]
    pub mandate: Account<'info, Mandate>,
    pub permission: Option<Account<'info, Permission>>,
    #[account(
        init,
        payer = actor,
        space = 8 + ActionReceipt::INIT_SPACE,
        seeds = [SEED_RECEIPT, account.key().as_ref(), &request_id.to_le_bytes()],
        bump
    )]
    pub receipt: Account<'info, ActionReceipt>,
    #[account(
        init_if_needed,
        payer = actor,
        space = 8 + DailyLedger::INIT_SPACE,
        seeds = [SEED_DAY, account.key().as_ref(), &day.to_le_bytes()],
        bump
    )]
    pub daily_ledger: Account<'info, DailyLedger>,
    #[account(seeds = [SEED_CONFIG], bump = config.bump)]
    pub config: Account<'info, GlobalConfig>,
    /// CHECK: instructions sysvar
    #[account(address = INSTRUCTIONS_ID)]
    pub instructions: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct FinalizeDecision<'info> {
    pub actor: Signer<'info>,
    #[account(
        mut,
        seeds = [SEED_RECEIPT, receipt.account.as_ref(), &receipt.request_id.to_le_bytes()],
        bump = receipt.bump,
        constraint = receipt.actor == actor.key() @ MarkovError::Unauthorized
    )]
    pub receipt: Account<'info, ActionReceipt>,
}

#[derive(Accounts)]
#[instruction(request_id: u128)]
pub struct RecordSkip<'info> {
    #[account(mut)]
    pub actor: Signer<'info>,
    #[account(seeds = [SEED_ACCOUNT, account.owner.as_ref()], bump = account.bump)]
    pub account: Account<'info, MarkovAccount>,
    #[account(
        seeds = [SEED_PERM, account.key().as_ref(), actor.key().as_ref()],
        bump = permission.bump
    )]
    pub permission: Account<'info, Permission>,
    #[account(
        init,
        payer = actor,
        space = 8 + ActionReceipt::INIT_SPACE,
        seeds = [SEED_RECEIPT, account.key().as_ref(), &request_id.to_le_bytes()],
        bump
    )]
    pub receipt: Account<'info, ActionReceipt>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(request_id: u128)]
pub struct InvestExecute<'info> {
    #[account(mut)]
    pub actor: Signer<'info>,
    #[account(seeds = [SEED_ACCOUNT, account.owner.as_ref()], bump = account.bump)]
    pub account: Account<'info, MarkovAccount>,
    #[account(
        mut,
        seeds = [SEED_INVEST, account.key().as_ref(), &account.active_invest_mandate_version.to_le_bytes()],
        bump = invest.bump
    )]
    pub invest: Account<'info, InvestMandate>,
    #[account(
        seeds = [SEED_PERM, account.key().as_ref(), actor.key().as_ref()],
        bump = permission.bump
    )]
    pub permission: Account<'info, Permission>,
    #[account(
        init,
        payer = actor,
        space = 8 + ActionReceipt::INIT_SPACE,
        seeds = [SEED_RECEIPT, account.key().as_ref(), &request_id.to_le_bytes()],
        bump
    )]
    pub receipt: Account<'info, ActionReceipt>,
    #[account(seeds = [SEED_CONFIG], bump = config.bump)]
    pub config: Account<'info, GlobalConfig>,
    /// CHECK: instructions sysvar
    #[account(address = INSTRUCTIONS_ID)]
    pub instructions: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CloseReceipt<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(
        seeds = [SEED_ACCOUNT, owner.key().as_ref()],
        bump = account.bump,
        has_one = owner
    )]
    pub account: Account<'info, MarkovAccount>,
    #[account(
        mut,
        close = owner,
        seeds = [SEED_RECEIPT, account.key().as_ref(), &receipt.request_id.to_le_bytes()],
        bump = receipt.bump
    )]
    pub receipt: Account<'info, ActionReceipt>,
}
