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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Features {
    pub regime: Regime,
    /// STUB in Gate B: a constant until a real venue exposes funding. Never a
    /// measured number; never shown on a page as if it were.
    pub funding_favourable_stub: bool,
}

pub trait RegimeSource {
    fn features(&self) -> Features;
}

pub struct StubSidecar;

impl RegimeSource for StubSidecar {
    fn features(&self) -> Features {
        Features {
            regime: Regime::Chop,
            funding_favourable_stub: false,
        }
    }
}
