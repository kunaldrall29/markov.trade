//! Receipts are also events, so an indexer never has to read PDAs.
use anchor_lang::prelude::*;

use crate::state::{Check, Decision, MarketId, ReceiptKind};

#[event]
pub struct ReceiptEmitted {
    pub account: Pubkey,
    pub request_id: u128,
    pub actor: Pubkey,
    pub kind: ReceiptKind,
    pub decision: Decision,
    pub reason_code: u16,
    pub mandate_version: u32,
    pub invest_mandate_version: u32,
    pub data_slot: u64,
    pub venue_id: u8,
    pub market_id: MarketId,
    pub checks: Vec<Check>,
    pub route_snapshot_hash: [u8; 32],
    pub pre_state_hash: [u8; 32],
    pub created_slot: u64,
    pub created_ts: i64,
}

#[event]
pub struct ReceiptFinalized {
    pub account: Pubkey,
    pub request_id: u128,
    pub post_state_hash: [u8; 32],
    pub tx_signature_hint: [u8; 32],
    pub slot: u64,
}
