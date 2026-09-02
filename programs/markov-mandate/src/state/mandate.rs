//! `Mandate`: one depositor's account. SMA, never a pool (ADR-02).

use anchor_lang::prelude::*;

use crate::state::policy::Policy;

/// How many recent `intent_id`s the replay guard remembers. One tick is 60 s
/// and the submitter retries at most three times, so eight is generous.
pub const RECENT_INTENTS: usize = 8;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq, InitSpace)]
pub enum MandateState {
    Active,
    Paused,
    Revoked,
}

#[account]
#[derive(InitSpace)]
pub struct Mandate {
    /// The only key that can withdraw, unpause or amend.
    pub owner: Pubkey,
    /// The house agent. Propose-only.
    pub operator: Pubkey,
    /// Pause and revoke only. Never unpause, never withdraw.
    pub emergency: Pubkey,
    /// `BOOK_ONE` for the house book.
    pub strategy_id: [u8; 16],
    pub state: MandateState,
    pub policy: Policy,
    /// Token account owned by this PDA.
    pub vault: Pubkey,
    /// Settlement mint (USDC-d).
    pub mint: Pubkey,
    /// The mark account the policy binds this mandate to (a Pyth
    /// `PriceUpdateV2`); the freshness gate refuses any other account.
    pub mark_account: Pubkey,
    /// SOL/USD feed id the mark must carry.
    pub feed_id: [u8; 32],
    /// UTC day index (`unix_timestamp.div_euclid(86_400)`) for the rolling counters.
    pub day_epoch: i64,
    pub day_notional_used: u64,
    pub day_spend_used: u64,
    /// Monotonic; goes on every receipt.
    pub action_seq: u64,
    /// Ring of recent intent ids for replay protection; cleared on day rollover.
    pub recent_intents: [[u8; 32]; RECENT_INTENTS],
    pub recent_intents_len: u8,
    pub recent_intents_next: u8,
    pub created_at: i64,
    /// PDA seed nonce, so one owner can hold several mandates for one strategy.
    pub nonce: u64,
    pub bump: u8,
    pub vault_bump: u8,
    /// Room for the Phase-1 fields (`max_net_delta_usd`, `max_gross_usd` on
    /// chain) without a migration (docs/10 §1.1).
    pub reserve: [u8; 128],
}

pub const SECONDS_PER_UTC_DAY: i64 = 86_400;

impl Mandate {
    pub const SEED: &'static [u8] = b"mandate";
    pub const VAULT_SEED: &'static [u8] = b"vault";

    pub fn utc_day(ts: i64) -> i64 {
        ts.div_euclid(SECONDS_PER_UTC_DAY)
    }

    /// Roll the daily counters and clear the replay ring when the UTC day
    /// changes. Called on the execute path before any gate reads a counter.
    pub fn rollover(&mut self, now: i64) {
        let day = Self::utc_day(now);
        if day != self.day_epoch {
            self.day_epoch = day;
            self.day_notional_used = 0;
            self.day_spend_used = 0;
            self.recent_intents_len = 0;
            self.recent_intents_next = 0;
        }
    }

    pub fn is_owner(&self, k: &Pubkey) -> bool {
        self.owner == *k
    }
    pub fn is_operator(&self, k: &Pubkey) -> bool {
        self.operator == *k
    }
    pub fn is_emergency(&self, k: &Pubkey) -> bool {
        self.emergency != Pubkey::default() && self.emergency == *k
    }

    pub fn has_recent_intent(&self, id: &[u8; 32]) -> bool {
        self.recent_intents
            .iter()
            .take(self.recent_intents_len.min(RECENT_INTENTS as u8) as usize)
            .any(|x| x == id)
    }

    pub fn remember_intent(&mut self, id: [u8; 32]) {
        let idx = self.recent_intents_next as usize % RECENT_INTENTS;
        self.recent_intents[idx] = id;
        self.recent_intents_next = ((idx + 1) % RECENT_INTENTS) as u8;
        if (self.recent_intents_len as usize) < RECENT_INTENTS {
            self.recent_intents_len += 1;
        }
    }
}
