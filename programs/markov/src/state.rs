use crate::ids::*;
use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct GlobalConfig {
    pub admin: Pubkey,
    pub paused: bool,
    pub caps: Caps,
    pub fee_bps: u16,
    pub version: u16,
    pub bump: u8,
    /// Index = venue_id. Unset = default Pubkey (adjacency disabled / unknown).
    pub venue_programs: [Pubkey; MAX_VENUES],
    pub reserved: [u8; 32],
}

#[account]
#[derive(InitSpace)]
pub struct MarkovAccount {
    pub owner: Pubkey,
    pub bump: u8,
    pub execution_mode: ExecutionMode,
    pub active_mandate_version: u32,
    pub active_invest_mandate_version: u32,
    pub status: AccountStatus,
    pub replay_domain: u64,
    pub linked_venues: u32,
    pub created_slot: u64,
    pub reserved: [u8; 64],
}

#[account]
#[derive(InitSpace)]
pub struct Mandate {
    pub account: Pubkey,
    pub version: u32,
    pub hash: [u8; 32],
    pub max_leverage_bps: u32,
    pub max_notional_usd: u64,
    pub min_safety_buffer_bps: u32,
    pub max_daily_loss_usd: u64,
    #[max_len(MAX_MARKETS)]
    pub approved_markets: Vec<[u8; 16]>,
    pub tier_caps: [u32; 5],
    pub allowed_venues: u32,
    pub activated_slot: u64,
    pub activated_by: Pubkey,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct InvestMandate {
    pub account: Pubkey,
    pub version: u32,
    pub hash: [u8; 32],
    #[max_len(MAX_MINTS)]
    pub allowlist: Vec<Pubkey>,
    pub per_period_budget_usd: u64,
    pub period_seconds: u32,
    pub monthly_ceiling_usd: u64,
    pub reserve_floor_usd: u64,
    pub max_cost_bps: u32,
    pub single_asset_cap_bps: u32,
    pub execution_mode: InvestExecMode,
    pub window_start_utc: u16,
    pub window_end_utc: u16,
    pub days_mask: u8,
    pub paused: bool,
    pub spent_this_month_usd: u64,
    pub month_epoch: u32,
    pub activated_slot: u64,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct Permission {
    pub account: Pubkey,
    pub actor: Pubkey,
    pub kind: ActorKind,
    pub scopes: u32,
    pub per_action_cap_usd: u64,
    pub daily_cap_usd: u64,
    pub spent_today_usd: u64,
    pub day_epoch: u32,
    pub expires_slot: u64,
    pub revoked: bool,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct ActionReceipt {
    pub account: Pubkey,
    pub request_id: u128,
    pub actor: Pubkey,
    pub kind: ActionKind,
    pub decision: Decision,
    pub reason_code: u16,
    pub mandate_version: u32,
    pub invest_mandate_version: u32,
    pub data_slot: u64,
    pub venue_id: u8,
    pub market_id: [u8; 16],
    #[max_len(MAX_CHECKS)]
    pub checks: Vec<Check>,
    pub route_snapshot_hash: [u8; 32],
    pub tx_signature_hint: [u8; 32],
    pub pre_state_hash: [u8; 32],
    pub post_state_hash: [u8; 32],
    pub created_slot: u64,
    pub finalized: bool,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct DailyLedger {
    pub account: Pubkey,
    pub day_epoch: u32,
    pub realized_loss_usd: u64,
    pub gross_notional_usd: u64,
    pub actions_count: u32,
    pub bump: u8,
}
