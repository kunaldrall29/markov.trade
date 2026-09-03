//! Everything in the agent that touches the network, in one crate, so that
//! "the guard is pure" is a property of the dependency graph rather than a
//! promise. `markov-guard` cannot reach this and CI checks that it does not.
//!
//! Two endpoints, always: FACTS names a primary and a fallback, and a read
//! that fails on the primary is retried once on the fallback before it becomes
//! an error. A read that fails on both is a `Skip` upstream, never a guess.
//!
//! Sending is deliberately unclever. The caller builds and signs a
//! transaction; this crate sends *those bytes*, and on a retry sends **the
//! same bytes again**. It never rebuilds, never re-signs with a new blockhash
//! mid-retry, and never touches a parameter. `intent_id` is the idempotency
//! key at the program level; sending identical bytes is the idempotency at
//! this one (`docs/11` §5).
#![forbid(unsafe_code)]
#![warn(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::time::Duration;

use solana_account::Account;
use solana_client::rpc_client::RpcClient;
use solana_commitment_config::CommitmentConfig;
use solana_hash::Hash;
use solana_pubkey::Pubkey;
use solana_signature::Signature;
use solana_transaction::Transaction;

#[derive(Debug, thiserror::Error)]
pub enum ChainError {
    #[error("both endpoints failed: primary {primary}; fallback {fallback}")]
    BothEndpoints { primary: String, fallback: String },
    #[error("account {0} not found")]
    NotFound(Pubkey),
    #[error("not confirmed after {attempts} attempts: {last}")]
    NotConfirmed { attempts: u32, last: String },
}

/// What the chain did with a transaction that *landed*.
///
/// A landed transaction is not necessarily a successful one, and that
/// distinction is the whole point: this program commits its refusals, so
/// `err: None` with a `RefusalReceipt` inside is the normal way to be told no.
/// `err: Some` means the transaction failed outright.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Landed {
    pub signature: Signature,
    pub slot: u64,
    pub err: Option<String>,
    pub logs: Vec<String>,
}

impl Landed {
    pub fn succeeded(&self) -> bool {
        self.err.is_none()
    }
}

pub struct Chain {
    primary: RpcClient,
    fallback: RpcClient,
    primary_url: String,
    fallback_url: String,
}

impl Chain {
    pub fn new(primary_url: &str, fallback_url: &str, timeout: Duration) -> Chain {
        let commitment = CommitmentConfig::confirmed();
        Chain {
            primary: RpcClient::new_with_timeout_and_commitment(
                primary_url.to_string(),
                timeout,
                commitment,
            ),
            fallback: RpcClient::new_with_timeout_and_commitment(
                fallback_url.to_string(),
                timeout,
                commitment,
            ),
            primary_url: primary_url.to_string(),
            fallback_url: fallback_url.to_string(),
        }
    }

    /// Run `f` against the primary, then the fallback. Both failing is an
    /// error naming both, so a log line says which endpoint was tried.
    fn either<T>(&self, f: impl Fn(&RpcClient) -> Result<T, String>) -> Result<T, ChainError> {
        match f(&self.primary) {
            Ok(v) => Ok(v),
            Err(primary) => {
                tracing::warn!(url = %self.primary_url, error = %primary, "primary rpc failed, trying fallback");
                f(&self.fallback).map_err(|fallback| {
                    tracing::warn!(url = %self.fallback_url, error = %fallback, "fallback rpc failed too");
                    ChainError::BothEndpoints { primary, fallback }
                })
            }
        }
    }

    pub fn slot(&self) -> Result<u64, ChainError> {
        self.either(|c| c.get_slot().map_err(|e| e.to_string()))
    }

    pub fn blockhash(&self) -> Result<Hash, ChainError> {
        self.either(|c| c.get_latest_blockhash().map_err(|e| e.to_string()))
    }

    /// `Ok(None)` when the account genuinely does not exist, which is a fact;
    /// an error only when we could not find out, which is not.
    pub fn account(&self, key: &Pubkey) -> Result<Option<Account>, ChainError> {
        self.either(
            |c| match c.get_account_with_commitment(key, c.commitment()) {
                Ok(r) => Ok(r.value),
                Err(e) => Err(e.to_string()),
            },
        )
    }

    pub fn account_or_missing(&self, key: &Pubkey) -> Result<Account, ChainError> {
        self.account(key)?.ok_or(ChainError::NotFound(*key))
    }

    /// Send an already-signed transaction and wait for `confirmed`.
    ///
    /// The same bytes are re-sent on each attempt. A transaction that lands is
    /// returned whether or not the program liked it — deciding what a refusal
    /// means is the caller's job, and re-sending one would be retrying a
    /// *result*.
    pub fn send_confirm(&self, tx: &Transaction, attempts: u32) -> Result<Landed, ChainError> {
        let mut last = String::from("no attempt made");
        for attempt in 0..attempts.max(1) {
            if attempt > 0 {
                std::thread::sleep(backoff(attempt));
            }
            match self.either(|c| {
                c.send_and_confirm_transaction(tx)
                    .map_err(|e| e.to_string())
            }) {
                Ok(signature) => return self.landed(signature),
                Err(e) => {
                    last = e.to_string();
                    // The transaction may have landed even though the reply
                    // did not arrive. Ask before sending again, so a timeout
                    // cannot become a second submission.
                    if let Some(landed) = self.already_landed(tx) {
                        return Ok(landed);
                    }
                    tracing::warn!(attempt, error = %last, "send failed, retrying the same bytes");
                }
            }
        }
        Err(ChainError::NotConfirmed { attempts, last })
    }

    /// Did these exact bytes already land? The signature of a signed
    /// transaction is fixed, so this is a lookup, not a guess.
    fn already_landed(&self, tx: &Transaction) -> Option<Landed> {
        let signature = *tx.signatures.first()?;
        self.landed(signature).ok()
    }

    fn landed(&self, signature: Signature) -> Result<Landed, ChainError> {
        let status = self.either(|c| {
            c.get_signature_statuses(&[signature])
                .map(|r| r.value.into_iter().next().flatten())
                .map_err(|e| e.to_string())
        })?;
        let status = status.ok_or(ChainError::NotConfirmed {
            attempts: 1,
            last: format!("{signature} has no status"),
        })?;
        Ok(Landed {
            signature,
            slot: status.slot,
            err: status.err.map(|e| e.to_string()),
            logs: Vec::new(),
        })
    }

    /// Fetch the base58 instruction data of every **inner** instruction of a
    /// landed transaction.
    ///
    /// This is where receipts live. `emit_cpi!` writes an event as a
    /// self-CPI, so a receipt is instruction data, not a log line — the
    /// program emits no `msg!` at all. Reading the data rather than the log is
    /// also what the indexer does, so the agent and the indexer agree by
    /// construction rather than by convention.
    pub fn inner_instruction_data(
        &self,
        signature: &Signature,
    ) -> Result<Vec<Vec<u8>>, ChainError> {
        use solana_client::rpc_config::RpcTransactionConfig;
        use solana_transaction_status_client_types::{
            option_serializer::OptionSerializer, UiInstruction, UiTransactionEncoding,
        };
        let cfg = RpcTransactionConfig {
            encoding: Some(UiTransactionEncoding::Json),
            commitment: Some(CommitmentConfig::confirmed()),
            max_supported_transaction_version: Some(0),
        };
        let tx = self.either(|c| {
            c.get_transaction_with_config(signature, cfg)
                .map_err(|e| e.to_string())
        })?;
        let Some(meta) = tx.transaction.meta else {
            return Ok(Vec::new());
        };
        let OptionSerializer::Some(inner) = meta.inner_instructions else {
            return Ok(Vec::new());
        };
        let mut out = Vec::new();
        for set in inner {
            for ix in set.instructions {
                if let UiInstruction::Compiled(c) = ix {
                    if let Ok(bytes) = bs58::decode(&c.data).into_vec() {
                        out.push(bytes);
                    }
                }
            }
        }
        Ok(out)
    }

    /// Fetch the logs a landed transaction produced. Kept for diagnostics —
    /// the program emits none, so this is empty for a refusal.
    pub fn logs(&self, signature: &Signature) -> Result<Vec<String>, ChainError> {
        use solana_client::rpc_config::RpcTransactionConfig;
        let cfg = RpcTransactionConfig {
            commitment: Some(CommitmentConfig::confirmed()),
            max_supported_transaction_version: Some(0),
            ..RpcTransactionConfig::default()
        };
        let tx = self.either(|c| {
            c.get_transaction_with_config(signature, cfg)
                .map_err(|e| e.to_string())
        })?;
        Ok(tx
            .transaction
            .meta
            .and_then(|m| Option::<Vec<String>>::from(m.log_messages))
            .unwrap_or_default())
    }
}

/// Exponential, capped. Attempt 1 waits 1s, 2 waits 2s, 3 waits 4s.
///
/// Capped at 8s because the tick floor is 60s: a backoff longer than that
/// would push a retry into the next tick, where a *newer* intent should be
/// making the decision instead.
pub fn backoff(attempt: u32) -> Duration {
    let secs = 1u64 << attempt.min(3);
    Duration::from_secs(secs.min(8))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backoff_is_exponential_and_capped_below_the_tick_floor() {
        assert_eq!(backoff(0), Duration::from_secs(1));
        assert_eq!(backoff(1), Duration::from_secs(2));
        assert_eq!(backoff(2), Duration::from_secs(4));
        assert_eq!(backoff(3), Duration::from_secs(8));
        // Never longer than the cap, whatever it is asked for: a retry that
        // outlives its tick is a stale decision.
        for attempt in 4..100 {
            assert_eq!(
                backoff(attempt),
                Duration::from_secs(8),
                "attempt {attempt}"
            );
        }
        let total: Duration = (0..3).map(backoff).sum();
        assert!(
            total < Duration::from_secs(60),
            "three retries must fit inside one tick, got {total:?}"
        );
    }

    #[test]
    fn a_landed_transaction_with_an_error_did_not_succeed() {
        let l = Landed {
            signature: Signature::default(),
            slot: 1,
            err: None,
            logs: vec![],
        };
        assert!(
            l.succeeded(),
            "a committed refusal is a success at this layer"
        );
        assert!(!Landed {
            err: Some("custom program error: 0x1771".into()),
            ..l
        }
        .succeeded());
    }
}
