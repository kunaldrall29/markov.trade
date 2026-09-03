//! Kill switches and counters.
//!
//! Two different stops, because they answer two different questions.
//!
//! **HALT** — an environment variable or a file on the volume — stops
//! *submissions* and leaves the ticks running. That matters: an agent that
//! goes silent when someone halts it looks identical to an agent that crashed,
//! and the tape is supposed to be the thing you can check. Halted ticks are
//! recorded, with the reason, so the log keeps explaining itself.
//!
//! **`MAX_ACTIONS_PER_HOUR`** is the budget, and blowing it latches the agent
//! off for the rest of the process's life. A book that suddenly wants to act
//! twenty times an hour is either broken or being driven, and neither is a
//! state to trade through. It latches rather than throttles because a
//! throttled runaway is still a runaway, just a slower one that nobody
//! notices.

use std::collections::VecDeque;
use std::path::Path;
use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};

/// Why a submission was withheld. `&'static str` so it can go straight onto a
/// tick row without an allocation and without drifting into free text.
pub const WITHHELD_HALT: &str = "halted";
pub const WITHHELD_BUDGET: &str = "hourly_action_budget_exhausted";
pub const WITHHELD_SHADOW: &str = "shadow";

#[derive(Debug)]
pub struct Governor {
    max_actions_per_hour: u32,
    /// Unix seconds of each action in the last hour, oldest first.
    recent: VecDeque<i64>,
    /// Once true, never false again until the process restarts.
    latched: bool,
}

impl Governor {
    pub fn new(max_actions_per_hour: u32) -> Governor {
        Governor {
            max_actions_per_hour,
            recent: VecDeque::new(),
            latched: false,
        }
    }

    pub fn latched(&self) -> bool {
        self.latched
    }

    pub fn actions_in_last_hour(&self) -> usize {
        self.recent.len()
    }

    /// May the agent submit right now? Prunes the window first, so an hour of
    /// quiet restores the budget — but only if the latch never tripped.
    pub fn may_submit(&mut self, now: i64) -> Result<(), &'static str> {
        self.prune(now);
        if self.latched {
            return Err(WITHHELD_BUDGET);
        }
        if self.recent.len() >= self.max_actions_per_hour as usize {
            self.latched = true;
            return Err(WITHHELD_BUDGET);
        }
        Ok(())
    }

    pub fn record(&mut self, now: i64) {
        self.prune(now);
        self.recent.push_back(now);
    }

    fn prune(&mut self, now: i64) {
        while let Some(&oldest) = self.recent.front() {
            if now.saturating_sub(oldest) >= 3_600 {
                self.recent.pop_front();
            } else {
                break;
            }
        }
    }
}

/// Is the agent halted? Either source is enough; neither is cleared by the
/// other. Checked every tick rather than at boot, so a halt takes effect
/// within one tick without a redeploy.
pub fn halted(env_var: &str, halt_file: &Path) -> bool {
    let env_set = std::env::var(env_var)
        .map(|v| {
            let v = v.trim().to_ascii_lowercase();
            !v.is_empty() && v != "0" && v != "false"
        })
        .unwrap_or(false);
    env_set || halt_file.exists()
}

/// Counters, in the shape `/metrics` serves them.
///
/// `guard_divergence_total` is the one that matters: the guard mirrors the
/// program's ladder, so if the program refuses something the guard allowed,
/// one of the two is wrong about the rules and every claim the tape makes is
/// suspect until it is explained. It is expected to be zero, and a non-zero
/// value is a page, not a metric to watch trend upward.
#[derive(Debug, Default)]
pub struct Metrics {
    pub ticks_total: AtomicU64,
    pub skips_total: AtomicU64,
    pub allows_total: AtomicU64,
    pub vetoes_total: AtomicU64,
    pub submissions_total: AtomicU64,
    pub submissions_failed_total: AtomicU64,
    pub refusals_total: AtomicU64,
    pub redteam_probes_total: AtomicU64,
    pub redteam_refusals_total: AtomicU64,
    pub guard_divergence_total: AtomicU64,
    pub last_tick_unix: AtomicI64,
    pub last_redteam_refusal_unix: AtomicI64,
}

impl Metrics {
    pub fn incr(counter: &AtomicU64) {
        counter.fetch_add(1, Ordering::Relaxed);
    }

    pub fn render(&self) -> String {
        let g = |c: &AtomicU64| c.load(Ordering::Relaxed);
        let gi = |c: &AtomicI64| c.load(Ordering::Relaxed);
        let mut out = String::new();
        for (name, help, value) in [
            (
                "book_one_ticks_total",
                "ticks attempted",
                g(&self.ticks_total),
            ),
            (
                "book_one_skips_total",
                "ticks whose verdict was skip",
                g(&self.skips_total),
            ),
            (
                "book_one_allows_total",
                "ticks the guard allowed",
                g(&self.allows_total),
            ),
            (
                "book_one_vetoes_total",
                "ticks the guard vetoed",
                g(&self.vetoes_total),
            ),
            (
                "book_one_submissions_total",
                "transactions sent",
                g(&self.submissions_total),
            ),
            (
                "book_one_submissions_failed_total",
                "transactions that could not be confirmed",
                g(&self.submissions_failed_total),
            ),
            (
                "book_one_refusals_total",
                "refusals recorded on chain",
                g(&self.refusals_total),
            ),
            (
                "book_one_redteam_probes_total",
                "red-team probes sent",
                g(&self.redteam_probes_total),
            ),
            (
                "book_one_redteam_refusals_total",
                "red-team probes the program refused; zero in 24h is an alert",
                g(&self.redteam_refusals_total),
            ),
            (
                "book_one_guard_divergence_total",
                "the program refused something the guard allowed; must be zero",
                g(&self.guard_divergence_total),
            ),
        ] {
            out.push_str(&format!(
                "# HELP {name} {help}\n# TYPE {name} counter\n{name} {value}\n"
            ));
        }
        for (name, help, value) in [
            (
                "book_one_last_tick_unix",
                "unix time of the last tick",
                gi(&self.last_tick_unix),
            ),
            (
                "book_one_last_redteam_refusal_unix",
                "unix time of the last red-team refusal",
                gi(&self.last_redteam_refusal_unix),
            ),
        ] {
            out.push_str(&format!(
                "# HELP {name} {help}\n# TYPE {name} gauge\n{name} {value}\n"
            ));
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_budget_latches_rather_than_throttling() {
        let mut g = Governor::new(3);
        for i in 0..3 {
            assert!(
                g.may_submit(1_000 + i).is_ok(),
                "action {i} inside the budget"
            );
            g.record(1_000 + i);
        }
        assert_eq!(
            g.may_submit(1_003),
            Err(WITHHELD_BUDGET),
            "the fourth is over"
        );
        assert!(g.latched());
        // An hour later the window is empty, and it stays latched anyway: a
        // runaway that waits an hour is still a runaway.
        assert_eq!(g.may_submit(1_003 + 3_600), Err(WITHHELD_BUDGET));
        assert_eq!(g.actions_in_last_hour(), 0, "the window itself did prune");
    }

    #[test]
    fn the_window_is_an_hour() {
        let mut g = Governor::new(2);
        g.record(0);
        g.record(1);
        assert_eq!(g.actions_in_last_hour(), 2);
        // Exactly an hour later, both have aged out.
        assert!(g.may_submit(3_601).is_ok());
        assert_eq!(g.actions_in_last_hour(), 0);
    }

    #[test]
    fn halt_reads_either_source() {
        let dir = tempfile::tempdir().expect("tempdir");
        let file = dir.path().join("HALT");
        let var = "BOOK_ONE_TEST_HALT";

        // SAFETY-adjacent: this test owns a uniquely named variable, and the
        // test binary is single-threaded for these reads.
        unsafe { std::env::remove_var(var) };
        assert!(!halted(var, &file), "neither source set");

        std::fs::write(&file, "").expect("write");
        assert!(halted(var, &file), "the file alone is enough");
        std::fs::remove_file(&file).expect("rm");

        for value in ["1", "true", "yes", "please stop"] {
            unsafe { std::env::set_var(var, value) };
            assert!(halted(var, &file), "{value} should halt");
        }
        for value in ["", "0", "false", "  "] {
            unsafe { std::env::set_var(var, value) };
            assert!(!halted(var, &file), "{value:?} should not halt");
        }
        unsafe { std::env::remove_var(var) };
    }

    #[test]
    fn metrics_name_the_divergence_counter_and_start_at_zero() {
        let m = Metrics::default();
        let text = m.render();
        assert!(text.contains("book_one_guard_divergence_total 0"));
        assert!(text.contains("must be zero"));
        assert!(text.contains("zero in 24h is an alert"));
        Metrics::incr(&m.guard_divergence_total);
        assert!(m.render().contains("book_one_guard_divergence_total 1"));
    }
}
