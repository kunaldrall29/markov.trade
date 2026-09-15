//! On-chain re-validation of supplied observed values against the active mandate.
//! The control plane computes the same checks; the program is the last word.

use crate::ids::*;
use crate::state::Mandate;
// Check lives in ids.

#[derive(Clone, Debug)]
pub struct Observed {
    pub venue_id: u8,
    pub market_id: [u8; 16],
    pub data_age_ms: i64,
    pub freshness_limit_ms: i64,
    pub projected_leverage_bps: i64,
    pub projected_notional_usd: i64,
    pub safety_buffer_bps: i64,
    pub daily_loss_usd: i64,
    pub slippage_bps: i64,
    pub slippage_limit_bps: i64,
}

pub fn evaluate_mandate(mandate: &Mandate, obs: &Observed) -> (u16, Vec<Check>, bool) {
    let mut checks = Vec::with_capacity(8);
    let mut first_fail: u16 = 0;

    let venue_ok = venue_bit(obs.venue_id)
        .map(|b| mandate.allowed_venues & b != 0)
        .unwrap_or(false);
    push(
        &mut checks,
        &mut first_fail,
        10,
        REASON_VENUE,
        obs.venue_id as i64,
        mandate.allowed_venues as i64,
        venue_ok,
    );

    let market_ok = mandate
        .approved_markets
        .iter()
        .any(|m| m == &obs.market_id);
    push(
        &mut checks,
        &mut first_fail,
        1,
        REASON_MARKET_NOT_ALLOWED,
        i64::from_le_bytes(obs.market_id[0..8].try_into().unwrap_or([0; 8])),
        mandate.approved_markets.len() as i64,
        market_ok,
    );

    let fresh = obs.data_age_ms <= obs.freshness_limit_ms;
    push(
        &mut checks,
        &mut first_fail,
        6,
        REASON_STALE,
        obs.data_age_ms,
        obs.freshness_limit_ms,
        fresh,
    );

    let lev_ok = obs.projected_leverage_bps <= mandate.max_leverage_bps as i64;
    push(
        &mut checks,
        &mut first_fail,
        2,
        REASON_MAX_LEVERAGE,
        obs.projected_leverage_bps,
        mandate.max_leverage_bps as i64,
        lev_ok,
    );

    let notional_ok = obs.projected_notional_usd <= mandate.max_notional_usd as i64;
    push(
        &mut checks,
        &mut first_fail,
        3,
        REASON_MAX_NOTIONAL,
        obs.projected_notional_usd,
        mandate.max_notional_usd as i64,
        notional_ok,
    );

    let buf_ok = obs.safety_buffer_bps >= mandate.min_safety_buffer_bps as i64;
    push(
        &mut checks,
        &mut first_fail,
        4,
        REASON_MIN_SAFETY_BUFFER,
        obs.safety_buffer_bps,
        mandate.min_safety_buffer_bps as i64,
        buf_ok,
    );

    let loss_ok = obs.daily_loss_usd <= mandate.max_daily_loss_usd as i64;
    push(
        &mut checks,
        &mut first_fail,
        5,
        REASON_DAILY_LOSS,
        obs.daily_loss_usd,
        mandate.max_daily_loss_usd as i64,
        loss_ok,
    );

    let slip_ok = obs.slippage_bps <= obs.slippage_limit_bps;
    push(
        &mut checks,
        &mut first_fail,
        7,
        REASON_SLIPPAGE,
        obs.slippage_bps,
        obs.slippage_limit_bps,
        slip_ok,
    );

    (first_fail, checks, first_fail == 0)
}

fn push(
    checks: &mut Vec<Check>,
    first_fail: &mut u16,
    rule: u8,
    reason: u16,
    observed: i64,
    limit: i64,
    pass: bool,
) {
    checks.push(Check {
        rule,
        observed,
        limit,
        pass,
    });
    if !pass && *first_fail == 0 {
        *first_fail = reason;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::Mandate;
    use anchor_lang::prelude::Pubkey;

    fn mandate() -> Mandate {
        let mut mkt = [0u8; 16];
        mkt[..8].copy_from_slice(b"SOL-PERP");
        Mandate {
            account: Pubkey::default(),
            version: 1,
            hash: [0; 32],
            max_leverage_bps: 20_000,
            max_notional_usd: 500_000_000,
            min_safety_buffer_bps: 2_000,
            max_daily_loss_usd: 50_000_000,
            approved_markets: vec![mkt],
            tier_caps: [20_000; 5],
            allowed_venues: VENUE_BIT_PACIFICA | VENUE_BIT_DRIFT,
            activated_slot: 0,
            activated_by: Pubkey::default(),
            bump: 0,
        }
    }

    fn ok_obs() -> Observed {
        let mut mkt = [0u8; 16];
        mkt[..8].copy_from_slice(b"SOL-PERP");
        Observed {
            venue_id: VENUE_PACIFICA,
            market_id: mkt,
            data_age_ms: 400,
            freshness_limit_ms: 3_000,
            projected_leverage_bps: 15_000,
            projected_notional_usd: 50_000_000,
            safety_buffer_bps: 5_000,
            daily_loss_usd: 0,
            slippage_bps: 8,
            slippage_limit_bps: 30,
        }
    }

    #[test]
    fn allow_when_inside() {
        let (reason, checks, ok) = evaluate_mandate(&mandate(), &ok_obs());
        assert!(ok, "{reason}");
        assert!(checks.iter().all(|c| c.pass));
        assert_eq!(reason, 0);
    }

    #[test]
    fn leverage_rejects() {
        let mut o = ok_obs();
        o.projected_leverage_bps = 30_000;
        let (reason, _, ok) = evaluate_mandate(&mandate(), &o);
        assert!(!ok);
        assert_eq!(reason, REASON_MAX_LEVERAGE);
    }

    #[test]
    fn notional_rejects() {
        let mut o = ok_obs();
        o.projected_notional_usd = 9_000_000_000;
        let (reason, _, ok) = evaluate_mandate(&mandate(), &o);
        assert!(!ok);
        assert_eq!(reason, REASON_MAX_NOTIONAL);
    }

    #[test]
    fn market_rejects() {
        let mut o = ok_obs();
        o.market_id = *b"DOGE-PERP\0\0\0\0\0\0\0";
        let (reason, _, ok) = evaluate_mandate(&mandate(), &o);
        assert!(!ok);
        assert_eq!(reason, REASON_MARKET_NOT_ALLOWED);
    }

    #[test]
    fn phoenix_not_allowed_until_bitmap_set() {
        let mut o = ok_obs();
        o.venue_id = VENUE_PHOENIX;
        let (reason, _, ok) = evaluate_mandate(&mandate(), &o);
        assert!(!ok);
        assert_eq!(reason, REASON_VENUE);
    }

    #[test]
    fn stale_rejects() {
        let mut o = ok_obs();
        o.data_age_ms = 10_000;
        let (reason, _, ok) = evaluate_mandate(&mandate(), &o);
        assert!(!ok);
        assert_eq!(reason, REASON_STALE);
    }

    #[test]
    fn never_allow_on_failed_check() {
        let m = mandate();
        let mut o = ok_obs();
        o.safety_buffer_bps = 100;
        let (reason, checks, ok) = evaluate_mandate(&m, &o);
        assert!(!ok);
        assert_eq!(reason, REASON_MIN_SAFETY_BUFFER);
        assert!(checks.iter().any(|c| c.rule == 4 && !c.pass));
    }

    #[test]
    fn slippage_rejects() {
        let mut o = ok_obs();
        o.slippage_bps = 80;
        o.slippage_limit_bps = 30;
        let (reason, _, ok) = evaluate_mandate(&mandate(), &o);
        assert!(!ok);
        assert_eq!(reason, REASON_SLIPPAGE);
    }

    #[test]
    fn daily_loss_rejects() {
        let mut o = ok_obs();
        o.daily_loss_usd = 90_000_000;
        let (reason, _, ok) = evaluate_mandate(&mandate(), &o);
        assert!(!ok);
        assert_eq!(reason, REASON_DAILY_LOSS);
    }
}
