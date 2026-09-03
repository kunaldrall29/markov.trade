//! Devnet mode: the part of the tick that can put a transaction on chain.
//!
//! Everything above this — the mark, the core, the guard — is the same in
//! shadow and devnet. This module is the difference, and it is deliberately
//! the smallest piece: it decides whether to send, sends, and writes down what
//! the chain said.
//!
//! Three things it will not do.
//!
//! It will not send more than one transaction per tick. If the book has a real
//! action this tick, the red team waits for the next one — a tape with two
//! transactions in a tick is harder to read, and the probes have hours of
//! slack in their schedules.
//!
//! It will not retry a refusal. A refusal is the answer, and sending it again
//! would put a second receipt on chain for one decision.
//!
//! It will not widen anything. If the program says the cap refuses this, the
//! cap refuses this.

use std::path::PathBuf;
use std::sync::Arc;

use markov_chain::Chain;
use markov_guard::{PolicyView, Verdict};
use solana_pubkey::Pubkey;

use crate::chainstate::MandateSnapshot;
use crate::redteam::{self, LastRun, Probe};
use crate::runtime::{halted, Governor, Metrics, WITHHELD_BUDGET, WITHHELD_HALT};
use crate::submitter::{program_intent, refusal_reason, Submitted, Submitter};
use crate::tick::TickRecord;

pub struct Agent {
    pub chain: Chain,
    pub submitter: Submitter,
    pub governor: Governor,
    pub metrics: Arc<Metrics>,
    pub redteam_last: LastRun,
    pub redteam_enabled: bool,
    pub halt_env: String,
    pub halt_file: PathBuf,
    /// A valid Pyth price update account that is **not** this mandate's, for
    /// the `StaleOracle` probe. The program must refuse to price from it.
    pub wrong_mark_account: Option<Pubkey>,
    /// Bumped per submission so two identical decisions in one slot bucket
    /// still get distinct ids when they are genuinely distinct attempts.
    pub nonce: u64,
    pub attempts: u32,
    /// The Pyth feed the venue's mark must carry, for the relay.
    pub feed_id: [u8; 32],
}

impl Agent {
    /// Run the chain-facing half of a tick, filling in the record.
    pub fn act(
        &mut self,
        now_unix: i64,
        slot: u64,
        snapshot: &MandateSnapshot,
        policy: &PolicyView,
        verdict: &Verdict,
        record: &mut TickRecord,
    ) {
        if halted(&self.halt_env, &self.halt_file) {
            // Ticks continue: a halted agent that went silent would look
            // exactly like a crashed one, and the log is the thing anyone can
            // check.
            record.withheld = Some(WITHHELD_HALT.to_string());
            return;
        }

        if let Verdict::Allow(intent) = verdict {
            if let Err(why) = self.governor.may_submit(now_unix) {
                record.withheld = Some(why.to_string());
                tracing::error!(why, "submission withheld; the agent is latched off");
                return;
            }
            self.nonce = self.nonce.wrapping_add(1);
            let program_intent = program_intent(
                &self.submitter.wiring().mandate,
                slot,
                self.submitter.wiring().market_id,
                intent,
                policy.max_slippage_bps,
                false,
                self.nonce,
            );
            self.governor.record(now_unix);
            Metrics::incr(&self.metrics.submissions_total);
            // Refresh the venue's mark in the same transaction. The venue
            // enforces its own freshness and nothing else refreshes it, so
            // without this the venue refuses every action with `StaleMark` —
            // correctly, and uselessly. Measured on devnet before this
            // existed: signature 27V1KwrsmNuJuv7WqEpsULZo…, `VenueRejected`.
            let relay = self.submitter.mark_relay_instruction(self.feed_id);
            let sent = self.submitter.submit(
                &self.chain,
                &program_intent,
                None,
                Some(relay),
                self.attempts,
            );
            self.write_outcome(sent, record, None);
            return;
        }

        // No action this tick, so the red team may use it.
        if !self.redteam_enabled {
            return;
        }
        let Some(probe) = redteam::due(now_unix, &self.redteam_last, snapshot.state_at(now_unix))
        else {
            return;
        };
        if self.governor.may_submit(now_unix).is_err() {
            record.withheld = Some(WITHHELD_BUDGET.to_string());
            return;
        }
        let Some(mark_e6) = mark_for_probe(record) else {
            // A probe needs a price to build a limit around. Without one the
            // SlippageExceeded probe would be meaningless, so wait a tick.
            return;
        };
        let override_mark = if probe.needs_wrong_mark() {
            match self.wrong_mark_account {
                Some(k) => Some(k),
                None => {
                    tracing::warn!(
                        "the StaleOracle probe needs REDTEAM_WRONG_MARK_ACCOUNT; skipping it"
                    );
                    self.redteam_last.record(probe, now_unix);
                    return;
                }
            }
        } else {
            None
        };

        self.nonce = self.nonce.wrapping_add(1);
        let intent = probe.intent(policy, mark_e6);
        let program_intent = program_intent(
            &self.submitter.wiring().mandate,
            slot,
            self.submitter.wiring().market_id,
            &intent,
            policy.max_slippage_bps,
            true, // forced, and recorded as such on the receipt
            self.nonce,
        );
        self.governor.record(now_unix);
        self.redteam_last.record(probe, now_unix);
        record.forced = true;
        record.redteam_probe = Some(probe.name().to_string());
        Metrics::incr(&self.metrics.redteam_probes_total);
        Metrics::incr(&self.metrics.submissions_total);
        // A probe is refused by the mandate's own ladder before the CPI, so it
        // never reaches the venue and needs no mark relay.
        let sent = self.submitter.submit(
            &self.chain,
            &program_intent,
            override_mark,
            None,
            self.attempts,
        );
        self.write_outcome(sent, record, Some(probe));
    }

    /// Record what the chain said, and check the two things that would mean
    /// the tape cannot be trusted.
    fn write_outcome(&mut self, sent: Submitted, record: &mut TickRecord, probe: Option<Probe>) {
        match sent {
            Submitted::Landed(landed) => {
                record.signature = Some(landed.signature.to_string());
                let inner = self
                    .chain
                    .inner_instruction_data(&landed.signature)
                    .unwrap_or_else(|e| {
                        tracing::warn!(error = %e, "could not read the receipt");
                        Vec::new()
                    });
                let reason = refusal_reason(&inner);
                record.onchain_reason = reason.map(|r| r.name().to_string());
                if let Some(err) = landed.err {
                    record.error = Some(err);
                }
                match (reason, probe) {
                    (Some(got), Some(probe)) => {
                        Metrics::incr(&self.metrics.redteam_refusals_total);
                        Metrics::incr(&self.metrics.refusals_total);
                        self.metrics
                            .last_redteam_refusal_unix
                            .store(record.ts_unix, std::sync::atomic::Ordering::Relaxed);
                        if got != probe.expected() {
                            // The probe provoked *a* refusal, but not the one
                            // it exists to demonstrate. The proof surface is
                            // not doing its job.
                            tracing::error!(
                                probe = probe.name(),
                                expected = probe.expected().name(),
                                got = got.name(),
                                signature = %landed.signature,
                                "red-team probe produced the wrong refusal"
                            );
                        }
                    }
                    (Some(got), None) => {
                        Metrics::incr(&self.metrics.refusals_total);
                        if is_divergence(got) {
                            // The guard allowed this and the program refused
                            // it on a rung the guard mirrors. One of the two is
                            // wrong about the rules, and until that is
                            // explained every claim the tape makes is suspect.
                            Metrics::incr(&self.metrics.guard_divergence_total);
                            tracing::error!(
                                onchain_reason = got.name(),
                                signature = %landed.signature,
                                "GUARD DIVERGENCE: the program refused what the guard allowed"
                            );
                        } else {
                            tracing::info!(
                                onchain_reason = got.name(),
                                signature = %landed.signature,
                                "the program refused for a reason the guard cannot mirror"
                            );
                        }
                    }
                    (None, Some(probe)) => {
                        tracing::error!(
                            probe = probe.name(),
                            signature = %landed.signature,
                            "red-team probe was NOT refused"
                        );
                    }
                    (None, None) => {}
                }
            }
            Submitted::Withheld(why) => record.withheld = Some(why.to_string()),
            Submitted::Failed(e) => {
                Metrics::incr(&self.metrics.submissions_failed_total);
                record.error = Some(format!("submit: {e}"));
            }
        }
    }
}

/// Is an on-chain refusal of a guard-allowed intent a *divergence*?
///
/// Only for rungs the guard actually mirrors. Two are not:
///
/// - `VenueRejected` is the venue's own state — its mark age, its pause flag,
///   its position cap. The guard has no view of any of that and never claimed
///   one, so counting it would page for something nobody can fix by changing
///   the guard.
/// - `DuplicateIntent` means this exact intent already landed, which is
///   success-already-happened (`docs/11` §5), not a disagreement.
///
/// Everything else is a real disagreement about the rules and must page.
fn is_divergence(reason: markov_types::BlockReason) -> bool {
    !matches!(
        reason,
        markov_types::BlockReason::VenueRejected | markov_types::BlockReason::DuplicateIntent
    )
}

/// The mark this tick saw, as an integer, for building a probe's limit.
fn mark_for_probe(record: &TickRecord) -> Option<u64> {
    let price = record.mark_price?;
    if price <= 0.0 {
        return None;
    }
    Some((price * 1_000_000.0) as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(price: Option<f64>) -> TickRecord {
        TickRecord {
            tick_id: "t".into(),
            ts_unix: 0,
            day: chrono::NaiveDate::from_ymd_opt(2026, 9, 3).expect("date"),
            slot: 1,
            mark_source: "test".into(),
            mark_price: price,
            mark_publish_time: None,
            mark_age_s: None,
            mark_age_slots: None,
            regime: "chop".into(),
            intent: "skip".into(),
            verdict: "skip".into(),
            reason: None,
            reason_enforcement: None,
            latency_ms: 0,
            error: None,
            signature: None,
            forced: false,
            withheld: None,
            onchain_reason: None,
            redteam_probe: None,
            net_delta_e6: None,
            gross_e6: None,
            marked_pnl_e6: None,
        }
    }

    /// The divergence counter must page for a rule disagreement and stay
    /// silent for the two reasons the guard structurally cannot mirror.
    /// Getting this wrong in either direction is bad: a false page teaches
    /// people to ignore it, and a missed one is the whole alarm.
    #[test]
    fn only_a_real_rule_disagreement_is_a_divergence() {
        use markov_types::BlockReason as R;
        for quiet in [R::VenueRejected, R::DuplicateIntent] {
            assert!(!is_divergence(quiet), "{quiet:?} must not page");
        }
        for loud in [
            R::OverTxCap,
            R::OverDailyCap,
            R::OverSpendCap,
            R::OverSpendDailyCap,
            R::SlippageExceeded,
            R::Paused,
            R::Revoked,
            R::Expired,
            R::StaleOracle,
            R::ActionNotAllowed,
            R::ProgramNotAllowed,
            R::TokenNotAllowed,
            R::Unauthorized,
            R::GlobalHalt,
            R::PostCheckFailed,
            R::ControlledAccountForwarded,
        ] {
            assert!(is_divergence(loud), "{loud:?} must page");
        }
    }

    /// A probe without a price would build a meaningless limit, so it waits.
    #[test]
    fn a_probe_needs_a_price() {
        assert_eq!(mark_for_probe(&record(Some(104.25))), Some(104_250_000));
        assert_eq!(mark_for_probe(&record(None)), None);
        assert_eq!(mark_for_probe(&record(Some(0.0))), None);
        assert_eq!(mark_for_probe(&record(Some(-1.0))), None);
    }
}
