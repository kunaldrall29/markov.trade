//! The receipts. Every allow and every block that reaches the program emits
//! one, and a refusal commits (`Ok(())`) so the log is durable.
//!
//! Emitted with Anchor's CPI-event mechanism (`emit_cpi!`), so the payload
//! lands in a self-CPI's instruction data and survives log truncation, and the
//! indexer decodes it from the IDL rather than by string matching.

use anchor_lang::prelude::*;
use markov_types::BlockReason;

#[event]
pub struct ActionReceipt {
    pub seq: u64,
    pub intent_id: [u8; 32],
    pub mandate: Pubkey,
    pub owner: Pubkey,
    pub operator: Pubkey,
    pub strategy_id: [u8; 16],
    pub venue: Pubkey,
    pub market: [u8; 16],
    pub action: u8,
    pub side: u8,
    pub notional: u64,
    /// The price the venue reported filling at. Never a limit, never a mark:
    /// if the venue reports no fill, no `ActionReceipt` is emitted at all.
    pub fill_price: u64,
    /// The venue's fee on this fill, as the venue reported it.
    pub fee: u64,
    pub mark_price: u64,
    pub mark_publish_time: i64,
    pub spend: u64,
    pub forced: bool,
    pub ts: i64,
    pub slot: u64,
    /// Metadata; enforced off-chain in v0 (ADR-05). The page must label these.
    pub net_delta_usd_e6: i64,
    pub gross_usd_e6: u64,
}

#[event]
pub struct RefusalReceipt {
    pub seq: u64,
    pub intent_id: [u8; 32],
    pub mandate: Pubkey,
    pub operator: Pubkey,
    pub strategy_id: [u8; 16],
    pub venue: Pubkey,
    pub action: u8,
    pub notional: u64,
    /// `BlockReason` discriminant; 0–10 are the eleven the predecessor emitted.
    pub reason: BlockReason,
    /// Which rung of the ladder refused, 1-based, matching docs/10 §3.
    pub gate_index: u8,
    pub forced: bool,
    pub ts: i64,
    pub slot: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum OwnerActionKind {
    Create,
    Fund,
    Amend,
    Pause,
    Unpause,
    Revoke,
    Withdraw,
    Close,
}

/// The owner's own moves, so the feed can show them next to the agent's.
/// B6's proof is a pair — revoke, then the next attempt refused — and both
/// halves have to be indexable.
#[event]
pub struct OwnerAction {
    pub kind: OwnerActionKind,
    pub mandate: Pubkey,
    pub owner: Pubkey,
    /// Who signed. For pause/revoke this may be the emergency key.
    pub actor: Pubkey,
    pub strategy_id: [u8; 16],
    pub mint: Pubkey,
    pub amount: u64,
    pub ts: i64,
    pub slot: u64,
}
