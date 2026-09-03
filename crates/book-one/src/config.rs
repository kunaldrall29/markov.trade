//! Environment configuration. Every value has a fail-closed default or refuses
//! to boot. Nothing here reads a keypair in shadow mode.

use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Venue {
    Shadow,
    Devnet,
}

#[derive(Clone, Debug)]
pub struct Config {
    pub venue: Venue,
    pub tick_seconds: u64,
    pub rpc_http_url: String,
    pub rpc_http_fallback: String,
    pub pyth_price_update_account: String,
    pub sol_usd_feed_id: [u8; 32],
    pub max_mark_age_secs: i64,
    pub per_tx_cap: u64,
    pub daily_cap: u64,
    pub spend_cap: u64,
    pub spend_daily_cap: u64,
    pub max_slippage_bps: u16,
    /// Ceiling on `|net delta|`, and on gross exposure, in mint base units.
    /// Enforced by the guard only in v0 (ADR-005).
    pub delta_band: u128,
    pub max_gross: u128,
    /// 500 = stop for the day once equity is 5% below the session's start.
    pub daily_loss_bps: u16,
    /// Devnet mode only; ignored in shadow.
    pub program_id: String,
    pub mandate: String,
    pub venue_program: String,
    pub market_id: [u8; 16],
    pub operator_key_path: String,
    /// The red team's `StaleOracle` probe needs a valid Pyth price update
    /// account that is **not** this mandate's. Absent means the probe is
    /// skipped and says so, rather than being faked.
    pub redteam_wrong_mark_account: Option<String>,
    pub max_actions_per_hour: u32,
    pub halt_env: String,
    pub halt_file: PathBuf,
    /// Health and metrics. Railway sets `PORT`.
    pub port: u16,
    pub submit_attempts: u32,
    pub paper_dir: PathBuf,
    pub paper_start_date: Option<chrono::NaiveDate>,
    pub max_ticks: Option<u64>,
}

pub const TICK_FLOOR_SECS: u64 = 60;

/// The settlement mint has 6 decimals, so one dollar is 1e6 base units. Every
/// cap below is quoted in dollars in the pack and in base units here; the
/// conversion lives in one place so no surface has to guess which it reads.
pub const E6: u64 = 1_000_000;

/// Gate B's policy, from `docs/11-AGENT-SPEC.md` §3 and P06's pre-flight:
/// per-trade $50, daily $200, delta band ±$20, gross ceiling $100, 50 bps.
pub const GATE_B_PER_TX_CAP: u64 = 50 * E6;
pub const GATE_B_DAILY_CAP: u64 = 200 * E6;
pub const GATE_B_DELTA_BAND: u128 = 20 * E6 as u128;
pub const GATE_B_MAX_GROSS: u128 = 100 * E6 as u128;
pub const GATE_B_MAX_SLIPPAGE_BPS: u16 = 50;
/// The spend budgets in the deployed template. Smaller than the notional caps
/// because a Gate B venue takes no custody, so an action's "spend" is fees and
/// data, not collateral.
pub const GATE_B_SPEND_PER_CALL: u64 = E6;
pub const GATE_B_SPEND_DAILY: u64 = 5 * E6;
pub const GATE_B_MAX_MARK_AGE_SECS: i64 = 150;
/// Stop for the day once equity is 5% below the session's start.
pub const GATE_B_DAILY_LOSS_BPS: u16 = 500;

/// The deployed Gate B programs (FACTS `PROGRAM_ID`, `DEMO_PERPS_ID`).
pub const DEFAULT_PROGRAM_ID: &str = "25CdYaZeB18QvUR7cTyZPgTZPNREb7t6xL8zmk1eXAU6";
pub const DEFAULT_VENUE_PROGRAM: &str = "3Zcd8XsFWBTVku5GxQjwEBC7sLrJhF8vadyTnTr56hxB";
/// Six actions an hour is roughly one every ten minutes against a 60-second
/// tick. A book that wants more than that is not the book this is.
pub const DEFAULT_MAX_ACTIONS_PER_HOUR: u32 = 6;
pub const DEFAULT_PUBLIC_DEVNET_RPC: &str = "https://api.devnet.solana.com";
pub const DEFAULT_PYTH_ACCOUNT: &str = "7UVimffxr9ow1uXYxsr4LHAcV58mLzhmwaeKvJ1pjLiE";
pub const DEFAULT_SOL_USD_FEED: &str =
    "ef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d";

fn env(key: &str) -> Option<String> {
    std::env::var(key).ok().filter(|v| !v.trim().is_empty())
}

impl Config {
    pub fn from_env() -> anyhow::Result<Config> {
        let venue = match env("VENUE").as_deref() {
            None | Some("shadow") => Venue::Shadow,
            Some("devnet") => Venue::Devnet,
            Some(other) => anyhow::bail!("VENUE must be shadow or devnet, got {other:?}"),
        };
        let tick_seconds: u64 = env("TICK_SECONDS")
            .map(|v| v.parse())
            .transpose()?
            .unwrap_or(TICK_FLOOR_SECS);
        if tick_seconds < TICK_FLOOR_SECS {
            anyhow::bail!(
                "TICK_SECONDS={tick_seconds} is below the Gate B floor of {TICK_FLOOR_SECS}"
            );
        }
        if env("MARK_SOURCE").as_deref().unwrap_or("onchain") != "onchain" {
            anyhow::bail!("only MARK_SOURCE=onchain is implemented in this build (ADR-003)");
        }
        let sol_usd_feed_id = markov_marks::parse_feed_id(
            &env("SOL_USD_FEED_ID").unwrap_or_else(|| DEFAULT_SOL_USD_FEED.to_string()),
        )?;
        let max_mark_age_secs: i64 = env("MARK_MAX_AGE_SECS")
            .map(|v| v.parse())
            .transpose()?
            .unwrap_or(150);
        if max_mark_age_secs <= 0 {
            anyhow::bail!("MARK_MAX_AGE_SECS must be positive");
        }
        let paper_start_date = env("PAPER_START_DATE")
            .map(|v| chrono::NaiveDate::parse_from_str(&v, "%Y-%m-%d"))
            .transpose()?;
        if venue == Venue::Shadow && paper_start_date.is_none() {
            anyhow::bail!("PAPER_START_DATE is required in shadow mode (docs/FACTS.md PAPER_START_DATE, never edited); without it missing days could be silently omitted");
        }
        let paper_dir_str = env("PAPER_DIR").unwrap_or_else(|| "paper".to_string());
        // A fixed-width market id, padded with zeroes exactly as the program
        // stores it. Longer than 16 bytes is a configuration error, not a
        // truncation: silently trading a different market is the worst
        // possible way to be wrong.
        let market_name = env("MARKET_ID").unwrap_or_else(|| "SOL-PERP".to_string());
        if market_name.len() > 16 {
            anyhow::bail!("MARKET_ID={market_name} is longer than 16 bytes");
        }
        let mut market_id = [0u8; 16];
        market_id[..market_name.len()].copy_from_slice(market_name.as_bytes());

        Ok(Config {
            venue,
            tick_seconds,
            rpc_http_url: env("RPC_HTTP_URL")
                .unwrap_or_else(|| DEFAULT_PUBLIC_DEVNET_RPC.to_string()),
            rpc_http_fallback: env("RPC_HTTP_FALLBACK")
                .unwrap_or_else(|| DEFAULT_PUBLIC_DEVNET_RPC.to_string()),
            pyth_price_update_account: env("PYTH_PRICE_UPDATE_ACCOUNT")
                .unwrap_or_else(|| DEFAULT_PYTH_ACCOUNT.to_string()),
            sol_usd_feed_id,
            max_mark_age_secs,
            per_tx_cap: env("PER_TX_CAP")
                .map(|v| v.parse())
                .transpose()?
                .unwrap_or(GATE_B_PER_TX_CAP),
            daily_cap: env("DAILY_CAP")
                .map(|v| v.parse())
                .transpose()?
                .unwrap_or(GATE_B_DAILY_CAP),
            // Gate B's venue takes no token custody, so an action spends
            // nothing; the cap exists so a custody venue cannot arrive later
            // and find it unset.
            spend_cap: env("SPEND_CAP")
                .map(|v| v.parse())
                .transpose()?
                .unwrap_or(GATE_B_SPEND_PER_CALL),
            spend_daily_cap: env("SPEND_DAILY_CAP")
                .map(|v| v.parse())
                .transpose()?
                .unwrap_or(GATE_B_SPEND_DAILY),
            max_slippage_bps: env("MAX_SLIPPAGE_BPS")
                .map(|v| v.parse())
                .transpose()?
                .unwrap_or(GATE_B_MAX_SLIPPAGE_BPS),
            delta_band: env("DELTA_BAND")
                .map(|v| v.parse())
                .transpose()?
                .unwrap_or(GATE_B_DELTA_BAND),
            max_gross: env("MAX_GROSS")
                .map(|v| v.parse())
                .transpose()?
                .unwrap_or(GATE_B_MAX_GROSS),
            daily_loss_bps: env("DAILY_LOSS_BPS")
                .map(|v| v.parse())
                .transpose()?
                .unwrap_or(GATE_B_DAILY_LOSS_BPS),
            program_id: env("PROGRAM_ID").unwrap_or_else(|| DEFAULT_PROGRAM_ID.to_string()),
            mandate: env("MANDATE").unwrap_or_default(),
            venue_program: env("VENUE_PROGRAM")
                .unwrap_or_else(|| DEFAULT_VENUE_PROGRAM.to_string()),
            market_id,
            operator_key_path: env("OPERATOR_KEY_PATH")
                .unwrap_or_else(|| "keys/operator.json".to_string()),
            redteam_wrong_mark_account: env("REDTEAM_WRONG_MARK_ACCOUNT"),
            max_actions_per_hour: env("MAX_ACTIONS_PER_HOUR")
                .map(|v| v.parse())
                .transpose()?
                .unwrap_or(DEFAULT_MAX_ACTIONS_PER_HOUR),
            halt_env: env("HALT_ENV").unwrap_or_else(|| "HALT".to_string()),
            halt_file: PathBuf::from(
                env("HALT_FILE").unwrap_or_else(|| format!("{paper_dir_str}/HALT")),
            ),
            port: env("PORT").map(|v| v.parse()).transpose()?.unwrap_or(8080),
            submit_attempts: env("SUBMIT_ATTEMPTS")
                .map(|v| v.parse())
                .transpose()?
                .unwrap_or(3),
            paper_dir: PathBuf::from(&paper_dir_str),
            paper_start_date,
            max_ticks: env("MAX_TICKS").map(|v| v.parse()).transpose()?,
        })
    }
}
