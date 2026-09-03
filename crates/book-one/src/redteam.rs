//! The red team: the only component allowed to submit an intent the local
//! guard vetoed, and it is still the **program** that refuses it.
//!
//! Its purpose is a proof surface. A book that never gets told no on chain has
//! not demonstrated that anything would stop it; these probes exist so the
//! refusal path is exercised on a schedule, in public, with `forced = true` on
//! every receipt so nobody can later present them as organic trades.
//!
//! `docs/11-AGENT-SPEC.md` §6 sets the schedule. Everything here is pure: it
//! decides *what* to send and *when*, and the submitter sends it.
//!
//! Disabled entirely when `VENUE=shadow` — a shadow runner submits nothing, so
//! a red team there would be theatre.

use markov_guard::{ActionKind, BlockReason, Intent, MandateState, PolicyView, Side};

/// One probe, its cadence, and the refusal it must provoke.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Probe {
    /// Notional one unit over the per-trade cap.
    OverTxCap,
    /// A limit further from the mark than the policy tolerates.
    SlippageExceeded,
    /// Spend above the per-call budget.
    OverSpendCap,
    /// A mark account that is not this mandate's, so the program cannot bind
    /// it and refuses rather than pricing from whatever it was handed.
    ///
    /// §6 words this as "replayed old slot". We cannot make the real feed
    /// stale on demand, and waiting for it to go stale would make the proof
    /// surface depend on an outage. Supplying the wrong mark account tests the
    /// same gate — the program has no bound mark — and is a probe worth having
    /// in its own right: it is what a compromised operator would try.
    StaleOracle,
    /// After a revoke: any otherwise-valid action must still be refused.
    /// Triggered by the mandate's state, not by the clock.
    Revoked,
}

impl Probe {
    /// The four scheduled probes, in the order they are checked. `Revoked` is
    /// not here: it is triggered by state, not by time.
    pub const SCHEDULED: [Probe; 4] = [
        Probe::OverTxCap,
        Probe::SlippageExceeded,
        Probe::OverSpendCap,
        Probe::StaleOracle,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            Probe::OverTxCap => "over_tx_cap",
            Probe::SlippageExceeded => "slippage_exceeded",
            Probe::OverSpendCap => "over_spend_cap",
            Probe::StaleOracle => "stale_oracle",
            Probe::Revoked => "revoked",
        }
    }

    /// Cadence in seconds, from `docs/11` §6.
    pub const fn interval_secs(self) -> i64 {
        match self {
            Probe::OverTxCap => 6 * 3_600,
            Probe::SlippageExceeded | Probe::OverSpendCap => 12 * 3_600,
            Probe::StaleOracle => 24 * 3_600,
            // Not scheduled; fires on the state transition.
            Probe::Revoked => i64::MAX,
        }
    }

    /// The refusal this probe must produce. If the chain says anything else,
    /// the proof surface is broken and the run is not evidence.
    pub const fn expected(self) -> BlockReason {
        match self {
            Probe::OverTxCap => BlockReason::OverTxCap,
            Probe::SlippageExceeded => BlockReason::SlippageExceeded,
            Probe::OverSpendCap => BlockReason::OverSpendCap,
            Probe::StaleOracle => BlockReason::StaleOracle,
            Probe::Revoked => BlockReason::Revoked,
        }
    }

    /// True when this probe needs a mark account other than the mandate's.
    pub const fn needs_wrong_mark(self) -> bool {
        matches!(self, Probe::StaleOracle)
    }

    /// The intent to force. Each one breaks exactly one rule, so the refusal
    /// names the gate we meant to test and not an earlier one.
    ///
    /// `+ 1` rather than a wild number: a probe that is over the cap by orders
    /// of magnitude proves the cap rejects absurdities. Over by one proves it
    /// rejects the boundary, which is where a real bug would live.
    pub fn intent(self, policy: &PolicyView, mark_e6: u64) -> Intent {
        let inside = Intent {
            action: ActionKind::Open,
            side: Side::Long,
            // A tenth of the cap: small enough that nothing else trips.
            notional: policy.per_tx_cap / 10,
            limit_price_e6: mark_e6,
            spend: 0,
        };
        match self {
            Probe::OverTxCap => Intent {
                notional: policy.per_tx_cap.saturating_add(1),
                ..inside
            },
            Probe::SlippageExceeded => Intent {
                // One basis point past the bound, in the direction that costs
                // us: a limit above the mark on a buy.
                limit_price_e6: mark_e6.saturating_add(
                    mark_e6.saturating_mul(u64::from(policy.max_slippage_bps) + 1) / 10_000,
                ),
                ..inside
            },
            Probe::OverSpendCap => Intent {
                spend: policy.spend_cap.saturating_add(1),
                ..inside
            },
            // Both of these send a perfectly ordinary intent; what makes them
            // probes is the account supplied (StaleOracle) or the mandate's
            // state (Revoked).
            Probe::StaleOracle | Probe::Revoked => inside,
        }
    }
}

/// When each scheduled probe last ran, as unix seconds. `None` means never.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LastRun {
    pub over_tx_cap: Option<i64>,
    pub slippage: Option<i64>,
    pub over_spend: Option<i64>,
    pub stale_oracle: Option<i64>,
    /// The mandate state at the last `Revoked` probe, so one revoke produces
    /// one probe rather than one per tick forever.
    pub probed_revoked: bool,
}

impl LastRun {
    pub fn get(&self, p: Probe) -> Option<i64> {
        match p {
            Probe::OverTxCap => self.over_tx_cap,
            Probe::SlippageExceeded => self.slippage,
            Probe::OverSpendCap => self.over_spend,
            Probe::StaleOracle => self.stale_oracle,
            Probe::Revoked => None,
        }
    }

    pub fn record(&mut self, p: Probe, now: i64) {
        match p {
            Probe::OverTxCap => self.over_tx_cap = Some(now),
            Probe::SlippageExceeded => self.slippage = Some(now),
            Probe::OverSpendCap => self.over_spend = Some(now),
            Probe::StaleOracle => self.stale_oracle = Some(now),
            Probe::Revoked => self.probed_revoked = true,
        }
    }
}

/// Which probe, if any, is due now. At most one per tick: the red team is a
/// proof surface, not a load test, and two forced refusals in one tick would
/// make the tape harder to read rather than more convincing.
///
/// A probe that has never run is due immediately, so a fresh deployment starts
/// producing evidence on its first tick rather than six hours later.
pub fn due(now: i64, last: &LastRun, state: MandateState) -> Option<Probe> {
    // A revoked mandate is the one case that jumps the queue: it is the
    // strongest refusal we can show and the window for it closes when the
    // owner un-revokes.
    if state == MandateState::Revoked {
        return if last.probed_revoked {
            None
        } else {
            Some(Probe::Revoked)
        };
    }
    Probe::SCHEDULED.into_iter().find(|p| match last.get(*p) {
        None => true,
        Some(then) => now.saturating_sub(then) >= p.interval_secs(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{GATE_B_MAX_SLIPPAGE_BPS, GATE_B_PER_TX_CAP};

    const MARK: u64 = 100_000_000;

    fn policy() -> PolicyView {
        PolicyView {
            max_mark_age_secs: 150,
            allowed_actions: PolicyView::actions_mask(&[ActionKind::Open]),
            per_tx_cap: GATE_B_PER_TX_CAP,
            daily_cap: GATE_B_PER_TX_CAP * 4,
            spend_cap: GATE_B_PER_TX_CAP,
            spend_daily_cap: GATE_B_PER_TX_CAP * 4,
            max_slippage_bps: GATE_B_MAX_SLIPPAGE_BPS,
            delta_band: 20_000_000,
            max_gross: 100_000_000,
            daily_loss_bps: 500,
        }
    }

    /// Every scheduled probe is due on a cold start, so a fresh deployment
    /// produces evidence immediately.
    #[test]
    fn every_probe_is_due_on_a_cold_start() {
        let mut last = LastRun::default();
        let mut seen = Vec::new();
        for _ in 0..Probe::SCHEDULED.len() {
            let p = due(1_000, &last, MandateState::Active).expect("a probe is due");
            last.record(p, 1_000);
            seen.push(p);
        }
        assert_eq!(seen, Probe::SCHEDULED.to_vec());
        assert_eq!(
            due(1_000, &last, MandateState::Active),
            None,
            "one per tick"
        );
    }

    /// Each probe comes back on its own cadence and not before.
    #[test]
    fn each_probe_waits_its_interval() {
        for p in Probe::SCHEDULED {
            let i = p.interval_secs();
            let mut last = LastRun::default();
            // Every *other* probe ran a moment ago, so only `p`'s clock can
            // make anything due. Without this the 6-hour probe is due in every
            // case and the test would pass without testing `p` at all.
            for q in Probe::SCHEDULED {
                last.record(q, i - 1);
            }
            last.record(p, 0);

            assert_eq!(due(i - 1, &last, MandateState::Active), None, "{p:?} early");
            assert_eq!(
                due(i, &last, MandateState::Active),
                Some(p),
                "{p:?} not due at its own interval"
            );
        }
    }

    /// A revoke jumps the queue, and produces exactly one probe.
    #[test]
    fn a_revoke_is_probed_once() {
        let mut last = LastRun::default();
        assert_eq!(
            due(1_000, &last, MandateState::Revoked),
            Some(Probe::Revoked),
            "a revoked mandate is the strongest refusal available"
        );
        last.record(Probe::Revoked, 1_000);
        assert_eq!(
            due(9_999_999, &last, MandateState::Revoked),
            None,
            "one revoke, one probe — not one per tick forever"
        );
    }

    /// Each intent breaks exactly one rule, so the refusal names the gate the
    /// probe meant to test rather than an earlier one.
    #[test]
    fn each_probe_breaks_exactly_one_rule() {
        let p = policy();
        let over = Probe::OverTxCap.intent(&p, MARK);
        assert_eq!(
            over.notional,
            p.per_tx_cap + 1,
            "over by one, at the boundary"
        );
        assert_eq!(over.spend, 0);
        assert_eq!(over.limit_price_e6, MARK, "the limit is not also broken");

        let slip = Probe::SlippageExceeded.intent(&p, MARK);
        let bps = slip.limit_price_e6.abs_diff(MARK) * 10_000 / MARK;
        assert!(
            bps > u64::from(p.max_slippage_bps),
            "{bps} bps is not past the {} bps bound",
            p.max_slippage_bps
        );
        assert!(slip.notional <= p.per_tx_cap, "the cap is not also broken");

        let spend = Probe::OverSpendCap.intent(&p, MARK);
        assert_eq!(spend.spend, p.spend_cap + 1);
        assert!(spend.notional <= p.per_tx_cap);

        // These two are ordinary intents; the probe is in the accounts or the
        // mandate's state.
        for probe in [Probe::StaleOracle, Probe::Revoked] {
            let i = probe.intent(&p, MARK);
            assert!(i.notional <= p.per_tx_cap, "{probe:?}");
            assert!(i.spend <= p.spend_cap, "{probe:?}");
            assert_eq!(i.limit_price_e6, MARK, "{probe:?}");
        }
        assert!(Probe::StaleOracle.needs_wrong_mark());
        assert!(!Probe::OverTxCap.needs_wrong_mark());
    }

    /// The expected refusals are distinct, so a probe cannot be satisfied by
    /// another probe's receipt.
    #[test]
    fn expected_refusals_are_distinct() {
        let mut reasons: Vec<u8> = Probe::SCHEDULED
            .iter()
            .chain([Probe::Revoked].iter())
            .map(|p| p.expected() as u8)
            .collect();
        let before = reasons.len();
        reasons.sort_unstable();
        reasons.dedup();
        assert_eq!(reasons.len(), before, "two probes expect the same refusal");
    }
}
