//! `book-one`: the house agent. One binary, two modes.
//!
//! `VENUE=shadow` (P08, this build): no keypair, no chain writes, no redteam.
//! Ticks every `TICK_SECONDS` (floor 60) on an interval anchored at boot,
//! reads the bound mark, runs the core and the guard, records one tick row,
//! and re-renders today's paper file from the day's tick log. `VENUE=devnet`
//! refuses to boot until P07 ships.
#![forbid(unsafe_code)]
#![warn(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use book_one::{config, core, paper, tick};

use std::process::ExitCode;
use std::time::Duration;

use chrono::Utc;
use markov_marks::{MarkSource, OnchainPyth};
use tracing::{error, info, warn};

fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let cfg = match config::Config::from_env() {
        Ok(c) => c,
        Err(e) => {
            error!(error = %e, "refusing to boot");
            return ExitCode::from(2);
        }
    };
    if cfg.venue != config::Venue::Shadow {
        error!("VENUE=devnet is not shipped until P07; refusing to boot");
        return ExitCode::from(2);
    }

    let rt = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => {
            error!(error = %e, "tokio runtime");
            return ExitCode::from(2);
        }
    };
    match rt.block_on(run(cfg)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            error!(error = %e, "runner stopped");
            ExitCode::from(1)
        }
    }
}

async fn run(cfg: config::Config) -> anyhow::Result<()> {
    let store = paper::PaperStore::new(cfg.paper_dir.clone(), cfg.tick_seconds);
    store.ensure_dirs()?;
    let label = source_label(&cfg);
    let mut source = OnchainPyth::new(
        cfg.rpc_http_url.clone(),
        Some(cfg.rpc_http_fallback.clone()),
        cfg.pyth_price_update_account.clone(),
        cfg.sol_usd_feed_id,
    );
    info!(
        venue = "shadow",
        tick_seconds = cfg.tick_seconds,
        mark_source = source.name(),
        mark_account = %cfg.pyth_price_update_account,
        max_mark_age_secs = cfg.max_mark_age_secs,
        paper_dir = %cfg.paper_dir.display(),
        "book-one starting"
    );

    // Resume today's log: continue the tick counter and keep the cadence
    // across a restart so two processes never write ticks seconds apart.
    let mut last_day = Utc::now().date_naive();
    let today_log = store.read_ticks(last_day)?;
    // Shadow mode holds no position, so this only ever carries the previous
    // tick's proposal — which is what hysteresis needs.
    let mut book = core::BookState::default();
    let mut n: u64 = today_log.ticks.len() as u64;
    if let Some(last_ts) = today_log.ticks.iter().map(|t| t.ts_unix).max() {
        let since = Utc::now().timestamp().saturating_sub(last_ts);
        if since >= 0 && (since as u64) < cfg.tick_seconds {
            let wait = cfg.tick_seconds - since as u64;
            info!(
                wait_secs = wait,
                "resuming today's log; holding the cadence"
            );
            tokio::time::sleep(Duration::from_secs(wait)).await;
        }
    }

    // Days the runner was not up get an honest marker — only on a directory
    // that has actually seen PAPER_START_DATE, never on a fresh volume.
    if let Some(start) = cfg.paper_start_date {
        if store.is_seeded(start)? {
            for d in store.mark_missing_days(start, last_day, &label)? {
                info!(day = %d, "wrote missing-day file");
            }
        } else {
            warn!(start = %start, "paper directory is not seeded with PAPER_START_DATE; no missing-day markers written");
        }
    }

    let mut interval = tokio::time::interval(Duration::from_secs(cfg.tick_seconds));
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    let ctrl_c = tokio::signal::ctrl_c();
    tokio::pin!(ctrl_c);

    loop {
        tokio::select! {
            _ = interval.tick() => {}
            _ = &mut ctrl_c => {
                info!(ticks = n, "interrupted, exiting");
                return Ok(());
            }
        }
        // 0–5 s jitter so a fleet of runners does not hit the RPC in lockstep.
        let jitter_ms = (Utc::now().timestamp_millis() as u64).wrapping_mul(2_654_435_761) % 5_000;
        tokio::time::sleep(Duration::from_millis(jitter_ms)).await;

        n += 1;
        let record = tick::run_tick(&cfg, &mut source, &mut book, n).await;
        info!(
            tick_id = %record.tick_id,
            slot = record.slot,
            regime = %record.regime,
            intent = %record.intent,
            verdict = %record.verdict,
            reason = record.reason.as_deref().unwrap_or("-"),
            mark_age_s = record.mark_age_s,
            latency_ms = record.latency_ms,
            error = record.error.as_deref().unwrap_or("-"),
            "tick"
        );
        store.record_tick(&record, Utc::now())?;
        if record.day != last_day {
            // First tick after midnight: close yesterday's file with any late tick.
            store.render_day(last_day, &label)?;
            last_day = record.day;
            n = 1;
        }
        store.render_day(record.day, &label)?;

        if cfg.max_ticks.is_some_and(|m| n >= m) {
            info!(ticks = n, "MAX_TICKS reached, exiting");
            return Ok(());
        }
    }
}

fn source_label(cfg: &config::Config) -> String {
    format!(
        "pyth-devnet-account SOL/USD {} (feed id in FACTS)",
        cfg.pyth_price_update_account
    )
}
