//! `VenueAdapter` — the one interface `demo_perps` implements today and a real
//! venue implements in Gate C. This is **B11**.
//!
//! Two consumers, one shape: the mandate program CPIs into a venue using the
//! encoding in `programs/markov-mandate/src/cpi/venue.rs`, and this crate
//! mirrors the same calls off chain so the agent can quote before it proposes.
//! If the two ever disagree, the conformance suite in `tests/` is what catches
//! it.
//!
//! Four rules make this a seam rather than a stub:
//!
//! 1. **Every write is authorised by the mandate PDA.** The operator key never
//!    signs to a venue. An adapter that accepts any other signer fails
//!    conformance.
//! 2. **No caller-supplied price.** A write carries a *limit*, which is a
//!    bound, and the mark comes from an account the venue reads itself.
//! 3. **Fixed-width market ids.** Never a `&str`, so a market cannot be
//!    conjured by spelling.
//! 4. **A write reports what actually happened.** It returns `Filled` with a
//!    fill the program can post-check, or `Requested` when the venue only
//!    accepted a request that a keeper settles later. It may never invent a
//!    fill, because a receipt carrying an invented fill is a lie with a
//!    signature on it.
#![forbid(unsafe_code)]
#![warn(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

pub mod conformance;

use markov_types::Side;
use solana_pubkey::Pubkey;

/// Fixed-width market id, e.g. `SOL-PERP` padded with zeros.
pub type MarketId = [u8; 16];

/// Build a market id from a short name. Names longer than 16 bytes are
/// rejected rather than truncated, so two markets can never collide.
pub fn market_id(name: &str) -> Option<MarketId> {
    let b = name.as_bytes();
    if b.len() > 16 {
        return None;
    }
    let mut out = [0u8; 16];
    out[..b.len()].copy_from_slice(b);
    Some(out)
}

pub fn market_name(id: &MarketId) -> &str {
    let end = id.iter().position(|b| *b == 0).unwrap_or(id.len());
    core::str::from_utf8(&id[..end]).unwrap_or("<invalid>")
}

/// A mark, as the venue itself read it. `publish_time` and `slot` are both
/// carried because freshness is judged in seconds (ADR-003) while the venue's
/// own staleness rule may be in slots.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Mark {
    pub price: i64,
    pub expo: i32,
    pub publish_time: i64,
    pub slot: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Position {
    pub market: MarketId,
    pub side: Side,
    /// In settlement-mint base units.
    pub notional: u64,
    /// Scaled 1e6 per unit, the same scale as a limit price.
    pub entry_price: u64,
    /// Signed: negative means the position paid funding.
    pub funding_accrued: i64,
    pub updated_slot: u64,
}

/// What a write actually did. The program post-checks a `Fill`; it cannot
/// post-check a promise.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Fill {
    /// Scaled 1e6 per unit.
    pub price: u64,
    pub notional: u64,
    pub fee: u64,
}

/// The honest outcome of a venue write.
///
/// `demo_perps` is synchronous and only ever returns `Filled`, which is what
/// Gate B needs. `Requested` exists because some real Solana perp venues take
/// a position *request* that a keeper fulfils in a later transaction: at CPI
/// time there is no fill, and the alternative to naming that is fabricating
/// one. No Gate B code handles `Requested`; the variant is here so a Gate C
/// adapter cannot quietly lie instead.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VenueOutcome {
    Filled(Fill),
    Requested { request_id: [u8; 32] },
}

/// The fixed error set. A real venue's error space maps onto these; anything
/// unmapped becomes `BlockReason::VenueRejected` at gate 13 rather than being
/// guessed at.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum VenueError {
    #[error("market unknown to this venue")]
    MarketUnknown,
    #[error("the venue's mark is stale")]
    StaleMark,
    #[error("fill would breach the limit price")]
    SlippageExceeded,
    #[error("not enough collateral")]
    InsufficientCollateral,
    #[error("position limit reached")]
    PositionLimit,
    #[error("venue is paused")]
    VenuePaused,
}

impl VenueError {
    /// Every venue error reaches the chain as one `BlockReason`. The mandate
    /// program does not re-interpret a venue's refusal; it records that the
    /// venue refused, and the venue's own error is the detail.
    pub const fn block_reason(self) -> markov_types::BlockReason {
        markov_types::BlockReason::VenueRejected
    }
}

/// One venue write, fully specified. `signer` is the authority the venue must
/// insist on: the mandate PDA, never the operator.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WriteRequest {
    pub mandate: Pubkey,
    /// Who authorised this call. Conformance requires `signer == mandate`.
    pub signer: Pubkey,
    pub market: MarketId,
    pub side: Side,
    /// In settlement-mint base units. Ignored by `close`, which flattens.
    pub notional: u64,
    /// A bound, not a price. Scaled 1e6 per unit.
    pub limit_price: u64,
    pub max_slippage_bps: u16,
}

/// The seam. `demo_perps` implements it now; a real venue implements it in
/// Gate C without any consumer changing.
pub trait VenueAdapter {
    /// The venue's program id, so gate 5 can check policy **and** registry.
    fn venue_program_id(&self) -> Pubkey;

    fn mark(&self, market: MarketId) -> Result<Mark, VenueError>;

    fn positions(&self, mandate: Pubkey) -> Result<Vec<Position>, VenueError>;

    fn open(&mut self, req: WriteRequest) -> Result<VenueOutcome, VenueError>;
    fn increase(&mut self, req: WriteRequest) -> Result<VenueOutcome, VenueError>;
    fn reduce(&mut self, req: WriteRequest) -> Result<VenueOutcome, VenueError>;
    fn close(&mut self, req: WriteRequest) -> Result<VenueOutcome, VenueError>;
}

/// What a fill must satisfy for the mandate program to accept it: the price
/// has to sit inside the slippage bound the intent asked for. A venue that
/// fills outside the bound has not honoured the intent, whatever it reports.
pub fn fill_within_bound(fill: &Fill, req: &WriteRequest) -> bool {
    if req.limit_price == 0 {
        return false;
    }
    let diff = fill.price.abs_diff(req.limit_price) as u128;
    let bound = (req.limit_price as u128).saturating_mul(req.max_slippage_bps as u128) / 10_000;
    diff <= bound
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn market_ids_are_fixed_width_and_cannot_collide_by_truncation() {
        assert_eq!(
            market_name(&market_id("SOL-PERP").expect("fits")),
            "SOL-PERP"
        );
        // 16 bytes exactly is fine; 17 is refused rather than silently cut,
        // which is what would let two markets share an id.
        assert!(market_id("0123456789ABCDEF").is_some());
        assert!(market_id("0123456789ABCDEFG").is_none());
    }

    #[test]
    fn a_fill_outside_the_bound_is_not_acceptable() {
        let req = WriteRequest {
            mandate: Pubkey::new_from_array([1; 32]),
            signer: Pubkey::new_from_array([1; 32]),
            market: market_id("SOL-PERP").expect("fits"),
            side: Side::Long,
            notional: 10,
            limit_price: 100_000_000,
            max_slippage_bps: 50,
        };
        // 50 bps of 100.000000 is 0.5
        assert!(fill_within_bound(
            &Fill {
                price: 100_400_000,
                notional: 10,
                fee: 1
            },
            &req
        ));
        assert!(!fill_within_bound(
            &Fill {
                price: 100_600_000,
                notional: 10,
                fee: 1
            },
            &req
        ));
        // A zero limit is not a bound, so nothing satisfies it.
        let mut zero = req;
        zero.limit_price = 0;
        assert!(!fill_within_bound(
            &Fill {
                price: 1,
                notional: 10,
                fee: 1
            },
            &zero
        ));
    }

    #[test]
    fn every_venue_error_reaches_the_chain_as_venue_rejected() {
        for e in [
            VenueError::MarketUnknown,
            VenueError::StaleMark,
            VenueError::SlippageExceeded,
            VenueError::InsufficientCollateral,
            VenueError::PositionLimit,
            VenueError::VenuePaused,
        ] {
            assert_eq!(e.block_reason(), markov_types::BlockReason::VenueRejected);
        }
    }
}
