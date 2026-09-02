//! The pure risk guard: `(Intent, GuardState, PolicyView) -> Verdict`.
//!
//! Plain words: the guard is a function. It is handed everything it needs —
//! the intent, the current time, the mark and its age, the policy limits —
//! and returns Allow, Veto(reason) or Skip. It never reads a clock, opens a
//! socket, or logs. If any input is missing it vetoes; there is no path that
//! allows on missing data. P05 fills in the full ladder; P08 needs only the
//! freshness rule and the Skip default, so that is what exists here.
#![forbid(unsafe_code)]
#![warn(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![no_std]

pub use markov_types::{ActionKind, BlockReason};

/// What the core proposed for this tick.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Intent {
    pub action: ActionKind,
    /// Notional in mint base units (0 for Skip).
    pub notional: u64,
}

impl Intent {
    pub const SKIP: Intent = Intent {
        action: ActionKind::Skip,
        notional: 0,
    };
}

/// Everything the guard is allowed to know, passed in — never read.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GuardState {
    pub now_unix: i64,
    /// `None` means the mark could not be read this tick.
    pub mark_publish_time: Option<i64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PolicyView {
    pub max_mark_age_secs: i64,
    pub per_tx_cap: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    Allow(Intent),
    Veto(BlockReason),
    Skip,
}

/// Mirrors the on-chain ladder order for the gates that exist off-chain.
/// P08 subset: freshness, then per-tx cap. A `Skip` intent is never vetoed on
/// size, but a stale mark is still reported as `StaleOracle` so the paper log
/// shows feed gaps honestly.
pub fn evaluate(intent: &Intent, state: &GuardState, policy: &PolicyView) -> Verdict {
    let Some(published) = state.mark_publish_time else {
        return Verdict::Veto(BlockReason::StaleOracle);
    };
    let age = state.now_unix.checked_sub(published);
    match age {
        Some(a) if a >= 0 && a <= policy.max_mark_age_secs => {}
        _ => return Verdict::Veto(BlockReason::StaleOracle),
    }
    if intent.action == ActionKind::Skip {
        return Verdict::Skip;
    }
    if intent.notional > policy.per_tx_cap {
        return Verdict::Veto(BlockReason::OverTxCap);
    }
    Verdict::Allow(*intent)
}

#[cfg(test)]
mod tests {
    use super::*;

    const POLICY: PolicyView = PolicyView {
        max_mark_age_secs: 150,
        per_tx_cap: 50,
    };

    #[test]
    fn fail_closed_on_missing_input() {
        let s = GuardState {
            now_unix: 1_000,
            mark_publish_time: None,
        };
        assert_eq!(
            evaluate(&Intent::SKIP, &s, &POLICY),
            Verdict::Veto(BlockReason::StaleOracle)
        );
    }

    #[test]
    fn stale_mark_is_stale_oracle_even_for_skip() {
        let s = GuardState {
            now_unix: 1_000,
            mark_publish_time: Some(1_000 - 151),
        };
        assert_eq!(
            evaluate(&Intent::SKIP, &s, &POLICY),
            Verdict::Veto(BlockReason::StaleOracle)
        );
    }

    #[test]
    fn skip_is_the_default() {
        let s = GuardState {
            now_unix: 1_000,
            mark_publish_time: Some(990),
        };
        assert_eq!(evaluate(&Intent::SKIP, &s, &POLICY), Verdict::Skip);
    }

    #[test]
    fn over_tx_cap_is_vetoed() {
        let s = GuardState {
            now_unix: 1_000,
            mark_publish_time: Some(990),
        };
        let i = Intent {
            action: ActionKind::Open,
            notional: 51,
        };
        assert_eq!(
            evaluate(&i, &s, &POLICY),
            Verdict::Veto(BlockReason::OverTxCap)
        );
    }

    #[test]
    fn future_mark_is_not_trusted() {
        let s = GuardState {
            now_unix: 1_000,
            mark_publish_time: Some(1_001),
        };
        assert_eq!(
            evaluate(&Intent::SKIP, &s, &POLICY),
            Verdict::Veto(BlockReason::StaleOracle)
        );
    }
}
