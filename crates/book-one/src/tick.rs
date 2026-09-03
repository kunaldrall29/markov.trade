//! One tick: read the mark, build features, propose, guard, record.

use chrono::{DateTime, Utc};
use markov_guard::{evaluate, ActionKind, GuardState, MandateState, PolicyView, Verdict};

use crate::core::BookState;
use markov_marks::{MarkError, MarkSource};
use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::sidecar::{Features, RegimeSource, StubSidecar};

/// One row per tick, including the boring ones. This is the paper file's
/// only input and it is never edited after being written.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TickRecord {
    pub tick_id: String,
    pub ts_unix: i64,
    pub day: chrono::NaiveDate,
    pub slot: u64,
    pub mark_source: String,
    pub mark_price: Option<f64>,
    pub mark_publish_time: Option<i64>,
    pub mark_age_s: Option<i64>,
    pub mark_age_slots: Option<u64>,
    pub regime: String,
    pub intent: String,
    pub verdict: String,
    pub reason: Option<String>,
    /// `on_chain` or `off_chain_v0` for a veto, absent otherwise. Defaulted so
    /// tick logs written before this field existed still parse.
    #[serde(default)]
    pub reason_enforcement: Option<String>,
    pub latency_ms: u64,
    pub error: Option<String>,
    /// Devnet mode only. All defaulted so tick logs written by a shadow runner
    /// — or by a build before these existed — still parse.
    #[serde(default)]
    pub signature: Option<String>,
    /// True when the red team forced this intent past the local guard. The
    /// program still refused it; the flag exists so nobody can present a
    /// forced refusal as organic.
    #[serde(default)]
    pub forced: bool,
    /// Why nothing was sent, when the verdict alone does not explain it:
    /// `shadow`, `halted`, `hourly_action_budget_exhausted`.
    #[serde(default)]
    pub withheld: Option<String>,
    /// The reason the *program* gave, which is the one that counts. A
    /// disagreement with `reason` is a guard divergence.
    #[serde(default)]
    pub onchain_reason: Option<String>,
    /// Which red-team probe this tick carried, if any.
    #[serde(default)]
    pub redteam_probe: Option<String>,
}

pub async fn run_tick<S: MarkSource>(
    cfg: &Config,
    source: &mut S,
    book: &mut BookState,
    n: u64,
) -> (TickRecord, Verdict, PolicyView) {
    let t0 = std::time::Instant::now();
    let mark = source.get().await;
    // `now` is taken after the fetch so a mark published during the RPC round
    // trip cannot show a negative age.
    let now: DateTime<Utc> = Utc::now();
    let m = mark.as_ref().ok();
    let feats = Features::new(
        StubSidecar.features(),
        m.and_then(|m| m.price_e6()),
        m.map(|m| m.age_secs(now.timestamp())),
        m.map(|m| m.age_slots()),
    );
    let policy = PolicyView {
        max_mark_age_secs: cfg.max_mark_age_secs,
        allowed_actions: PolicyView::actions_mask(&[
            ActionKind::Open,
            ActionKind::Increase,
            ActionKind::Reduce,
            ActionKind::Close,
            ActionKind::Flatten,
        ]),
        per_tx_cap: cfg.per_tx_cap,
        daily_cap: cfg.daily_cap,
        spend_cap: cfg.spend_cap,
        spend_daily_cap: cfg.spend_daily_cap,
        max_slippage_bps: cfg.max_slippage_bps,
        delta_band: cfg.delta_band,
        max_gross: cfg.max_gross,
        daily_loss_bps: cfg.daily_loss_bps,
    };
    let intent = crate::core::propose(book, &feats, &policy);
    // Shadow mode holds no mandate and no position, so the exposure, the
    // counters and the equity are all genuinely zero — not unknown, and not
    // defaulted to make a rule pass. The guard's daily-loss rule does not fire
    // on a zero session-start equity, which is the correct behaviour for a book
    // that has never had any: the caps above are what bound it.
    let state = GuardState {
        now_unix: now.timestamp(),
        slot: m.map(|m| m.observed_slot).unwrap_or(0),
        state: MandateState::Active,
        mark_e6: feats.mark_e6,
        mark_publish_time: m.map(|m| m.publish_time),
        net_delta: book.net_delta,
        gross: book.gross,
        vault_balance: 0,
        day_notional_used: 0,
        day_spend_used: 0,
        session_start_equity: 0,
        equity: 0,
    };
    let verdict = evaluate(&intent, &state, &policy);
    let (verdict_name, reason, enforcement) = match verdict {
        Verdict::Allow(_) => ("allow", None, None),
        // The tape carries where the veto is enforced, so no surface reading
        // it has to guess whether "DeltaBandExceeded" is a chain guarantee.
        Verdict::Veto(r) => (
            "veto",
            Some(r.name().to_string()),
            Some(
                if r.is_off_chain() {
                    "off_chain_v0"
                } else {
                    "on_chain"
                }
                .to_string(),
            ),
        ),
        Verdict::Skip => ("skip", None, None),
    };
    let error = match &mark {
        Ok(_) => None,
        Err(e @ MarkError::Rpc(_)) => Some(format!("rpc: {e}")),
        Err(e) => Some(e.to_string()),
    };
    // Shadow mode submits nothing, so the book's exposure never changes. What
    // *does* carry across ticks is the previous tick's proposal, which is what
    // hysteresis compares against.
    book.last_action = Some(intent.action);
    book.last_net_delta = book.net_delta;

    let record = TickRecord {
        tick_id: format!("{}-{n:06}", now.format("%Y%m%dT%H%M%SZ")),
        ts_unix: now.timestamp(),
        day: now.date_naive(),
        slot: m.map(|m| m.observed_slot).unwrap_or(0),
        mark_source: source.name().to_string(),
        mark_price: m.map(|m| m.price_f64()),
        mark_publish_time: m.map(|m| m.publish_time),
        mark_age_s: m.map(|m| m.age_secs(now.timestamp())),
        mark_age_slots: m.map(|m| m.age_slots()),
        regime: feats.regime.name().to_string(),
        intent: action_name(intent.action).to_string(),
        verdict: verdict_name.to_string(),
        reason,
        reason_enforcement: enforcement,
        signature: None,
        forced: false,
        withheld: None,
        onchain_reason: None,
        redteam_probe: None,
        latency_ms: t0.elapsed().as_millis() as u64,
        error,
    };
    // The verdict and the policy go back with the record because the chain
    // half of the tick needs both, and re-deriving them there would be a
    // second source of truth for what this tick decided.
    (record, verdict, policy)
}

pub const fn action_name(a: markov_types::ActionKind) -> &'static str {
    use markov_types::ActionKind::*;
    match a {
        Skip => "skip",
        Open => "open",
        Increase => "increase",
        Reduce => "reduce",
        Close => "close",
        Flatten => "flatten",
    }
}
