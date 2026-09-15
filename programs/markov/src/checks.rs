//! The pure part of every decision. No accounts, no clock reads, no I/O:
//! everything is handed in, so the same functions are unit-tested and
//! property-tested on the host and executed unchanged on chain.
//!
//! The caller (the API's policy engine) supplies `Check`s with the values it
//! *observed*; the program does not trust the supplied `limit` or `pass` for
//! any rule it enforces. It replaces the limit with the mandate's own value,
//! recomputes `pass`, and an `Allow` that fails any enforced rule is refused
//! outright (invariant 1).

use crate::state::{
    reason, rule, Check, GlobalCaps, InvestExecutionMode, InvestMandate, Mandate, MarketId,
    MarketStatus, MAX_CHECKS,
};

/// What the program knows about the trade being decided, in the units the
/// checks are stated in.
#[derive(Clone, Copy, Debug)]
pub struct TradeFacts {
    pub venue_id: u8,
    pub market: MarketId,
    /// Slot the market data was observed at; `now_slot - data_slot` is the age.
    pub data_slot: u64,
    pub now_slot: u64,
    /// Realized loss already on today's ledger, micro-USD.
    pub ledger_loss_usd: u64,
}

/// Outcome of recomputing the enforced checks.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Verdict {
    /// The checks as the program will record them (limits from the mandate).
    pub checks: Vec<Check>,
    /// First failing enforced rule's reason code, or `reason::NONE`.
    pub first_failure: u16,
}

fn find(checks: &[Check], id: u8) -> Option<&Check> {
    checks.iter().find(|c| c.rule == id)
}

fn i64_of(v: u64) -> i64 {
    i64::try_from(v).unwrap_or(i64::MAX)
}

/// Recompute every rule the program enforces for a perp decision.
///
/// Rules whose observed value the caller must supply (leverage, notional,
/// safety buffer, daily loss, freshness): if the caller omitted the check, the
/// rule is treated as failed with the reason for that rule — an absent
/// observation is not a pass.
pub fn evaluate_trade(
    mandate: &Mandate,
    caps: &GlobalCaps,
    supplied: &[Check],
    facts: &TradeFacts,
) -> Verdict {
    let mut out: Vec<Check> = Vec::with_capacity(MAX_CHECKS);
    let mut first = reason::NONE;
    let mut push = |c: Check, code: u16| {
        if !c.pass && first == reason::NONE {
            first = code;
        }
        out.push(c);
    };

    // 5 venue allowed by the mandate bitmap.
    push(
        Check {
            rule: rule::VENUE_ALLOWED,
            observed: i64::from(facts.venue_id),
            limit: i64::try_from(mandate.allowed_venues).unwrap_or(i64::MAX),
            pass: mandate.venue_allowed(facts.venue_id),
        },
        reason::VENUE_NOT_ALLOWED,
    );
    // 6 market in the approved list.
    push(
        Check {
            rule: rule::MARKET_ALLOWED,
            observed: i64::from(mandate.market_allowed(&facts.market)),
            limit: 1,
            pass: mandate.market_allowed(&facts.market),
        },
        reason::MARKET_NOT_ALLOWED,
    );
    // 7 freshness: age in slots against the global window.
    let age = facts.now_slot.saturating_sub(facts.data_slot);
    push(
        Check {
            rule: rule::DATA_FRESHNESS,
            observed: i64_of(age),
            limit: i64_of(caps.freshness_slots),
            pass: age <= caps.freshness_slots && facts.data_slot <= facts.now_slot,
        },
        reason::STALE_MARKET_DATA,
    );
    // 8 projected leverage against the mandate and the global cap.
    let lev_limit = mandate
        .max_leverage_bps
        .min(caps.per_position_leverage_bps.max(1));
    match find(supplied, rule::LEVERAGE) {
        Some(c) => push(
            Check {
                rule: rule::LEVERAGE,
                observed: c.observed,
                limit: i64::from(lev_limit),
                pass: c.observed >= 0 && c.observed <= i64::from(lev_limit),
            },
            reason::MAX_LEVERAGE_EXCEEDED,
        ),
        None => push(
            Check {
                rule: rule::LEVERAGE,
                observed: i64::MAX,
                limit: i64::from(lev_limit),
                pass: false,
            },
            reason::MAX_LEVERAGE_EXCEEDED,
        ),
    }
    // 9 projected notional against the mandate and the global cap.
    let notional_limit = mandate
        .max_notional_usd
        .min(caps.per_account_gross_notional_usd.max(1));
    match find(supplied, rule::NOTIONAL) {
        Some(c) => push(
            Check {
                rule: rule::NOTIONAL,
                observed: c.observed,
                limit: i64_of(notional_limit),
                pass: c.observed >= 0 && c.observed <= i64_of(notional_limit),
            },
            reason::MAX_NOTIONAL_EXCEEDED,
        ),
        None => push(
            Check {
                rule: rule::NOTIONAL,
                observed: i64::MAX,
                limit: i64_of(notional_limit),
                pass: false,
            },
            reason::MAX_NOTIONAL_EXCEEDED,
        ),
    }
    // 10 projected safety buffer must stay above the floor.
    match find(supplied, rule::SAFETY_BUFFER) {
        Some(c) => push(
            Check {
                rule: rule::SAFETY_BUFFER,
                observed: c.observed,
                limit: i64::from(mandate.min_safety_buffer_bps),
                pass: c.observed >= i64::from(mandate.min_safety_buffer_bps),
            },
            reason::MIN_SAFETY_BUFFER,
        ),
        None => push(
            Check {
                rule: rule::SAFETY_BUFFER,
                observed: i64::MIN,
                limit: i64::from(mandate.min_safety_buffer_bps),
                pass: false,
            },
            reason::MIN_SAFETY_BUFFER,
        ),
    }
    // 11 daily loss: today's ledger plus the caller's unrealized figure.
    let loss_limit = i64_of(mandate.max_daily_loss_usd);
    match find(supplied, rule::DAILY_LOSS) {
        Some(c) => {
            let total = c.observed.saturating_add(i64_of(facts.ledger_loss_usd));
            push(
                Check {
                    rule: rule::DAILY_LOSS,
                    observed: total,
                    limit: loss_limit,
                    pass: total <= loss_limit,
                },
                reason::DAILY_LOSS_BUDGET_EXCEEDED,
            )
        }
        None => push(
            Check {
                rule: rule::DAILY_LOSS,
                observed: i64::MAX,
                limit: loss_limit,
                pass: false,
            },
            reason::DAILY_LOSS_BUDGET_EXCEEDED,
        ),
    }
    // Recorded-only rules travel as supplied, capped by MAX_CHECKS.
    for c in supplied.iter().filter(|c| !is_enforced_trade_rule(c.rule)) {
        if out.len() >= MAX_CHECKS {
            break;
        }
        out.push(*c);
    }
    Verdict {
        checks: out,
        first_failure: first,
    }
}

pub fn is_enforced_trade_rule(id: u8) -> bool {
    matches!(
        id,
        rule::GLOBAL_PAUSE
            | rule::ACCOUNT_PAUSE
            | rule::ACTOR_SCOPE
            | rule::ACTOR_CAP
            | rule::VENUE_ALLOWED
            | rule::MARKET_ALLOWED
            | rule::DATA_FRESHNESS
            | rule::LEVERAGE
            | rule::NOTIONAL
            | rule::SAFETY_BUFFER
            | rule::DAILY_LOSS
    )
}

/// What the keeper hands `invest_execute`, besides the accounts it passes.
#[derive(Clone, Copy, Debug)]
pub struct InvestFacts {
    pub mint_allowed: bool,
    pub usdc_amount: u64,
    /// The owner's USDC balance *before* this pull, read from the token account.
    pub reserve_balance_usd: u64,
    pub quote_cost_bps: u16,
    pub market_status: MarketStatus,
    /// Official reference price and the quote price, both micro-USD per unit;
    /// zero when no reference is available.
    pub official_price: u64,
    pub quote_price: u64,
    /// Seconds after 00:00 UTC and weekday bit (Monday = bit 0), from the clock.
    pub seconds_into_day: u32,
    pub weekday_bit: u8,
    /// Global daily keeper spend already used today.
    pub global_spent_today_usd: u64,
    /// Spend already counted this month (after the month roll).
    pub spent_this_month_usd: u64,
}

pub fn evaluate_invest(
    m: &InvestMandate,
    caps: &GlobalCaps,
    f: &InvestFacts,
    supplied: &[Check],
) -> Verdict {
    let mut out: Vec<Check> = Vec::with_capacity(MAX_CHECKS);
    let mut first = reason::NONE;
    let mut push = |c: Check, code: u16| {
        if !c.pass && first == reason::NONE {
            first = code;
        }
        out.push(c);
    };
    push(
        Check {
            rule: rule::INVEST_ALLOWLIST,
            observed: i64::from(f.mint_allowed),
            limit: 1,
            pass: f.mint_allowed,
        },
        reason::ASSET_NOT_ALLOWLISTED,
    );
    let within_bounds = f.usdc_amount >= caps.invest_min_per_execution_usd
        && (caps.invest_max_per_execution_usd == 0
            || f.usdc_amount <= caps.invest_max_per_execution_usd);
    push(
        Check {
            rule: rule::INVEST_BUDGET,
            observed: i64_of(f.usdc_amount),
            limit: i64_of(m.per_period_budget_usd),
            pass: f.usdc_amount <= m.per_period_budget_usd && within_bounds,
        },
        reason::BUDGET_EXHAUSTED,
    );
    let month_total = f.spent_this_month_usd.saturating_add(f.usdc_amount);
    let global_total = f.global_spent_today_usd.saturating_add(f.usdc_amount);
    push(
        Check {
            rule: rule::INVEST_CEILING,
            observed: i64_of(month_total),
            limit: i64_of(m.monthly_ceiling_usd),
            pass: month_total <= m.monthly_ceiling_usd
                && (caps.global_daily_invest_spend_usd == 0
                    || global_total <= caps.global_daily_invest_spend_usd),
        },
        reason::MONTHLY_CEILING_EXCEEDED,
    );
    let after = f.reserve_balance_usd.saturating_sub(f.usdc_amount);
    push(
        Check {
            rule: rule::INVEST_RESERVE,
            observed: i64_of(after),
            limit: i64_of(m.reserve_floor_usd),
            pass: f.reserve_balance_usd >= f.usdc_amount && after >= m.reserve_floor_usd,
        },
        reason::RESERVE_FLOOR,
    );
    push(
        Check {
            rule: rule::INVEST_COST,
            observed: i64::from(f.quote_cost_bps),
            limit: i64::from(m.max_cost_bps),
            pass: f.quote_cost_bps <= m.max_cost_bps,
        },
        reason::COST_LIMIT,
    );
    let (status_ok, status_code) = match m.execution_mode {
        InvestExecutionMode::AlwaysOn => (true, reason::MARKET_CLOSED),
        InvestExecutionMode::MarketHoursOnly => {
            (f.market_status == MarketStatus::Open, reason::MARKET_CLOSED)
        }
        InvestExecutionMode::ReferenceSafe => (
            matches!(f.market_status, MarketStatus::Open | MarketStatus::Closed),
            reason::MARKET_CLOSED,
        ),
    };
    push(
        Check {
            rule: rule::INVEST_MARKET_STATUS,
            observed: f.market_status as i64,
            limit: m.execution_mode as i64,
            pass: status_ok,
        },
        status_code,
    );
    if m.execution_mode == InvestExecutionMode::ReferenceSafe {
        let deviation = deviation_bps(f.quote_price, f.official_price);
        push(
            Check {
                rule: rule::INVEST_DEVIATION,
                observed: deviation.map(i64::from).unwrap_or(i64::MAX),
                limit: i64::from(m.max_reference_deviation_bps),
                pass: matches!(deviation, Some(d) if d <= u32::from(m.max_reference_deviation_bps)),
            },
            reason::OFF_HOURS_DEVIATION,
        );
    }
    let in_window = in_window(m.window_start_utc, m.window_end_utc, f.seconds_into_day)
        && (m.days_mask & f.weekday_bit) != 0;
    push(
        Check {
            rule: rule::INVEST_WINDOW,
            observed: i64::from(f.seconds_into_day),
            limit: i64::from(m.window_end_utc),
            pass: in_window,
        },
        reason::MARKET_CLOSED,
    );
    for c in supplied.iter().filter(|c| !is_enforced_invest_rule(c.rule)) {
        if out.len() >= MAX_CHECKS {
            break;
        }
        out.push(*c);
    }
    Verdict {
        checks: out,
        first_failure: first,
    }
}

pub fn is_enforced_invest_rule(id: u8) -> bool {
    matches!(
        id,
        rule::INVEST_ALLOWLIST
            | rule::INVEST_BUDGET
            | rule::INVEST_CEILING
            | rule::INVEST_RESERVE
            | rule::INVEST_COST
            | rule::INVEST_MARKET_STATUS
            | rule::INVEST_DEVIATION
            | rule::INVEST_WINDOW
    )
}

/// `|quote - official| / official` in bps; `None` when no reference exists.
pub fn deviation_bps(quote: u64, official: u64) -> Option<u32> {
    if official == 0 || quote == 0 {
        return None;
    }
    let diff = quote.abs_diff(official) as u128;
    let bps = diff.saturating_mul(10_000) / official as u128;
    Some(u32::try_from(bps).unwrap_or(u32::MAX))
}

/// A window may wrap midnight (`start > end`). `end == 86_400` means "to midnight".
pub fn in_window(start: u32, end: u32, t: u32) -> bool {
    if start == end {
        return true;
    }
    if start < end {
        t >= start && t < end
    } else {
        t >= start || t < end
    }
}

/// `year * 12 + month` for a unix timestamp (proleptic Gregorian, UTC).
pub fn month_epoch(ts: i64) -> u32 {
    let days = ts.div_euclid(86_400);
    // Civil-from-days (Howard Hinnant), valid for the range we care about.
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y * 12 + m) as u32
}

/// Monday = bit 0 … Sunday = bit 6, for a unix timestamp.
pub fn weekday_bit(ts: i64) -> u8 {
    let days = ts.div_euclid(86_400);
    // 1970-01-01 was a Thursday (index 3 with Monday = 0).
    let idx = (days + 3).rem_euclid(7);
    1u8 << idx
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use anchor_lang::prelude::Pubkey;
    use proptest::prelude::*;

    fn mandate() -> Mandate {
        Mandate {
            account: Pubkey::default(),
            version: 1,
            hash: [0; 32],
            max_leverage_bps: 20_000,
            max_notional_usd: 2_000_000_000,
            min_safety_buffer_bps: 2_000,
            max_daily_loss_usd: 100_000_000,
            approved_markets: vec![*b"SOL-PERP\0\0\0\0\0\0\0\0", *b"BTC-PERP\0\0\0\0\0\0\0\0"],
            tier_caps_bps: [20_000, 20_000, 10_000, 5_000, 0],
            allowed_venues: 0b11,
            activated_slot: 0,
            activated_by: Pubkey::default(),
            bump: 0,
        }
    }
    fn caps() -> GlobalCaps {
        GlobalCaps {
            per_account_gross_notional_usd: 2_000_000_000,
            per_position_leverage_bps: 20_000,
            global_daily_invest_spend_usd: 2_000_000_000,
            invest_min_per_execution_usd: 5_000_000,
            invest_max_per_execution_usd: 100_000_000,
            freshness_slots: 2,
        }
    }
    fn facts() -> TradeFacts {
        TradeFacts {
            venue_id: 1,
            market: *b"SOL-PERP\0\0\0\0\0\0\0\0",
            data_slot: 100,
            now_slot: 101,
            ledger_loss_usd: 0,
        }
    }
    fn ok_checks() -> Vec<Check> {
        vec![
            Check {
                rule: rule::LEVERAGE,
                observed: 15_000,
                limit: 0,
                pass: false,
            },
            Check {
                rule: rule::NOTIONAL,
                observed: 1_000_000_000,
                limit: 0,
                pass: false,
            },
            Check {
                rule: rule::SAFETY_BUFFER,
                observed: 3_000,
                limit: 0,
                pass: false,
            },
            Check {
                rule: rule::DAILY_LOSS,
                observed: 10_000_000,
                limit: 0,
                pass: false,
            },
            Check {
                rule: rule::SLIPPAGE,
                observed: 12,
                limit: 20,
                pass: true,
            },
        ]
    }

    #[test]
    fn supplied_limits_and_pass_flags_are_ignored() {
        // The caller lied about every limit and flag; the program recomputes.
        let v = evaluate_trade(&mandate(), &caps(), &ok_checks(), &facts());
        assert_eq!(v.first_failure, reason::NONE);
        let lev = v.checks.iter().find(|c| c.rule == rule::LEVERAGE).unwrap();
        assert_eq!(lev.limit, 20_000);
        assert!(lev.pass);
        assert!(v
            .checks
            .iter()
            .any(|c| c.rule == rule::SLIPPAGE && c.limit == 20));
    }

    #[test]
    fn first_failure_names_the_rule_in_ladder_order() {
        let mut f = facts();
        f.venue_id = 5;
        let v = evaluate_trade(&mandate(), &caps(), &ok_checks(), &f);
        assert_eq!(v.first_failure, reason::VENUE_NOT_ALLOWED);
        let mut f = facts();
        f.market = *b"DOGE-PERP\0\0\0\0\0\0\0";
        assert_eq!(
            evaluate_trade(&mandate(), &caps(), &ok_checks(), &f).first_failure,
            reason::MARKET_NOT_ALLOWED
        );
        let mut f = facts();
        f.now_slot = 200;
        assert_eq!(
            evaluate_trade(&mandate(), &caps(), &ok_checks(), &f).first_failure,
            reason::STALE_MARKET_DATA
        );
    }

    #[test]
    fn a_missing_observation_is_a_failure_not_a_pass() {
        let checks: Vec<Check> = ok_checks()
            .into_iter()
            .filter(|c| c.rule != rule::SAFETY_BUFFER)
            .collect();
        let v = evaluate_trade(&mandate(), &caps(), &checks, &facts());
        assert_eq!(v.first_failure, reason::MIN_SAFETY_BUFFER);
    }

    #[test]
    fn global_caps_tighten_the_mandate() {
        let mut c = caps();
        c.per_position_leverage_bps = 10_000;
        let v = evaluate_trade(&mandate(), &c, &ok_checks(), &facts());
        assert_eq!(v.first_failure, reason::MAX_LEVERAGE_EXCEEDED);
    }

    #[test]
    fn ledger_loss_counts_toward_the_daily_budget() {
        let mut f = facts();
        f.ledger_loss_usd = 95_000_000;
        let v = evaluate_trade(&mandate(), &caps(), &ok_checks(), &f);
        assert_eq!(v.first_failure, reason::DAILY_LOSS_BUDGET_EXCEEDED);
    }

    #[test]
    fn calendar_helpers() {
        assert_eq!(month_epoch(0), 1970 * 12 + 1);
        assert_eq!(month_epoch(1_789_430_400), 2026 * 12 + 9); // 2026-09-15
        assert_eq!(weekday_bit(0), 1 << 3); // Thursday
        assert_eq!(weekday_bit(1_789_430_400), 1 << 1); // 2026-09-15 is a Tuesday
        assert!(in_window(0, 86_400, 5));
        assert!(in_window(50_000, 10_000, 55_000));
        assert!(in_window(50_000, 10_000, 5_000));
        assert!(!in_window(50_000, 10_000, 20_000));
        assert_eq!(deviation_bps(101_000_000, 100_000_000), Some(100));
        assert_eq!(deviation_bps(0, 100), None);
    }

    proptest! {
        /// Invariant 1: whatever the caller supplies, an Allow verdict never
        /// contains an enforced check that violates the mandate.
        #[test]
        fn no_allow_violates_the_mandate(
            lev in -1i64..40_000, notional in -1i64..3_000_000_000, buffer in -5_000i64..10_000,
            loss in -1i64..200_000_000, venue in 0u8..4, age in 0u64..5, max_lev in 1u32..30_000,
            max_notional in 1u64..3_000_000_000, min_buffer in 0u32..10_000, max_loss in 0u64..200_000_000,
        ) {
            let mut m = mandate();
            m.max_leverage_bps = max_lev; m.max_notional_usd = max_notional;
            m.min_safety_buffer_bps = min_buffer; m.max_daily_loss_usd = max_loss;
            let supplied = vec![
                Check { rule: rule::LEVERAGE, observed: lev, limit: 0, pass: true },
                Check { rule: rule::NOTIONAL, observed: notional, limit: 0, pass: true },
                Check { rule: rule::SAFETY_BUFFER, observed: buffer, limit: 0, pass: true },
                Check { rule: rule::DAILY_LOSS, observed: loss, limit: 0, pass: true },
            ];
            let f = TradeFacts { venue_id: venue, market: *b"SOL-PERP\0\0\0\0\0\0\0\0", data_slot: 100, now_slot: 100 + age, ledger_loss_usd: 0 };
            let v = evaluate_trade(&m, &caps(), &supplied, &f);
            let violates = !m.venue_allowed(venue) || age > 2
                || lev < 0 || lev > i64::from(max_lev.min(20_000))
                || notional < 0 || notional > i64::try_from(max_notional.min(2_000_000_000)).unwrap()
                || buffer < i64::from(min_buffer)
                || loss > i64::try_from(max_loss).unwrap();
            prop_assert_eq!(v.first_failure == reason::NONE, !violates);
            for c in v.checks.iter().filter(|c| is_enforced_trade_rule(c.rule)) {
                prop_assert!(!(v.first_failure == reason::NONE && !c.pass));
            }
        }
    }
}
