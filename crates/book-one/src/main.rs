//! `book-one`: the house agent. One binary, two modes.
//!
//! `VENUE=shadow` (P08, this build): no keypair, no chain writes, no redteam.
//! Ticks every `TICK_SECONDS` (floor 60) on an interval anchored at boot,
//! reads the bound mark, runs the core and the guard, records one tick row,
//! and re-renders today's paper file from the day's tick log. `VENUE=devnet`
//! refuses to boot until P07 ships.
#![forbid(unsafe_code)]
#![warn(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use book_one::{agent, chainstate, config, core, health, paper, runtime, submitter, tick};

use std::process::ExitCode;
use std::str::FromStr;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use markov_chain::Chain;
use markov_marks::{MarkSource, OnchainPyth};
use solana_instruction::AccountMeta;
use solana_pubkey::Pubkey;
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

/// Everything devnet mode needs, or a reason it cannot run.
///
/// This is P07's pre-flight, executed rather than described: the mandate is
/// read from the chain, its policy is compared with the Gate B template, the
/// operator key is loaded, and the venue accounts are derived. Any of these
/// failing is a refusal to boot — an agent that starts anyway and skips every
/// tick would look, on the tape, exactly like a quiet market.
fn boot_devnet(
    cfg: &config::Config,
    metrics: Arc<runtime::Metrics>,
) -> anyhow::Result<(agent::Agent, chainstate::MandateSnapshot)> {
    if cfg.mandate.trim().is_empty() {
        anyhow::bail!("VENUE=devnet needs MANDATE set to the mandate's address");
    }
    let program = Pubkey::from_str(&cfg.program_id)?;
    let mandate = Pubkey::from_str(&cfg.mandate)?;
    let venue_program = Pubkey::from_str(&cfg.venue_program)?;

    let chain = Chain::new(
        &cfg.rpc_http_url,
        &cfg.rpc_http_fallback,
        Duration::from_secs(20),
    );
    let snapshot = chainstate::read_mandate(
        &chain,
        &mandate,
        cfg.delta_band,
        cfg.max_gross,
        cfg.daily_loss_bps,
    )?;

    // Pre-flight 2: does the policy on chain match the template the pack
    // specifies? A mismatch is reported in full and refuses the boot — running
    // against a policy nobody agreed to is how a demo becomes a claim.
    let differences = chainstate::gate_b_policy_differences(&snapshot.policy);
    if !differences.is_empty() {
        for d in &differences {
            error!(difference = %d, "mandate policy does not match the Gate B template");
        }
        anyhow::bail!("{} policy differences; refusing to boot", differences.len());
    }
    if !snapshot.venues.contains(&venue_program) {
        anyhow::bail!(
            "the mandate's policy does not allow venue {venue_program}; every action would be refused at gate 5"
        );
    }

    let (registry, _) = Pubkey::find_program_address(&[b"registry"], &program);
    let (event_authority, _) = Pubkey::find_program_address(&[b"__event_authority"], &program);
    let (venue_market, _) =
        Pubkey::find_program_address(&[b"market", cfg.market_id.as_ref()], &venue_program);
    let (venue_mark, _) =
        Pubkey::find_program_address(&[b"mark", cfg.market_id.as_ref()], &venue_program);
    let (venue_position, _) = Pubkey::find_program_address(
        &[b"pos", mandate.as_ref(), cfg.market_id.as_ref()],
        &venue_program,
    );

    let wiring = submitter::Wiring {
        program,
        registry,
        mandate,
        mint: snapshot.mint,
        vault: snapshot.vault,
        price_update: snapshot.mark_account,
        venue_program,
        token_program: Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA")?,
        event_authority,
        // The mandate is forwarded because the venue needs its signature. The
        // vault is not, and must never be: gate 15 refuses it (ADR-009).
        venue_accounts: vec![
            AccountMeta::new_readonly(mandate, false),
            AccountMeta::new_readonly(venue_market, false),
            AccountMeta::new_readonly(venue_mark, false),
            AccountMeta::new(venue_position, false),
        ],
        market_id: cfg.market_id,
    };
    let sub = submitter::Submitter::new(&cfg.operator_key_path, wiring)?;

    let wrong_mark_account = cfg
        .redteam_wrong_mark_account
        .as_deref()
        .map(Pubkey::from_str)
        .transpose()?;
    if wrong_mark_account.is_none() {
        warn!("REDTEAM_WRONG_MARK_ACCOUNT is unset; the StaleOracle probe will be skipped");
    }

    info!(
        operator = %sub.operator(),
        mandate = %mandate,
        vault = %snapshot.vault,
        mint = %snapshot.mint,
        venue = %venue_program,
        venue_position = %venue_position,
        per_tx_cap = snapshot.policy.per_tx_cap,
        daily_cap = snapshot.policy.daily_cap,
        state = ?snapshot.state,
        "devnet pre-flight passed"
    );

    Ok((
        agent::Agent {
            chain,
            submitter: sub,
            governor: runtime::Governor::new(cfg.max_actions_per_hour),
            metrics,
            redteam_last: Default::default(),
            // §6: never in shadow. Shadow never reaches here, but stating it
            // where the flag is set is better than relying on the call site.
            redteam_enabled: cfg.venue == config::Venue::Devnet,
            halt_env: cfg.halt_env.clone(),
            halt_file: cfg.halt_file.clone(),
            wrong_mark_account,
            nonce: 0,
            attempts: cfg.submit_attempts,
        },
        snapshot,
    ))
}

async fn run(cfg: config::Config) -> anyhow::Result<()> {
    let store = paper::PaperStore::new(cfg.paper_dir.clone(), cfg.tick_seconds);
    store.ensure_dirs()?;

    let metrics = Arc::new(runtime::Metrics::default());
    // Devnet mode boots its chain wiring before anything else, so a
    // misconfiguration fails loudly at start rather than as a skipped tick an
    // hour later.
    let mut agent = match cfg.venue {
        config::Venue::Shadow => None,
        config::Venue::Devnet => {
            let (a, snap) = boot_devnet(&cfg, metrics.clone())?;
            info!(state = ?snap.state, "devnet mode armed");
            Some(a)
        }
    };

    // Health and metrics on their own socket, so Railway can tell a wedged
    // process from a quiet one (BACKLOG, open since P08).
    let health = Arc::new(health::Health {
        metrics: metrics.clone(),
        tick_seconds: cfg.tick_seconds,
        started_unix: Utc::now().timestamp(),
    });
    match tokio::net::TcpListener::bind(("0.0.0.0", cfg.port)).await {
        Ok(listener) => {
            info!(port = cfg.port, "health and metrics listening");
            tokio::spawn(health::serve(listener, health, || Utc::now().timestamp()));
        }
        // Not fatal: the agent's job is the tape, and losing the healthcheck
        // should not stop it producing one.
        Err(e) => error!(port = cfg.port, error = %e, "could not bind the health port"),
    }
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
        let (mut record, verdict, policy) = tick::run_tick(&cfg, &mut source, &mut book, n).await;
        runtime::Metrics::incr(&metrics.ticks_total);
        metrics
            .last_tick_unix
            .store(record.ts_unix, Ordering::Relaxed);
        match verdict {
            markov_guard::Verdict::Skip => runtime::Metrics::incr(&metrics.skips_total),
            markov_guard::Verdict::Allow(_) => runtime::Metrics::incr(&metrics.allows_total),
            markov_guard::Verdict::Veto(_) => runtime::Metrics::incr(&metrics.vetoes_total),
        }

        if let Some(a) = agent.as_mut() {
            // Re-read the mandate every tick: the owner may have paused,
            // revoked or amended it since the last one, and acting on a
            // remembered policy is acting on a policy that is not in force.
            match chainstate::read_mandate(
                &a.chain,
                &a.submitter.wiring().mandate,
                cfg.delta_band,
                cfg.max_gross,
                cfg.daily_loss_bps,
            ) {
                Ok(snapshot) => {
                    a.act(
                        record.ts_unix,
                        record.slot,
                        &snapshot,
                        &snapshot.policy,
                        &verdict,
                        &mut record,
                    );
                }
                Err(e) => {
                    warn!(error = %e, "could not read the mandate; nothing submitted this tick");
                    record.error = Some(format!("mandate read: {e}"));
                }
            }
        } else {
            record.withheld = Some(runtime::WITHHELD_SHADOW.to_string());
        }
        let _ = &policy;
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
            signature = record.signature.as_deref().unwrap_or("-"),
            forced = record.forced,
            withheld = record.withheld.as_deref().unwrap_or("-"),
            onchain_reason = record.onchain_reason.as_deref().unwrap_or("-"),
            redteam_probe = record.redteam_probe.as_deref().unwrap_or("-"),
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
