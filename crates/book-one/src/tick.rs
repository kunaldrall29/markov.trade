//! One tick: read the mark, build features, propose, guard, record.

use chrono::{DateTime, Utc};
use markov_guard::{evaluate, GuardState, PolicyView, Verdict};
use markov_marks::{MarkError, MarkSource};
use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::sidecar::{RegimeSource, StubSidecar};

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
    pub latency_ms: u64,
    pub error: Option<String>,
}

pub async fn run_tick<S: MarkSource>(cfg: &Config, source: &mut S, n: u64) -> TickRecord {
    let t0 = std::time::Instant::now();
    let mark = source.get().await;
    // `now` is taken after the fetch so a mark published during the RPC round
    // trip cannot show a negative age.
    let now: DateTime<Utc> = Utc::now();
    let feats = StubSidecar.features();
    let intent = crate::core::propose(&feats);
    let state = GuardState {
        now_unix: now.timestamp(),
        mark_publish_time: mark.as_ref().ok().map(|m| m.publish_time),
    };
    let policy = PolicyView {
        max_mark_age_secs: cfg.max_mark_age_secs,
        per_tx_cap: cfg.per_tx_cap,
    };
    let verdict = evaluate(&intent, &state, &policy);
    let (verdict_name, reason) = match verdict {
        Verdict::Allow(_) => ("allow", None),
        Verdict::Veto(r) => ("veto", Some(r.name().to_string())),
        Verdict::Skip => ("skip", None),
    };
    let error = match &mark {
        Ok(_) => None,
        Err(e @ MarkError::Rpc(_)) => Some(format!("rpc: {e}")),
        Err(e) => Some(e.to_string()),
    };
    let m = mark.as_ref().ok();
    TickRecord {
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
        latency_ms: t0.elapsed().as_millis() as u64,
        error,
    }
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
