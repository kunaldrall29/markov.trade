//! `MarkSource`: where the book's price comes from.
//!
//! Gate B mark (ADR-003): the Pyth sponsored devnet `PriceUpdateV2` account for
//! SOL/USD, read over RPC and decoded here. The reader binds the account —
//! owner must be the Pyth receiver program, `feed_id` must match, verification
//! must be `Full` — or the mark is refused. A refused or unreadable mark is an
//! error, never a default price.
#![forbid(unsafe_code)]
#![warn(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::future::Future;

/// Pyth receiver program on devnet and mainnet (docs/FACTS.md PYTH_RECEIVER_PROGRAM).
pub const PYTH_RECEIVER_PROGRAM: &str = "rec5EKMGg6MxZYaMdyBfgwp4d5rB9T1VQH5pJv5LtFJ";
/// Anchor account discriminator of `PriceUpdateV2`: `sha256("account:PriceUpdateV2")[..8]`.
pub const PRICE_UPDATE_V2_DISCRIMINATOR: [u8; 8] = [34, 241, 35, 99, 157, 126, 244, 205];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mark {
    pub price: i64,
    pub conf: u64,
    pub exponent: i32,
    pub publish_time: i64,
    pub posted_slot: u64,
    /// Slot the reader observed when it fetched the account.
    pub observed_slot: u64,
    pub source: &'static str,
}

impl Mark {
    pub fn price_f64(&self) -> f64 {
        self.price as f64 * 10f64.powi(self.exponent)
    }
    pub fn age_secs(&self, now_unix: i64) -> i64 {
        now_unix.saturating_sub(self.publish_time)
    }
    pub fn age_slots(&self) -> u64 {
        self.observed_slot.saturating_sub(self.posted_slot)
    }

    /// The price rescaled to 1e6, the integer scale the program, the guard and
    /// every surface use.
    ///
    /// `None` rather than a fallback: a negative price is not a price, and a
    /// rescaling that overflows is a number we cannot state. A saturated or
    /// zeroed value here would become a mark the book traded on.
    pub fn price_e6(&self) -> Option<u64> {
        let p = u64::try_from(self.price).ok()?;
        let shift = self.exponent.checked_add(6)?;
        if shift >= 0 {
            let factor = 10u64.checked_pow(u32::try_from(shift).ok()?)?;
            p.checked_mul(factor)
        } else {
            let factor = 10u64.checked_pow(u32::try_from(shift.checked_neg()?).ok()?)?;
            Some(p / factor)
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MarkError {
    #[error("rpc: {0}")]
    Rpc(String),
    #[error("account {0} not found")]
    NotFound(String),
    #[error("mark account is not owned by the Pyth receiver (owner {0})")]
    WrongOwner(String),
    #[error("mark account carries feed {0}, expected {1}")]
    WrongFeed(String, String),
    #[error("mark account verification level is not Full")]
    NotFullyVerified,
    #[error("malformed PriceUpdateV2 ({0})")]
    Malformed(&'static str),
    #[error("replay exhausted")]
    Exhausted,
}

pub trait MarkSource {
    fn get(&mut self) -> impl Future<Output = Result<Mark, MarkError>> + Send;
    fn name(&self) -> &'static str;
}

/// Decoded `PriceUpdateV2` (pyth-solana-receiver-sdk layout, borsh):
/// 8 discriminator | 32 write_authority | VerificationLevel (1 byte tag; Partial
/// carries 1 extra byte) | PriceFeedMessage { feed_id[32], price i64, conf u64,
/// exponent i32, publish_time i64, prev_publish_time i64, ema_price i64,
/// ema_conf u64 } | posted_slot u64. The account is allocated for the larger
/// `Partial` variant (134 bytes), so a `Full` account ends with one padding
/// byte; trailing bytes are ignored.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PriceUpdateV2 {
    pub feed_id: [u8; 32],
    pub verification_full: bool,
    pub price: i64,
    pub conf: u64,
    pub exponent: i32,
    pub publish_time: i64,
    pub posted_slot: u64,
}

pub fn decode_price_update_v2(data: &[u8]) -> Result<PriceUpdateV2, MarkError> {
    if data.get(..8) != Some(&PRICE_UPDATE_V2_DISCRIMINATOR[..]) {
        return Err(MarkError::Malformed("discriminator"));
    }
    let mut o = 8usize + 32; // discriminator + write_authority
    let tag = *data
        .get(o)
        .ok_or(MarkError::Malformed("short: verification tag"))?;
    o += 1;
    let verification_full = match tag {
        0 => {
            o += 1; // num_signatures
            false
        }
        1 => true,
        _ => return Err(MarkError::Malformed("verification tag")),
    };
    let take = |o: &mut usize, n: usize| -> Result<&[u8], MarkError> {
        let s = data
            .get(*o..*o + n)
            .ok_or(MarkError::Malformed("short: message"))?;
        *o += n;
        Ok(s)
    };
    let mut feed_id = [0u8; 32];
    feed_id.copy_from_slice(take(&mut o, 32)?);
    let price = i64::from_le_bytes(arr8(take(&mut o, 8)?)?);
    let conf = u64::from_le_bytes(arr8(take(&mut o, 8)?)?);
    let exponent = i32::from_le_bytes(arr4(take(&mut o, 4)?)?);
    let publish_time = i64::from_le_bytes(arr8(take(&mut o, 8)?)?);
    let _prev_publish_time = take(&mut o, 8)?;
    let _ema_price = take(&mut o, 8)?;
    let _ema_conf = take(&mut o, 8)?;
    let posted_slot = u64::from_le_bytes(arr8(take(&mut o, 8)?)?);
    Ok(PriceUpdateV2 {
        feed_id,
        verification_full,
        price,
        conf,
        exponent,
        publish_time,
        posted_slot,
    })
}

fn arr8(s: &[u8]) -> Result<[u8; 8], MarkError> {
    s.try_into().map_err(|_| MarkError::Malformed("u64"))
}
fn arr4(s: &[u8]) -> Result<[u8; 4], MarkError> {
    s.try_into().map_err(|_| MarkError::Malformed("i32"))
}

/// Bind a decoded account to what the policy expects. Same three checks the
/// program will make on chain (ADR-003).
pub fn bind(
    owner: &str,
    update: &PriceUpdateV2,
    expected_feed: &[u8; 32],
) -> Result<(), MarkError> {
    if owner != PYTH_RECEIVER_PROGRAM {
        return Err(MarkError::WrongOwner(owner.to_string()));
    }
    if &update.feed_id != expected_feed {
        return Err(MarkError::WrongFeed(
            hex(&update.feed_id),
            hex(expected_feed),
        ));
    }
    if !update.verification_full {
        return Err(MarkError::NotFullyVerified);
    }
    Ok(())
}

pub fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

pub fn parse_feed_id(s: &str) -> Result<[u8; 32], MarkError> {
    let s = s.trim().trim_start_matches("0x");
    if s.len() != 64 {
        return Err(MarkError::Malformed("feed id length"));
    }
    let mut out = [0u8; 32];
    for (i, chunk) in s.as_bytes().chunks(2).enumerate() {
        let h = std::str::from_utf8(chunk).map_err(|_| MarkError::Malformed("feed id"))?;
        out[i] = u8::from_str_radix(h, 16).map_err(|_| MarkError::Malformed("feed id hex"))?;
    }
    Ok(out)
}

/// Reads the Pyth `PriceUpdateV2` account over JSON-RPC.
pub struct OnchainPyth {
    client: reqwest::Client,
    rpc_url: String,
    fallback_url: Option<String>,
    account: String,
    feed_id: [u8; 32],
}

impl OnchainPyth {
    pub fn new(
        rpc_url: String,
        fallback_url: Option<String>,
        account: String,
        feed_id: [u8; 32],
    ) -> Self {
        Self {
            client: reqwest::Client::new(),
            rpc_url,
            fallback_url,
            account,
            feed_id,
        }
    }

    async fn rpc(
        &self,
        url: &str,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, MarkError> {
        let body = serde_json::json!({"jsonrpc":"2.0","id":1,"method":method,"params":params});
        let resp = self
            .client
            .post(url)
            .json(&body)
            .timeout(std::time::Duration::from_secs(15))
            .send()
            .await
            .map_err(|e| MarkError::Rpc(e.to_string()))?;
        let status = resp.status();
        let v: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| MarkError::Rpc(format!("{status}: {e}")))?;
        if let Some(err) = v.get("error") {
            return Err(MarkError::Rpc(err.to_string()));
        }
        v.get("result")
            .cloned()
            .ok_or_else(|| MarkError::Rpc(format!("{status}: no result")))
    }

    async fn fetch_from(&self, url: &str) -> Result<Mark, MarkError> {
        let slot = self
            .rpc(
                url,
                "getSlot",
                serde_json::json!([{"commitment":"confirmed"}]),
            )
            .await?
            .as_u64()
            .ok_or(MarkError::Rpc("getSlot: not a number".into()))?;
        let info = self
            .rpc(
                url,
                "getAccountInfo",
                serde_json::json!([self.account, {"encoding":"base64","commitment":"confirmed"}]),
            )
            .await?;
        let value = info
            .get("value")
            .filter(|v| !v.is_null())
            .ok_or_else(|| MarkError::NotFound(self.account.clone()))?;
        let owner = value.get("owner").and_then(|o| o.as_str()).unwrap_or("");
        let b64 = value
            .get("data")
            .and_then(|d| d.get(0))
            .and_then(|d| d.as_str())
            .ok_or(MarkError::Malformed("data field"))?;
        use base64::Engine as _;
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(b64)
            .map_err(|_| MarkError::Malformed("base64"))?;
        let update = decode_price_update_v2(&bytes)?;
        bind(owner, &update, &self.feed_id)?;
        Ok(Mark {
            price: update.price,
            conf: update.conf,
            exponent: update.exponent,
            publish_time: update.publish_time,
            posted_slot: update.posted_slot,
            observed_slot: slot,
            source: "pyth-devnet-account",
        })
    }
}

impl MarkSource for OnchainPyth {
    async fn get(&mut self) -> Result<Mark, MarkError> {
        match self.fetch_from(&self.rpc_url.clone()).await {
            Ok(m) => Ok(m),
            Err(MarkError::Rpc(e)) => match &self.fallback_url {
                Some(fb) if fb != &self.rpc_url => {
                    let fb = fb.clone();
                    // A binding refusal from the fallback stays a binding refusal;
                    // only an RPC failure is folded into the RPC error.
                    self.fetch_from(&fb).await.map_err(|e2| match e2 {
                        MarkError::Rpc(f) => MarkError::Rpc(format!("primary: {e}; fallback: {f}")),
                        other => other,
                    })
                }
                _ => Err(MarkError::Rpc(e)),
            },
            Err(other) => Err(other),
        }
    }
    fn name(&self) -> &'static str {
        "pyth-devnet-account"
    }
}

/// Replays a fixed sequence of marks. For tests and the stale-mark redteam tick.
pub struct Replay {
    marks: std::collections::VecDeque<Mark>,
}

impl Replay {
    pub fn new(marks: Vec<Mark>) -> Self {
        Self {
            marks: marks.into(),
        }
    }
}

impl MarkSource for Replay {
    async fn get(&mut self) -> Result<Mark, MarkError> {
        self.marks.pop_front().ok_or(MarkError::Exhausted)
    }
    fn name(&self) -> &'static str {
        "replay"
    }
}

#[cfg(test)]
mod price_e6_tests {
    use super::*;

    fn mark(price: i64, exponent: i32) -> Mark {
        Mark {
            price,
            conf: 0,
            exponent,
            publish_time: 0,
            posted_slot: 0,
            observed_slot: 0,
            source: "test",
        }
    }

    #[test]
    fn rescales_to_1e6_both_ways() {
        // Pyth's devnet SOL/USD is expo -8: 10429498839 is $104.29498839.
        assert_eq!(mark(10_429_498_839, -8).price_e6(), Some(104_294_988));
        // An exponent coarser than 1e6 scales up instead of down.
        assert_eq!(mark(104, 0).price_e6(), Some(104_000_000));
        assert_eq!(mark(1_042_949, -4).price_e6(), Some(104_294_900));
    }

    #[test]
    fn a_price_we_cannot_state_is_none_not_a_fallback() {
        assert_eq!(mark(-1, -8).price_e6(), None, "negative is not a price");
        assert_eq!(
            mark(i64::MAX, 0).price_e6(),
            None,
            "overflow is not a price"
        );
        assert_eq!(mark(1, 300).price_e6(), None, "absurd exponent");
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    fn encode(
        full: bool,
        feed: [u8; 32],
        price: i64,
        expo: i32,
        publish: i64,
        slot: u64,
    ) -> Vec<u8> {
        let mut v = PRICE_UPDATE_V2_DISCRIMINATOR.to_vec();
        v.extend_from_slice(&[7u8; 32]); // write_authority
        if full {
            v.push(1);
        } else {
            v.push(0);
            v.push(3);
        }
        v.extend_from_slice(&feed);
        v.extend_from_slice(&price.to_le_bytes());
        v.extend_from_slice(&1_500_000u64.to_le_bytes());
        v.extend_from_slice(&expo.to_le_bytes());
        v.extend_from_slice(&publish.to_le_bytes());
        v.extend_from_slice(&(publish - 50).to_le_bytes());
        v.extend_from_slice(&price.to_le_bytes());
        v.extend_from_slice(&1_400_000u64.to_le_bytes());
        v.extend_from_slice(&slot.to_le_bytes());
        if full {
            v.push(0); // padding: the account is sized for the Partial variant
        }
        v
    }

    const FEED: [u8; 32] = [0xef; 32];

    #[test]
    fn decodes_full_price_update_v2() {
        let bytes = encode(true, FEED, 9_999_848_408, -8, 1_788_000_000, 491_633_562);
        assert_eq!(bytes.len(), 134);
        let u = decode_price_update_v2(&bytes).unwrap();
        assert!(u.verification_full);
        assert_eq!(u.price, 9_999_848_408);
        assert_eq!(u.exponent, -8);
        assert_eq!(u.publish_time, 1_788_000_000);
        assert_eq!(u.posted_slot, 491_633_562);
        assert!(bind(PYTH_RECEIVER_PROGRAM, &u, &FEED).is_ok());
    }

    #[test]
    fn partial_verification_is_refused() {
        let u = decode_price_update_v2(&encode(false, FEED, 1, -8, 1, 1)).unwrap();
        assert!(matches!(
            bind(PYTH_RECEIVER_PROGRAM, &u, &FEED),
            Err(MarkError::NotFullyVerified)
        ));
    }

    #[test]
    fn wrong_owner_is_refused() {
        let u = decode_price_update_v2(&encode(true, FEED, 1, -8, 1, 1)).unwrap();
        assert!(matches!(
            bind("11111111111111111111111111111111", &u, &FEED),
            Err(MarkError::WrongOwner(_))
        ));
    }

    #[test]
    fn wrong_feed_is_refused() {
        let u = decode_price_update_v2(&encode(true, [1u8; 32], 1, -8, 1, 1)).unwrap();
        assert!(matches!(
            bind(PYTH_RECEIVER_PROGRAM, &u, &FEED),
            Err(MarkError::WrongFeed(_, _))
        ));
    }

    #[test]
    fn short_account_is_malformed_not_a_price() {
        assert!(matches!(
            decode_price_update_v2(&[0u8; 40]),
            Err(MarkError::Malformed(_))
        ));
    }

    #[test]
    fn feed_id_parses_with_or_without_prefix() {
        let s = "ef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d";
        assert_eq!(parse_feed_id(s).unwrap()[0], 0xef);
        assert_eq!(parse_feed_id(&format!("0x{s}")).unwrap()[31], 0x6d);
        assert!(parse_feed_id("abc").is_err());
    }

    #[test]
    fn wrong_discriminator_is_refused_before_any_field_is_read() {
        let mut bytes = encode(true, FEED, 1, -8, 1, 1);
        bytes[0] ^= 0xff;
        assert!(matches!(
            decode_price_update_v2(&bytes),
            Err(MarkError::Malformed("discriminator"))
        ));
    }

    #[test]
    fn age_never_overflows() {
        let m = Mark {
            price: 1,
            conf: 1,
            exponent: -8,
            publish_time: i64::MIN,
            posted_slot: 1,
            observed_slot: 1,
            source: "replay",
        };
        assert_eq!(m.age_secs(0), i64::MAX);
    }
}
