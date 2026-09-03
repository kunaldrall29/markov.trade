//! Sidecar / `RegimeSource`. Gate B implementation is a stub that returns
//! `Chop` every tick (docs/11 §1, GATE-B.md §3.2). It sits behind a trait so
//! nothing downstream knows it is a stub. A model may only ever write into
//! `Features`; it never reaches the guard or a key.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)] // the stub only ever returns Chop; the other two are the contract P06 fills
pub enum Regime {
    Chop,
    Trend,
    Halt,
}

impl Regime {
    pub const fn name(self) -> &'static str {
        match self {
            Regime::Chop => "chop",
            Regime::Trend => "trend",
            Regime::Halt => "halt",
        }
    }
}

/// What a sidecar may contribute. A model may only ever write here: it never
/// reaches the guard, a key, or the mark.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RegimeFeatures {
    pub regime: Regime,
    /// STUB in Gate B: a constant until a real venue exposes funding. Never a
    /// measured number; never shown on a page as if it were.
    pub funding_favourable_stub: bool,
}

/// Everything the core is given about the world this tick: the sidecar's view,
/// plus the mark, which the sidecar never touches.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Features {
    pub regime: Regime,
    /// STUB in Gate B. See `RegimeFeatures`.
    pub funding_favourable_stub: bool,
    /// `None` when the mark could not be read or bound this tick.
    pub mark_e6: Option<u64>,
    pub mark_age_secs: Option<i64>,
    /// Reported on the tape; the freshness *decision* is in seconds (ADR-003).
    pub mark_age_slots: Option<u64>,
}

impl Features {
    /// Compose what the sidecar said with the mark the runtime read.
    pub fn new(
        r: RegimeFeatures,
        mark_e6: Option<u64>,
        mark_age_secs: Option<i64>,
        mark_age_slots: Option<u64>,
    ) -> Features {
        Features {
            regime: r.regime,
            funding_favourable_stub: r.funding_favourable_stub,
            mark_e6,
            mark_age_secs,
            mark_age_slots,
        }
    }
}

pub trait RegimeSource {
    fn features(&self) -> RegimeFeatures;
}

pub struct StubSidecar;

impl RegimeSource for StubSidecar {
    fn features(&self) -> RegimeFeatures {
        RegimeFeatures {
            regime: Regime::Chop,
            funding_favourable_stub: false,
        }
    }
}
