//! B11: the conformance suite runs against more than one adapter, with the
//! **same test bodies**, to prove it is a contract and not a description of
//! one implementation.
//!
//! Three adapters, deliberately unalike:
//!   * `DemoPerpsClient` — mirrors the on-chain mock: deterministic fill at
//!     `mark ± fee_bps`, positions in a vector.
//!   * `ExactFillVenue` — synchronous but different in every detail: fills at
//!     the mark with no fee, keeps positions in a map, different program id,
//!     different collateral accounting.
//!   * `RequestVenue` — takes a *request* a keeper would fulfil later, so its
//!     writes return `Requested` and never a fill. This is the shape a real
//!     Solana perp venue can have, and it is here to prove the suite does not
//!     assume synchronous fills.
//!
//! Adding a fourth adapter means implementing `Fixture` and adding one
//! `#[test]` that calls `assert_conforms`. No assertion changes.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::collections::HashMap;

use markov_types::Side;
use markov_venue::conformance::{assert_conforms, Fixture, NOTIONAL};
use markov_venue::{
    market_id, Fill, Mark, MarketId, Position, VenueAdapter, VenueError, VenueOutcome, WriteRequest,
};
use solana_pubkey::Pubkey;

const NOW: i64 = 1_788_400_000;

/// State every adapter happens to need. Shared so the adapters differ in
/// behaviour rather than in bookkeeping boilerplate.
struct Common {
    mandate: Pubkey,
    market: MarketId,
    price: u64,
    mark_age_secs: i64,
    paused: bool,
    collateral: u64,
    position_cap: u64,
}

impl Common {
    fn new(mandate: Pubkey, market: MarketId, price: u64) -> Self {
        Self {
            mandate,
            market,
            price,
            mark_age_secs: 5,
            paused: false,
            collateral: NOTIONAL * 10,
            position_cap: NOTIONAL * 10,
        }
    }

    /// The checks every adapter must make before it does anything, in the
    /// order that makes a refusal honest: who is asking, is the venue open,
    /// does the market exist, is the mark usable, can it be paid for.
    fn guard(&self, req: &WriteRequest, max_age: i64) -> Result<(), VenueError> {
        if req.signer != req.mandate || req.mandate != self.mandate {
            // Not in the fixed error set on purpose: an unauthorised caller is
            // not a venue condition, it is a rejection. Gate 13 turns any
            // venue refusal into `VenueRejected`.
            return Err(VenueError::VenuePaused);
        }
        if self.paused {
            return Err(VenueError::VenuePaused);
        }
        if req.market != self.market {
            return Err(VenueError::MarketUnknown);
        }
        if self.mark_age_secs > max_age {
            return Err(VenueError::StaleMark);
        }
        if req.notional > self.collateral {
            return Err(VenueError::InsufficientCollateral);
        }
        if req.notional > self.position_cap {
            return Err(VenueError::PositionLimit);
        }
        Ok(())
    }

    fn mark(&self) -> Mark {
        Mark {
            price: self.price as i64,
            expo: -6,
            publish_time: NOW - self.mark_age_secs,
            slot: 1_000,
        }
    }
}

// ─────────────────────────── adapter 1: the mock ───────────────────────────

struct DemoPerpsClient {
    c: Common,
    fee_bps: u64,
    positions: Vec<Position>,
}

impl DemoPerpsClient {
    const MAX_MARK_AGE: i64 = 30;
}

impl VenueAdapter for DemoPerpsClient {
    fn venue_program_id(&self) -> Pubkey {
        Pubkey::new_from_array([11; 32])
    }
    fn mark(&self, market: MarketId) -> Result<Mark, VenueError> {
        if market != self.c.market {
            return Err(VenueError::MarketUnknown);
        }
        Ok(self.c.mark())
    }
    fn positions(&self, mandate: Pubkey) -> Result<Vec<Position>, VenueError> {
        if mandate != self.c.mandate {
            return Ok(Vec::new());
        }
        Ok(self.positions.clone())
    }
    fn open(&mut self, req: WriteRequest) -> Result<VenueOutcome, VenueError> {
        self.c.guard(&req, Self::MAX_MARK_AGE)?;
        // Deterministic: mark plus the fee, never randomised, never generous.
        let price = self.c.price + self.c.price * self.fee_bps / 10_000;
        let fill = Fill {
            price,
            notional: req.notional,
            fee: req.notional * self.fee_bps / 10_000,
        };
        if !markov_venue::fill_within_bound(&fill, &req) {
            return Err(VenueError::SlippageExceeded);
        }
        self.positions.push(Position {
            market: req.market,
            side: req.side,
            notional: req.notional,
            entry_price: price,
            funding_accrued: 0,
            updated_slot: 1_000,
        });
        Ok(VenueOutcome::Filled(fill))
    }
    fn increase(&mut self, req: WriteRequest) -> Result<VenueOutcome, VenueError> {
        let out = self.open(req)?;
        Ok(out)
    }
    fn reduce(&mut self, req: WriteRequest) -> Result<VenueOutcome, VenueError> {
        self.c.guard(&req, Self::MAX_MARK_AGE)?;
        let price = self.c.price;
        for p in self.positions.iter_mut().filter(|p| p.market == req.market) {
            p.notional = p.notional.saturating_sub(req.notional);
        }
        self.positions.retain(|p| p.notional > 0);
        Ok(VenueOutcome::Filled(Fill {
            price,
            notional: req.notional,
            fee: 0,
        }))
    }
    fn close(&mut self, req: WriteRequest) -> Result<VenueOutcome, VenueError> {
        self.c.guard(&req, Self::MAX_MARK_AGE)?;
        let closed: u64 = self
            .positions
            .iter()
            .filter(|p| p.market == req.market)
            .map(|p| p.notional)
            .sum();
        self.positions.retain(|p| p.market != req.market);
        Ok(VenueOutcome::Filled(Fill {
            price: self.c.price,
            notional: closed,
            fee: 0,
        }))
    }
}

impl Fixture for DemoPerpsClient {
    fn new_fixture(mandate: Pubkey, market: MarketId, price: u64) -> Self {
        Self {
            c: Common::new(mandate, market, price),
            fee_bps: 10,
            positions: Vec::new(),
        }
    }
    fn market() -> MarketId {
        market_id("SOL-PERP").expect("fits")
    }
    fn unknown_market() -> MarketId {
        market_id("NOPE-PERP").expect("fits")
    }
    fn make_mark_stale(&mut self) {
        self.c.mark_age_secs = Self::MAX_MARK_AGE + 1;
    }
    fn pause(&mut self) {
        self.c.paused = true;
    }
    fn starve_collateral(&mut self) {
        self.c.collateral = 1;
    }
    fn cap_positions_below(&mut self, notional: u64) {
        self.c.position_cap = notional.saturating_sub(1);
    }
}

// ──────────────── adapter 2: synchronous, different in every detail ────────

struct ExactFillVenue {
    c: Common,
    book: HashMap<[u8; 16], (Side, u64, u64)>,
}

impl ExactFillVenue {
    const MAX_MARK_AGE: i64 = 90;
}

impl VenueAdapter for ExactFillVenue {
    fn venue_program_id(&self) -> Pubkey {
        Pubkey::new_from_array([22; 32])
    }
    fn mark(&self, market: MarketId) -> Result<Mark, VenueError> {
        if market != self.c.market {
            return Err(VenueError::MarketUnknown);
        }
        Ok(self.c.mark())
    }
    fn positions(&self, mandate: Pubkey) -> Result<Vec<Position>, VenueError> {
        if mandate != self.c.mandate {
            return Ok(Vec::new());
        }
        Ok(self
            .book
            .iter()
            .map(|(m, (side, notional, entry))| Position {
                market: *m,
                side: *side,
                notional: *notional,
                entry_price: *entry,
                funding_accrued: -1,
                updated_slot: 2_000,
            })
            .collect())
    }
    fn open(&mut self, req: WriteRequest) -> Result<VenueOutcome, VenueError> {
        self.c.guard(&req, Self::MAX_MARK_AGE)?;
        let fill = Fill {
            price: self.c.price,
            notional: req.notional,
            fee: 0,
        };
        if !markov_venue::fill_within_bound(&fill, &req) {
            return Err(VenueError::SlippageExceeded);
        }
        let e = self
            .book
            .entry(req.market)
            .or_insert((req.side, 0, self.c.price));
        e.1 += req.notional;
        Ok(VenueOutcome::Filled(fill))
    }
    fn increase(&mut self, req: WriteRequest) -> Result<VenueOutcome, VenueError> {
        self.open(req)
    }
    fn reduce(&mut self, req: WriteRequest) -> Result<VenueOutcome, VenueError> {
        self.c.guard(&req, Self::MAX_MARK_AGE)?;
        if let Some(e) = self.book.get_mut(&req.market) {
            e.1 = e.1.saturating_sub(req.notional);
            if e.1 == 0 {
                self.book.remove(&req.market);
            }
        }
        Ok(VenueOutcome::Filled(Fill {
            price: self.c.price,
            notional: req.notional,
            fee: 0,
        }))
    }
    fn close(&mut self, req: WriteRequest) -> Result<VenueOutcome, VenueError> {
        self.c.guard(&req, Self::MAX_MARK_AGE)?;
        let n = self.book.remove(&req.market).map(|e| e.1).unwrap_or(0);
        Ok(VenueOutcome::Filled(Fill {
            price: self.c.price,
            notional: n,
            fee: 0,
        }))
    }
}

impl Fixture for ExactFillVenue {
    fn new_fixture(mandate: Pubkey, market: MarketId, price: u64) -> Self {
        Self {
            c: Common::new(mandate, market, price),
            book: HashMap::new(),
        }
    }
    fn market() -> MarketId {
        market_id("SOL-PERP").expect("fits")
    }
    fn unknown_market() -> MarketId {
        market_id("OTHER").expect("fits")
    }
    fn make_mark_stale(&mut self) {
        self.c.mark_age_secs = Self::MAX_MARK_AGE + 1;
    }
    fn pause(&mut self) {
        self.c.paused = true;
    }
    fn starve_collateral(&mut self) {
        self.c.collateral = 0;
    }
    fn cap_positions_below(&mut self, notional: u64) {
        self.c.position_cap = notional / 2;
    }
}

// ───────────── adapter 3: request/fulfil, the asynchronous shape ───────────

struct RequestVenue {
    c: Common,
    next: u8,
}

impl RequestVenue {
    const MAX_MARK_AGE: i64 = 20;
    fn request(&mut self, req: WriteRequest) -> Result<VenueOutcome, VenueError> {
        self.c.guard(&req, Self::MAX_MARK_AGE)?;
        self.next = self.next.wrapping_add(1);
        // No fill is invented: a keeper settles this later, and the receipt
        // must say "requested" rather than claim a price that does not exist.
        Ok(VenueOutcome::Requested {
            request_id: [self.next; 32],
        })
    }
}

impl VenueAdapter for RequestVenue {
    fn venue_program_id(&self) -> Pubkey {
        Pubkey::new_from_array([33; 32])
    }
    fn mark(&self, market: MarketId) -> Result<Mark, VenueError> {
        if market != self.c.market {
            return Err(VenueError::MarketUnknown);
        }
        Ok(self.c.mark())
    }
    fn positions(&self, _mandate: Pubkey) -> Result<Vec<Position>, VenueError> {
        // Nothing is settled until a keeper acts, so there is no position yet.
        Ok(Vec::new())
    }
    fn open(&mut self, req: WriteRequest) -> Result<VenueOutcome, VenueError> {
        self.request(req)
    }
    fn increase(&mut self, req: WriteRequest) -> Result<VenueOutcome, VenueError> {
        self.request(req)
    }
    fn reduce(&mut self, req: WriteRequest) -> Result<VenueOutcome, VenueError> {
        self.request(req)
    }
    fn close(&mut self, req: WriteRequest) -> Result<VenueOutcome, VenueError> {
        self.request(req)
    }
}

impl Fixture for RequestVenue {
    fn new_fixture(mandate: Pubkey, market: MarketId, price: u64) -> Self {
        Self {
            c: Common::new(mandate, market, price),
            next: 0,
        }
    }
    fn market() -> MarketId {
        market_id("SOL-PERP").expect("fits")
    }
    fn unknown_market() -> MarketId {
        market_id("ASYNC-NOPE").expect("fits")
    }
    fn make_mark_stale(&mut self) {
        self.c.mark_age_secs = Self::MAX_MARK_AGE + 1;
    }
    fn pause(&mut self) {
        self.c.paused = true;
    }
    fn starve_collateral(&mut self) {
        self.c.collateral = 1;
    }
    fn cap_positions_below(&mut self, notional: u64) {
        self.c.position_cap = notional.saturating_sub(1);
    }
}

// ───────────────────────────── the tests ──────────────────────────────────
// Identical bodies. That is the point.

#[test]
fn demo_perps_conforms() {
    assert_conforms::<DemoPerpsClient>("DemoPerpsClient");
}

#[test]
fn a_second_unrelated_adapter_conforms() {
    assert_conforms::<ExactFillVenue>("ExactFillVenue");
}

#[test]
fn a_request_fulfil_adapter_conforms() {
    assert_conforms::<RequestVenue>("RequestVenue");
}

/// The suite would be worthless if it passed against an adapter that ignores
/// the rules, so prove it fails: this one fills anything, from any signer, at
/// any price.
#[test]
fn the_suite_rejects_an_adapter_that_ignores_the_rules() {
    struct Rogue {
        c: Common,
    }
    impl VenueAdapter for Rogue {
        fn venue_program_id(&self) -> Pubkey {
            Pubkey::default() // does not even identify itself
        }
        fn mark(&self, _m: MarketId) -> Result<Mark, VenueError> {
            Ok(self.c.mark())
        }
        fn positions(&self, _m: Pubkey) -> Result<Vec<Position>, VenueError> {
            Ok(Vec::new())
        }
        fn open(&mut self, req: WriteRequest) -> Result<VenueOutcome, VenueError> {
            // Fills for anyone, at a price of its choosing.
            Ok(VenueOutcome::Filled(Fill {
                price: 1,
                notional: req.notional,
                fee: 0,
            }))
        }
        fn increase(&mut self, r: WriteRequest) -> Result<VenueOutcome, VenueError> {
            self.open(r)
        }
        fn reduce(&mut self, r: WriteRequest) -> Result<VenueOutcome, VenueError> {
            self.open(r)
        }
        fn close(&mut self, r: WriteRequest) -> Result<VenueOutcome, VenueError> {
            self.open(r)
        }
    }
    impl Fixture for Rogue {
        fn new_fixture(mandate: Pubkey, market: MarketId, price: u64) -> Self {
            Self {
                c: Common::new(mandate, market, price),
            }
        }
        fn market() -> MarketId {
            market_id("SOL-PERP").expect("fits")
        }
        fn unknown_market() -> MarketId {
            market_id("X").expect("fits")
        }
        fn make_mark_stale(&mut self) {}
        fn pause(&mut self) {}
        fn starve_collateral(&mut self) {}
        fn cap_positions_below(&mut self, _n: u64) {}
    }

    let checks = markov_venue::conformance::run::<Rogue>();
    let failed: Vec<_> = checks
        .iter()
        .filter(|c| !c.passed)
        .map(|c| c.name)
        .collect();
    for expected in [
        "write_requires_the_mandate_pda_signer",
        "stale_mark_returns_stale_mark",
        "unknown_market_returns_market_unknown",
        "paused_venue_returns_venue_paused",
        "venue_reports_its_program_id",
    ] {
        assert!(
            failed.contains(&expected),
            "the suite let a rogue adapter pass {expected}: {failed:?}"
        );
    }
}
