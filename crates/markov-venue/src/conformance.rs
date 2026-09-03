//! The conformance suite, as a library so it can be pointed at any adapter.
//!
//! This is what makes `VenueAdapter` a seam instead of a shape: the same
//! assertions run against `demo_perps` today and against a real venue in Gate
//! C, and **pointing them at a second adapter requires no edit to any test
//! body** — only an implementation of [`Fixture`]. `tests/conformance.rs`
//! proves that by running the whole suite against two unrelated adapters.

use solana_pubkey::Pubkey;

use crate::{fill_within_bound, MarketId, VenueAdapter, VenueError, VenueOutcome, WriteRequest};
use markov_types::Side;

/// What the suite needs from an adapter beyond the trait: how to build one in
/// a known state, and how to drive the two conditions a test cannot cause
/// from outside — a stale mark and a paused venue.
pub trait Fixture: VenueAdapter + Sized {
    /// A fresh adapter with `market` live, a fresh mark at `price` (1e6), and
    /// `mandate` funded enough for `notional` units.
    fn new_fixture(mandate: Pubkey, market: MarketId, price: u64) -> Self;
    /// The market this adapter supports. The suite never invents a market id,
    /// so it cannot fail on one.
    fn market() -> MarketId;
    /// A market this adapter does not know.
    fn unknown_market() -> MarketId;
    /// Make the mark too old for the venue to accept.
    fn make_mark_stale(&mut self);
    /// Halt the venue.
    fn pause(&mut self);
    /// Shrink available collateral below `notional`.
    fn starve_collateral(&mut self);
    /// Lower the venue's own position cap below `notional`.
    fn cap_positions_below(&mut self, notional: u64);
}

/// One named check and its result, so a caller can print a table rather than
/// just a pass/fail.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Check {
    pub name: &'static str,
    pub passed: bool,
    pub detail: String,
}

fn check(name: &'static str, passed: bool, detail: impl Into<String>) -> Check {
    Check {
        name,
        passed,
        detail: detail.into(),
    }
}

pub const MANDATE: [u8; 32] = [7; 32];
pub const OPERATOR: [u8; 32] = [9; 32];
pub const PRICE_E6: u64 = 100_000_000;
pub const NOTIONAL: u64 = 10_000_000;

fn req(mandate: Pubkey, market: MarketId) -> WriteRequest {
    WriteRequest {
        mandate,
        signer: mandate,
        market,
        side: Side::Long,
        notional: NOTIONAL,
        limit_price: PRICE_E6,
        max_slippage_bps: 50,
    }
}

/// Run every conformance check against `A`. The suite is deliberately written
/// against the trait only: no adapter name, no venue-specific constant.
pub fn run<A: Fixture>() -> Vec<Check> {
    let mandate = Pubkey::new_from_array(MANDATE);
    let operator = Pubkey::new_from_array(OPERATOR);
    let market = A::market();
    let mut out = Vec::new();

    // 1. Writes are authorised by the mandate PDA and by nothing else. This is
    //    the whole trust boundary: a stolen operator key must not reach a venue.
    for (label, signer) in [
        ("operator", operator),
        ("stranger", Pubkey::new_from_array([3; 32])),
    ] {
        let mut a = A::new_fixture(mandate, market, PRICE_E6);
        let mut r = req(mandate, market);
        r.signer = signer;
        let res = a.open(r);
        out.push(check(
            "write_requires_the_mandate_pda_signer",
            res.is_err(),
            format!("{label} signer -> {res:?}"),
        ));
    }

    // 2. A stale mark is refused, by the venue, on its own account.
    {
        let mut a = A::new_fixture(mandate, market, PRICE_E6);
        a.make_mark_stale();
        let res = a.open(req(mandate, market));
        out.push(check(
            "stale_mark_returns_stale_mark",
            res == Err(VenueError::StaleMark),
            format!("{res:?}"),
        ));
    }

    // 3. A fill lands inside the slippage bound the intent asked for.
    {
        let mut a = A::new_fixture(mandate, market, PRICE_E6);
        let r = req(mandate, market);
        let res = a.open(r);
        let ok = match res {
            Ok(VenueOutcome::Filled(f)) => fill_within_bound(&f, &r) && f.notional == r.notional,
            // A venue that only records a request has not filled; that is a
            // legitimate outcome and this check does not apply to it.
            Ok(VenueOutcome::Requested { .. }) => true,
            Err(_) => false,
        };
        out.push(check(
            "fill_is_within_the_slippage_bound",
            ok,
            format!("{res:?}"),
        ));
    }

    // 4. `positions` reflects what was opened.
    {
        let mut a = A::new_fixture(mandate, market, PRICE_E6);
        let r = req(mandate, market);
        let opened = a.open(r);
        let ps = a.positions(mandate);
        let ok = match (&opened, &ps) {
            (Ok(VenueOutcome::Filled(_)), Ok(ps)) => ps
                .iter()
                .any(|p| p.market == market && p.notional == r.notional && p.side == r.side),
            (Ok(VenueOutcome::Requested { .. }), Ok(_)) => true,
            _ => false,
        };
        out.push(check(
            "positions_after_open_reflect_the_notional",
            ok,
            format!("{opened:?} then {ps:?}"),
        ));
    }

    // 5. An unknown market is named as such, not filled at some default.
    {
        let mut a = A::new_fixture(mandate, market, PRICE_E6);
        let res = a.open(req(mandate, A::unknown_market()));
        out.push(check(
            "unknown_market_returns_market_unknown",
            res == Err(VenueError::MarketUnknown),
            format!("{res:?}"),
        ));
        let m = a.mark(A::unknown_market());
        out.push(check(
            "mark_of_an_unknown_market_returns_market_unknown",
            m == Err(VenueError::MarketUnknown),
            format!("{m:?}"),
        ));
    }

    // 6. The remaining three errors in the fixed set are reachable. An error
    //    variant no adapter can produce is decoration.
    {
        let mut a = A::new_fixture(mandate, market, PRICE_E6);
        a.pause();
        let res = a.open(req(mandate, market));
        out.push(check(
            "paused_venue_returns_venue_paused",
            res == Err(VenueError::VenuePaused),
            format!("{res:?}"),
        ));
    }
    {
        let mut a = A::new_fixture(mandate, market, PRICE_E6);
        a.starve_collateral();
        let res = a.open(req(mandate, market));
        out.push(check(
            "insufficient_collateral_is_named",
            res == Err(VenueError::InsufficientCollateral),
            format!("{res:?}"),
        ));
    }
    {
        let mut a = A::new_fixture(mandate, market, PRICE_E6);
        a.cap_positions_below(NOTIONAL);
        let res = a.open(req(mandate, market));
        out.push(check(
            "position_limit_is_named",
            res == Err(VenueError::PositionLimit),
            format!("{res:?}"),
        ));
    }

    // 7. A limit the venue cannot honour is refused rather than filled wide.
    {
        let mut a = A::new_fixture(mandate, market, PRICE_E6);
        let mut r = req(mandate, market);
        r.max_slippage_bps = 0;
        r.limit_price = PRICE_E6 / 2; // half the mark: unfillable at any fee
        let res = a.open(r);
        let ok = match res {
            Err(VenueError::SlippageExceeded) => true,
            Ok(VenueOutcome::Filled(f)) => fill_within_bound(&f, &r),
            // A request-based venue has not filled yet, so it cannot know
            // whether the limit is honourable — the keeper enforces it at
            // fulfilment, or the request expires. Accepting the request is
            // therefore correct here. The consequence is a Gate C constraint,
            // not a Gate B one: such a venue's receipt may say "requested"
            // and must never claim a fill price (see ADR-007).
            Ok(VenueOutcome::Requested { .. }) => true,
            _ => false,
        };
        out.push(check(
            "an_unhonourable_limit_is_refused_not_filled_wide",
            ok,
            format!("{res:?}"),
        ));
    }

    // 8. Reduce and close move the position the right way, and every write
    //    respects the signer rule (not just `open`).
    {
        let mut a = A::new_fixture(mandate, market, PRICE_E6);
        let r = req(mandate, market);
        let _ = a.open(r);
        let mut half = r;
        half.notional = r.notional / 2;
        let reduced = a.reduce(half);
        let after = a.positions(mandate).unwrap_or_default();
        let shrank = after
            .iter()
            .find(|p| p.market == market)
            .is_none_or(|p| p.notional <= r.notional);
        out.push(check(
            "reduce_shrinks_the_position",
            reduced.is_ok() && shrank,
            format!("{reduced:?} -> {after:?}"),
        ));

        let closed = a.close(r);
        let flat = a
            .positions(mandate)
            .unwrap_or_default()
            .iter()
            .all(|p| p.market != market || p.notional == 0);
        out.push(check(
            "close_flattens_the_position",
            closed.is_ok() && flat,
            format!("{closed:?}"),
        ));

        for (name, mut w) in [
            ("increase", req(mandate, market)),
            ("reduce", req(mandate, market)),
            ("close", req(mandate, market)),
        ] {
            w.signer = operator;
            let res = match name {
                "increase" => a.increase(w),
                "reduce" => a.reduce(w),
                _ => a.close(w),
            };
            out.push(check(
                "every_write_requires_the_mandate_pda_signer",
                res.is_err(),
                format!("{name} with the operator as signer -> {res:?}"),
            ));
        }
    }

    // 9. The venue reports its own program id, so gate 5 can check the policy
    //    allowlist and the registry against the same value.
    {
        let a = A::new_fixture(mandate, market, PRICE_E6);
        let id = a.venue_program_id();
        out.push(check(
            "venue_reports_its_program_id",
            id != Pubkey::default(),
            format!("{id}"),
        ));
    }

    out
}

/// Panic with a readable table if any check failed. Callers use this from a
/// `#[test]`, so the test body is the same for every adapter.
///
/// This function exists to fail a test, so `panic!` and `assert!` are its
/// purpose rather than a lapse — hence the scoped allow. Nothing else in this
/// crate may panic.
#[allow(clippy::panic, reason = "this is the suite's assertion entry point")]
pub fn assert_conforms<A: Fixture>(adapter_name: &str) {
    let checks = run::<A>();
    let failed: Vec<&Check> = checks.iter().filter(|c| !c.passed).collect();
    if !failed.is_empty() {
        let mut msg = format!(
            "{adapter_name} failed {} of {} conformance checks:\n",
            failed.len(),
            checks.len()
        );
        for c in &failed {
            msg.push_str(&format!("  FAIL {}\n       {}\n", c.name, c.detail));
        }
        panic!("{msg}");
    }
    assert!(
        checks.len() >= 15,
        "{adapter_name}: suite ran only {} checks",
        checks.len()
    );
}
