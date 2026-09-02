//! `Policy`: what the operator is allowed to do. Amendments are tighten-only.

use anchor_lang::prelude::*;

use crate::errors::MandateError;

pub const MAX_VENUES: usize = 4;
pub const MAX_TOKENS: usize = 4;

/// Bit positions in `Policy::allowed_actions`, in `ActionKind` order.
pub mod action_bits {
    pub const OPEN: u16 = 1 << 0;
    pub const INCREASE: u16 = 1 << 1;
    pub const REDUCE: u16 = 1 << 2;
    pub const CLOSE: u16 = 1 << 3;
    pub const FLATTEN: u16 = 1 << 4;
    pub const ALL: u16 = OPEN | INCREASE | REDUCE | CLOSE | FLATTEN;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq, InitSpace)]
pub struct Policy {
    pub venues: [Pubkey; MAX_VENUES],
    pub venues_len: u8,
    pub tokens: [Pubkey; MAX_TOKENS],
    pub tokens_len: u8,
    /// Bitmask over `action_bits`.
    pub allowed_actions: u16,
    /// Notional per action, in mint base units.
    pub per_tx_cap: u64,
    /// Notional per UTC day.
    pub daily_cap: u64,
    /// Data/compute spend per call and per UTC day.
    pub spend_per_call: u64,
    pub spend_daily: u64,
    pub max_slippage_bps: u16,
    /// Freshness in **seconds** since the mark's `publish_time` (ADR-003:
    /// seconds, not slots — devnet pacing is ≈165 ms/slot and moves).
    pub max_mark_age_secs: u64,
    /// Unix seconds. The expiry gate reads this; `amend_policy` may only
    /// shorten it. (docs/10 §1.2 puts expiry in the policy; the mandate keeps
    /// `created_at` only, so there is one source of truth.)
    pub expiry_ts: i64,
}

impl Policy {
    pub fn venue_allowed(&self, venue: &Pubkey) -> bool {
        self.venues
            .iter()
            .take(self.venues_len.min(MAX_VENUES as u8) as usize)
            .any(|v| v == venue)
    }

    pub fn token_allowed(&self, mint: &Pubkey) -> bool {
        self.tokens
            .iter()
            .take(self.tokens_len.min(MAX_TOKENS as u8) as usize)
            .any(|t| t == mint)
    }

    pub fn action_allowed(&self, bit: u16) -> bool {
        self.allowed_actions & bit == bit
    }

    /// Structural validity, checked on create and on amend.
    pub fn validate(&self) -> Result<()> {
        require!(
            self.venues_len as usize >= 1 && self.venues_len as usize <= MAX_VENUES,
            MandateError::InvalidPolicy
        );
        require!(
            self.tokens_len as usize >= 1 && self.tokens_len as usize <= MAX_TOKENS,
            MandateError::InvalidPolicy
        );
        require!(
            self.allowed_actions & !action_bits::ALL == 0,
            MandateError::InvalidPolicy
        );
        require!(self.max_slippage_bps <= 10_000, MandateError::InvalidPolicy);
        require!(self.max_mark_age_secs > 0, MandateError::InvalidPolicy);
        Ok(())
    }

    /// Tighten-only diff. Every numeric cap may decrease, every allowlist may
    /// shrink (new ⊆ old), the action bitmask may only lose bits, expiry may
    /// only shorten. A widening amendment is a hard error, not a no-op.
    pub fn assert_tightens(&self, new: &Policy) -> Result<()> {
        new.validate()?;
        require!(new.per_tx_cap <= self.per_tx_cap, MandateError::PolicyNotTightened);
        require!(new.daily_cap <= self.daily_cap, MandateError::PolicyNotTightened);
        require!(
            new.spend_per_call <= self.spend_per_call,
            MandateError::PolicyNotTightened
        );
        require!(new.spend_daily <= self.spend_daily, MandateError::PolicyNotTightened);
        require!(
            new.max_slippage_bps <= self.max_slippage_bps,
            MandateError::PolicyNotTightened
        );
        require!(
            new.max_mark_age_secs <= self.max_mark_age_secs,
            MandateError::PolicyNotTightened
        );
        require!(new.expiry_ts <= self.expiry_ts, MandateError::PolicyNotTightened);
        require!(
            new.allowed_actions & !self.allowed_actions == 0,
            MandateError::PolicyNotTightened
        );
        for i in 0..new.venues_len.min(MAX_VENUES as u8) as usize {
            require!(self.venue_allowed(&new.venues[i]), MandateError::PolicyNotTightened);
        }
        for i in 0..new.tokens_len.min(MAX_TOKENS as u8) as usize {
            require!(self.token_allowed(&new.tokens[i]), MandateError::PolicyNotTightened);
        }
        Ok(())
    }
}
