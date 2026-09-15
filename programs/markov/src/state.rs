//! Accounts. Every PDA's seeds are documented on the struct; every money field
//! is micro-USD (`u64`); every observed value on a check is `i64` so a
//! negative safety buffer or loss can be stated rather than clamped.

use anchor_lang::prelude::*;

pub const MAX_MARKETS: usize = 32;
pub const MAX_MINTS: usize = 16;
pub const MAX_CHECKS: usize = 12;
pub const MAX_VENUES: usize = 8;
/// A canonical market id is 16 NUL-padded ASCII bytes, e.g. `SOL-PERP`.
pub type MarketId = [u8; 16];

pub const SECONDS_PER_DAY: i64 = 86_400;
/// Receipts may be closed for rent after this many seconds.
pub const RECEIPT_CLOSE_AFTER_SECS: i64 = 30 * SECONDS_PER_DAY;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq, InitSpace)]
pub enum ExecutionMode {
    OwnerSigned,
    KeeperWithinCap,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq, InitSpace)]
pub enum AccountStatus {
    Active,
    Paused,
    Closed,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq, InitSpace)]
pub enum InvestExecutionMode {
    AlwaysOn,
    ReferenceSafe,
    MarketHoursOnly,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq, InitSpace)]
pub enum ActorKind {
    Owner,
    Keeper,
    McpClient,
    ApiKey,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq, InitSpace)]
pub enum ReceiptKind {
    TradeOpen,
    TradeReduce,
    TradeClose,
    InvestExecute,
    InvestSkip,
    MandateSet,
    MandatePause,
    PermissionSet,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq, InitSpace)]
pub enum Decision {
    Allow,
    Reject,
    RequireApproval,
    Skip,
}

/// Official-price market status as supplied by the keeper for Invest checks.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq, InitSpace)]
pub enum MarketStatus {
    Unknown,
    Open,
    Closed,
    Halted,
}

/// Permission scope bits.
pub mod scopes {
    pub const ACCOUNT_READ: u32 = 1 << 0;
    pub const RISK_SIMULATE: u32 = 1 << 1;
    pub const TRADE_REQUEST: u32 = 1 << 2;
    pub const TRADE_REDUCE: u32 = 1 << 3;
    pub const INVEST_PROPOSE: u32 = 1 << 4;
    pub const INVEST_EXECUTE: u32 = 1 << 5;
    pub const ALL: u32 = ACCOUNT_READ
        | RISK_SIMULATE
        | TRADE_REQUEST
        | TRADE_REDUCE
        | INVEST_PROPOSE
        | INVEST_EXECUTE;
}

/// Stable reason codes (`01_Program_Build_Prompt` §3). Never renumber.
pub mod reason {
    pub const NONE: u16 = 0;
    pub const MARKET_NOT_ALLOWED: u16 = 1;
    pub const MAX_LEVERAGE_EXCEEDED: u16 = 2;
    pub const MAX_NOTIONAL_EXCEEDED: u16 = 3;
    pub const MIN_SAFETY_BUFFER: u16 = 4;
    pub const DAILY_LOSS_BUDGET_EXCEEDED: u16 = 5;
    pub const STALE_MARKET_DATA: u16 = 6;
    pub const SLIPPAGE_LIMIT: u16 = 7;
    pub const ACTOR_SCOPE_DENIED: u16 = 8;
    pub const ACTOR_CAP_EXCEEDED: u16 = 9;
    pub const VENUE_NOT_ALLOWED: u16 = 10;
    pub const ACCOUNT_PAUSED: u16 = 11;
    pub const GLOBAL_PAUSED: u16 = 12;
    pub const ASSET_NOT_ALLOWLISTED: u16 = 20;
    pub const BUDGET_EXHAUSTED: u16 = 21;
    pub const MONTHLY_CEILING_EXCEEDED: u16 = 22;
    pub const RESERVE_FLOOR: u16 = 23;
    pub const COST_LIMIT: u16 = 24;
    pub const MARKET_CLOSED: u16 = 25;
    pub const OFF_HOURS_DEVIATION: u16 = 26;
    pub const ASSET_PAUSED: u16 = 27;
    pub const NO_ROUTE: u16 = 28;
    pub const STALE_QUOTE: u16 = 29;
    pub const DELEGATION_INSUFFICIENT: u16 = 30;
    pub const SINGLE_ASSET_CAP: u16 = 31;
    pub const EXECUTION_CONFIRMED: u16 = 100;
    pub const EXECUTION_FAILED: u16 = 101;
}

/// Rule ids carried on `Check.rule`. The program enforces the ones marked
/// "on-chain" against the active mandate and the global config; the rest are
/// recorded as the caller evaluated them (the API and the program share this
/// table through `packages/sdk`).
pub mod rule {
    pub const GLOBAL_PAUSE: u8 = 1; // on-chain
    pub const ACCOUNT_PAUSE: u8 = 2; // on-chain
    pub const ACTOR_SCOPE: u8 = 3; // on-chain
    pub const ACTOR_CAP: u8 = 4; // on-chain
    pub const VENUE_ALLOWED: u8 = 5; // on-chain
    pub const MARKET_ALLOWED: u8 = 6; // on-chain
    pub const DATA_FRESHNESS: u8 = 7; // on-chain (config freshness_slots)
    pub const LEVERAGE: u8 = 8; // on-chain
    pub const NOTIONAL: u8 = 9; // on-chain
    pub const SAFETY_BUFFER: u8 = 10; // on-chain
    pub const DAILY_LOSS: u8 = 11; // on-chain
    pub const SLIPPAGE: u8 = 12; // off-chain (recorded)
    pub const INVEST_ALLOWLIST: u8 = 20; // on-chain
    pub const INVEST_BUDGET: u8 = 21; // on-chain
    pub const INVEST_CEILING: u8 = 22; // on-chain
    pub const INVEST_RESERVE: u8 = 23; // on-chain (token account read)
    pub const INVEST_COST: u8 = 24; // on-chain
    pub const INVEST_MARKET_STATUS: u8 = 25; // on-chain vs supplied status
    pub const INVEST_DEVIATION: u8 = 26; // on-chain vs supplied reference
    pub const INVEST_WINDOW: u8 = 27; // on-chain (clock)
    pub const INVEST_SINGLE_ASSET: u8 = 31; // off-chain (recorded)
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq, InitSpace)]
pub struct Check {
    pub rule: u8,
    pub observed: i64,
    pub limit: i64,
    pub pass: bool,
}

#[derive(
    AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq, InitSpace, Default,
)]
pub struct GlobalCaps {
    /// Per-account gross perp notional, micro-USD (conventions §7: $2,000).
    pub per_account_gross_notional_usd: u64,
    /// Per-position leverage ceiling, bps (2.0x = 20_000).
    pub per_position_leverage_bps: u32,
    /// Global daily Invest keeper spend, micro-USD ($2,000).
    pub global_daily_invest_spend_usd: u64,
    /// Per-execution Invest size bounds, micro-USD ($5–$100).
    pub invest_min_per_execution_usd: u64,
    pub invest_max_per_execution_usd: u64,
    /// Market data older than this many slots is stale (rule 7).
    pub freshness_slots: u64,
}

/// `["config"]`
#[account]
#[derive(InitSpace)]
pub struct GlobalConfig {
    pub admin: Pubkey,
    pub paused: bool,
    pub caps: GlobalCaps,
    pub fee_bps: u16,
    pub version: u16,
    /// Venue program ids by `venue_id` (index). Zero = unset.
    pub venue_programs: [Pubkey; MAX_VENUES],
    /// The spend-delegation program and the swap program an Invest execution
    /// must be followed by, in that order.
    pub invest_collect_program: Pubkey,
    pub invest_swap_program: Pubkey,
    /// The settlement mint an Invest reserve proof must be denominated in.
    pub usdc_mint: Pubkey,
    pub invest_day_epoch: i64,
    pub invest_spent_today_usd: u64,
    pub bump: u8,
}

impl GlobalConfig {
    pub const SEED: &'static [u8] = b"config";

    pub fn venue_program(&self, venue_id: u8) -> Option<Pubkey> {
        let p = *self.venue_programs.get(venue_id as usize)?;
        if p == Pubkey::default() {
            None
        } else {
            Some(p)
        }
    }

    pub fn roll_invest_day(&mut self, now: i64) {
        let day = now.div_euclid(SECONDS_PER_DAY);
        if day != self.invest_day_epoch {
            self.invest_day_epoch = day;
            self.invest_spent_today_usd = 0;
        }
    }
}

/// `["account", owner]`
#[account]
#[derive(InitSpace)]
pub struct MarkovAccount {
    pub owner: Pubkey,
    pub bump: u8,
    pub execution_mode: ExecutionMode,
    pub active_mandate_version: u32,
    pub active_invest_mandate_version: u32,
    pub status: AccountStatus,
    /// Domain for request ids; bumps never, reserved for a future re-key.
    pub replay_domain: u64,
    /// Bit `venue_id` set once the owner linked that venue.
    pub linked_venues: u64,
    pub created_slot: u64,
    pub reserved: [u8; 64],
}

impl MarkovAccount {
    pub const SEED: &'static [u8] = b"account";
}

/// `["mandate", account, version_le]`
#[account]
#[derive(InitSpace)]
pub struct Mandate {
    pub account: Pubkey,
    pub version: u32,
    /// sha256 of the canonical encoding of the fields below.
    pub hash: [u8; 32],
    pub max_leverage_bps: u32,
    pub max_notional_usd: u64,
    pub min_safety_buffer_bps: u32,
    pub max_daily_loss_usd: u64,
    #[max_len(32)]
    pub approved_markets: Vec<[u8; 16]>,
    /// Leverage ceilings by risk tier 0..5, bps.
    pub tier_caps_bps: [u32; 5],
    /// Bit `venue_id` set when the venue may execute for this mandate.
    pub allowed_venues: u64,
    pub activated_slot: u64,
    pub activated_by: Pubkey,
    pub bump: u8,
}

impl Mandate {
    pub const SEED: &'static [u8] = b"mandate";

    pub fn market_allowed(&self, market: &MarketId) -> bool {
        self.approved_markets.iter().any(|m| m == market)
    }
    pub fn venue_allowed(&self, venue_id: u8) -> bool {
        venue_id < 64 && self.allowed_venues & (1u64 << venue_id) != 0
    }
}

/// The fields an owner signs for `set_mandate`.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Eq)]
pub struct MandateFields {
    pub max_leverage_bps: u32,
    pub max_notional_usd: u64,
    pub min_safety_buffer_bps: u32,
    pub max_daily_loss_usd: u64,
    pub approved_markets: Vec<MarketId>,
    pub tier_caps_bps: [u32; 5],
    pub allowed_venues: u64,
}

impl MandateFields {
    pub fn validate(&self) -> bool {
        self.approved_markets.len() <= MAX_MARKETS
            && self.max_leverage_bps > 0
            && self.max_notional_usd > 0
            && self.min_safety_buffer_bps <= 10_000
    }
}

/// `["invest", account, version_le]`
#[account]
#[derive(InitSpace)]
pub struct InvestMandate {
    pub account: Pubkey,
    pub version: u32,
    pub hash: [u8; 32],
    #[max_len(16)]
    pub allowlist: Vec<Pubkey>,
    pub per_period_budget_usd: u64,
    pub period_seconds: u32,
    pub monthly_ceiling_usd: u64,
    pub reserve_floor_usd: u64,
    pub max_cost_bps: u16,
    /// Off-hours deviation from the official reference tolerated in
    /// ReferenceSafe mode, bps.
    pub max_reference_deviation_bps: u16,
    pub single_asset_cap_bps: u16,
    pub execution_mode: InvestExecutionMode,
    /// Seconds after 00:00 UTC.
    pub window_start_utc: u32,
    pub window_end_utc: u32,
    /// Bit 0 = Monday … bit 6 = Sunday.
    pub days_mask: u8,
    pub paused: bool,
    pub spent_this_month_usd: u64,
    /// `year * 12 + month` of the current spend window.
    pub month_epoch: u32,
    pub activated_slot: u64,
    pub bump: u8,
}

impl InvestMandate {
    pub const SEED: &'static [u8] = b"invest";
    pub fn mint_allowed(&self, mint: &Pubkey) -> bool {
        self.allowlist.iter().any(|m| m == mint)
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Eq)]
pub struct InvestMandateFields {
    pub allowlist: Vec<Pubkey>,
    pub per_period_budget_usd: u64,
    pub period_seconds: u32,
    pub monthly_ceiling_usd: u64,
    pub reserve_floor_usd: u64,
    pub max_cost_bps: u16,
    pub max_reference_deviation_bps: u16,
    pub single_asset_cap_bps: u16,
    pub execution_mode: InvestExecutionMode,
    pub window_start_utc: u32,
    pub window_end_utc: u32,
    pub days_mask: u8,
}

impl InvestMandateFields {
    pub fn validate(&self) -> bool {
        !self.allowlist.is_empty()
            && self.allowlist.len() <= MAX_MINTS
            && self.per_period_budget_usd > 0
            && self.period_seconds > 0
            && self.monthly_ceiling_usd >= self.per_period_budget_usd
            && self.max_cost_bps <= 10_000
            && self.max_reference_deviation_bps <= 10_000
            && self.single_asset_cap_bps <= 10_000
            && self.window_start_utc < 86_400
            && self.window_end_utc <= 86_400
            && self.days_mask != 0
    }
}

/// `["perm", account, actor]`
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
    pub day_epoch: i64,
    pub expires_slot: u64,
    pub revoked: bool,
    pub bump: u8,
}

impl Permission {
    pub const SEED: &'static [u8] = b"perm";
    pub fn roll_day(&mut self, now: i64) {
        let day = now.div_euclid(SECONDS_PER_DAY);
        if day != self.day_epoch {
            self.day_epoch = day;
            self.spent_today_usd = 0;
        }
    }
}

/// `["receipt", account, request_id_le]`
#[account]
#[derive(InitSpace)]
pub struct ActionReceipt {
    pub account: Pubkey,
    pub request_id: u128,
    pub actor: Pubkey,
    pub kind: ReceiptKind,
    pub decision: Decision,
    pub reason_code: u16,
    pub mandate_version: u32,
    pub invest_mandate_version: u32,
    pub data_slot: u64,
    pub venue_id: u8,
    pub market_id: [u8; 16],
    #[max_len(12)]
    pub checks: Vec<Check>,
    pub route_snapshot_hash: [u8; 32],
    pub tx_signature_hint: [u8; 32],
    pub pre_state_hash: [u8; 32],
    pub post_state_hash: [u8; 32],
    pub finalized: bool,
    pub created_slot: u64,
    pub created_ts: i64,
    pub bump: u8,
}

impl ActionReceipt {
    pub const SEED: &'static [u8] = b"receipt";
}

/// `["day", account, day_epoch_le]`
#[account]
#[derive(InitSpace)]
pub struct DailyLedger {
    pub account: Pubkey,
    pub day_epoch: i64,
    pub realized_loss_usd: u64,
    pub gross_notional_usd: u64,
    pub actions_count: u32,
    pub bump: u8,
}

impl DailyLedger {
    pub const SEED: &'static [u8] = b"day";
}
