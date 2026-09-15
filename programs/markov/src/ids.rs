//! Shared constants, bitmaps, and PDA seeds.

use anchor_lang::prelude::*;

pub const SEED_ACCOUNT: &[u8] = b"account";
pub const SEED_MANDATE: &[u8] = b"mandate";
pub const SEED_INVEST: &[u8] = b"invest";
pub const SEED_PERM: &[u8] = b"perm";
pub const SEED_RECEIPT: &[u8] = b"receipt";
pub const SEED_DAY: &[u8] = b"day";
pub const SEED_CONFIG: &[u8] = b"config";

pub const MAX_MARKETS: usize = 32;
pub const MAX_MINTS: usize = 16;
pub const MAX_CHECKS: usize = 12;
pub const MAX_VENUES: usize = 8;
pub const RECEIPT_CLOSE_SLOTS: u64 = 30 * 24 * 60 * 60 / 2; // ~30d at 2s slots; close also checks unix via Clock

pub const VENUE_PACIFICA: u8 = 1;
pub const VENUE_DRIFT: u8 = 2;
pub const VENUE_PHOENIX: u8 = 3;
pub const VENUE_JUPITER: u8 = 4;
pub const VENUE_SUBSCRIPTIONS: u8 = 5;

pub const VENUE_BIT_PACIFICA: u32 = 1 << 0;
pub const VENUE_BIT_DRIFT: u32 = 1 << 1;
pub const VENUE_BIT_PHOENIX: u32 = 1 << 2;
pub const VENUE_BIT_JUPITER: u32 = 1 << 3;

pub const SCOPE_ACCOUNT_READ: u32 = 1 << 0;
pub const SCOPE_RISK_SIMULATE: u32 = 1 << 1;
pub const SCOPE_TRADE_REQUEST: u32 = 1 << 2;
pub const SCOPE_TRADE_REDUCE: u32 = 1 << 3;
pub const SCOPE_INVEST_PROPOSE: u32 = 1 << 4;
pub const SCOPE_INVEST_EXECUTE: u32 = 1 << 5;

pub const REASON_MARKET_NOT_ALLOWED: u16 = 1;
pub const REASON_MAX_LEVERAGE: u16 = 2;
pub const REASON_MAX_NOTIONAL: u16 = 3;
pub const REASON_MIN_SAFETY_BUFFER: u16 = 4;
pub const REASON_DAILY_LOSS: u16 = 5;
pub const REASON_STALE: u16 = 6;
pub const REASON_SLIPPAGE: u16 = 7;
pub const REASON_SCOPE: u16 = 8;
pub const REASON_ACTOR_CAP: u16 = 9;
pub const REASON_VENUE: u16 = 10;
pub const REASON_ACCOUNT_PAUSED: u16 = 11;
pub const REASON_GLOBAL_PAUSED: u16 = 12;
pub const REASON_ASSET: u16 = 20;
pub const REASON_BUDGET: u16 = 21;
pub const REASON_CEILING: u16 = 22;
pub const REASON_RESERVE: u16 = 23;
pub const REASON_COST: u16 = 24;
pub const REASON_MARKET_CLOSED: u16 = 25;
pub const REASON_OFF_HOURS: u16 = 26;
pub const REASON_ASSET_PAUSED: u16 = 27;
pub const REASON_NO_ROUTE: u16 = 28;
pub const REASON_STALE_QUOTE: u16 = 29;
pub const REASON_DELEGATION: u16 = 30;
pub const REASON_SINGLE_ASSET: u16 = 31;
pub const REASON_CONFIRMED: u16 = 100;
pub const REASON_EXEC_FAILED: u16 = 101;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, InitSpace)]
#[borsh(use_discriminant = true)]
#[repr(u8)]
pub enum ExecutionMode {
    OwnerSigned = 0,
    KeeperWithinCap = 1,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, InitSpace)]
#[borsh(use_discriminant = true)]
#[repr(u8)]
pub enum AccountStatus {
    Active = 0,
    Paused = 1,
    Closed = 2,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, InitSpace)]
#[borsh(use_discriminant = true)]
#[repr(u8)]
pub enum ActorKind {
    Owner = 0,
    Keeper = 1,
    McpClient = 2,
    ApiKey = 3,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, InitSpace)]
#[borsh(use_discriminant = true)]
#[repr(u8)]
pub enum InvestExecMode {
    AlwaysOn = 0,
    ReferenceSafe = 1,
    MarketHoursOnly = 2,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, InitSpace)]
#[borsh(use_discriminant = true)]
#[repr(u8)]
pub enum ActionKind {
    TradeOpen = 0,
    TradeReduce = 1,
    TradeClose = 2,
    InvestExecute = 3,
    InvestSkip = 4,
    MandateSet = 5,
    MandatePause = 6,
    PermissionSet = 7,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, InitSpace)]
#[borsh(use_discriminant = true)]
#[repr(u8)]
pub enum Decision {
    Allow = 0,
    Reject = 1,
    RequireApproval = 2,
    Skip = 3,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, InitSpace)]
pub struct Check {
    pub rule: u8,
    pub observed: i64,
    pub limit: i64,
    pub pass: bool,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, InitSpace)]
pub struct Caps {
    pub per_account_gross_notional_usd: u64,
    pub per_position_leverage_bps: u32,
    pub global_daily_invest_usd: u64,
}

pub fn venue_bit(venue_id: u8) -> Option<u32> {
    match venue_id {
        VENUE_PACIFICA => Some(VENUE_BIT_PACIFICA),
        VENUE_DRIFT => Some(VENUE_BIT_DRIFT),
        VENUE_PHOENIX => Some(VENUE_BIT_PHOENIX),
        VENUE_JUPITER => Some(VENUE_BIT_JUPITER),
        _ => None,
    }
}

pub fn day_epoch(unix_ts: i64) -> u32 {
    (unix_ts.div_euclid(86_400)) as u32
}

pub fn month_epoch(unix_ts: i64) -> u32 {
    // UTC year*12+month; good enough for ceiling reset. Not a timezone.
    let days = unix_ts.div_euclid(86_400);
    let (y, m, _d) = civil_from_days(days);
    (y as u32) * 12 + (m as u32)
}

/// Howard Hinnant civil_from_days (UTC).
fn civil_from_days(z: i64) -> (i32, u32, u32) {
    let z = z + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y as i32, m as u32, d as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utc_day_and_month() {
        // 2026-09-15 00:00:00 UTC
        let ts = 1_789_430_400;
        assert_eq!(day_epoch(ts), (ts / 86_400) as u32);
        let (y, m, d) = civil_from_days(ts / 86_400);
        assert_eq!((y, m, d), (2026, 9, 15));
        assert_eq!(month_epoch(ts), 2026 * 12 + 9);
    }
}
