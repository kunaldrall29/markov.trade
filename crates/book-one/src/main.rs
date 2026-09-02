//! `book-one`: the house agent. One binary, two modes.
//!
//! `VENUE=shadow` (P08, this build): no keypair, no chain writes, no redteam.
//! Ticks every `TICK_SECONDS` (floor 60), reads the bound mark, runs the core
//! and the guard, records one tick row, and re-renders today's paper file
//! from the day's tick log. `VENUE=devnet` refuses to boot until P07 ships.
#![forbid(unsafe_code)]
#![warn(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod config;
mod core;
mod paper;
mod regime;
mod tick;

use std::process::ExitCode;

use markov_marks::{MarkSource, OnchainPyth};
use tracing::{error, info};

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
    let store = paper::PaperStore::new(cfg.paper_dir.clone());
    store.ensure_dirs()?;
    let started_at = chrono::Utc::now();
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
    // Days the runner was not up get an honest marker, never data.
    if let Some(start) = cfg.paper_start_date {
        let written = store.mark_missing_days(start, started_at.date_naive())?;
        for d in written {
            info!(day = %d, "wrote no-run marker");
        }
    }

    let mut n: u64 = 0;
    loop {
        n += 1;
        let now = chrono::Utc::now();
        let record = tick::run_tick(&cfg, &mut source, now, n).await;
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
        store.record_tick(&record, now)?;
        store.render_day(record.day, &source_label(&cfg), Some(started_at))?;

        if cfg.max_ticks.is_some_and(|m| n >= m) {
            info!(ticks = n, "MAX_TICKS reached, exiting");
            return Ok(());
        }
        let jitter = (now.timestamp() % 5).unsigned_abs();
        let sleep = std::time::Duration::from_secs(cfg.tick_seconds + jitter);
        tokio::select! {
            _ = tokio::time::sleep(sleep) => {}
            _ = tokio::signal::ctrl_c() => {
                info!(ticks = n, "interrupted, exiting");
                return Ok(());
            }
        }
    }
}

fn source_label(cfg: &config::Config) -> String {
    format!(
        "pyth-devnet-account SOL/USD {} (feed id in FACTS)",
        cfg.pyth_price_update_account
    )
}
