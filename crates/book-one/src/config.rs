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
    pub paper_dir: PathBuf,
    pub paper_start_date: Option<chrono::NaiveDate>,
    pub max_ticks: Option<u64>,
}

pub const TICK_FLOOR_SECS: u64 = 60;
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
                .unwrap_or(50),
            paper_dir: PathBuf::from(env("PAPER_DIR").unwrap_or_else(|| "paper".to_string())),
            paper_start_date,
            max_ticks: env("MAX_TICKS").map(|v| v.parse()).transpose()?,
        })
    }
}
