//! The mark, and the only way the program is allowed to read a price.
//!
//! ADR-003: the mark is a Pyth `PriceUpdateV2` account on devnet. Three things
//! bind it, and all three are enforced here, not assumed:
//!   * the account is owned by the Pyth receiver program (Anchor's
//!     `Account<'info, PriceUpdateV2>` checks the owner),
//!   * it is the account the mandate names (`mandate.mark_account`),
//!   * it carries the mandate's feed id and `VerificationLevel::Full`, and it
//!     is younger than `policy.max_mark_age_secs` (the SDK checks all three).
//!
//! A mark that fails any of these is not a price. There is no default.

use anchor_lang::prelude::*;
use pyth_solana_receiver_sdk::price_update::PriceUpdateV2;

pub struct BoundMark {
    /// Price in the feed's own exponent.
    pub price: i64,
    pub exponent: i32,
    pub publish_time: i64,
}

impl BoundMark {
    /// Price scaled to 1e6 per unit, saturating. Receipts carry `u64`, so a
    /// negative or absurd price becomes 0 rather than wrapping.
    pub fn price_e6(&self) -> u64 {
        if self.price <= 0 {
            return 0;
        }
        let p = self.price as i128;
        let scaled = match self.exponent.checked_add(6) {
            Some(e) if e >= 0 => p.saturating_mul(10i128.saturating_pow(e.min(30) as u32)),
            Some(e) => p / 10i128.saturating_pow((-e).min(30) as u32),
            None => 0,
        };
        u64::try_from(scaled.max(0)).unwrap_or(u64::MAX)
    }
}

/// Read the mark, or refuse. `None` means the gate must return
/// `StaleOracle`; it never means "use the last price". There is no error
/// detail on purpose — every way of failing to bind a mark is the same
/// refusal, and the reasons are the SDK's to report, not ours to reinterpret.
pub fn read_bound_mark(
    price_update: &Account<'_, PriceUpdateV2>,
    expected_account: &Pubkey,
    feed_id: &[u8; 32],
    max_age_secs: u64,
    clock: &Clock,
) -> Option<BoundMark> {
    if price_update.key() != *expected_account {
        return None;
    }
    price_update
        .get_price_no_older_than(clock, max_age_secs, feed_id)
        .ok()
        .map(|p| BoundMark {
            price: p.price,
            exponent: p.exponent,
            publish_time: p.publish_time,
        })
}
