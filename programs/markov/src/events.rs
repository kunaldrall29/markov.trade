use crate::ids::*;
use anchor_lang::prelude::*;

#[event]
pub struct ReceiptEmitted {
    pub account: Pubkey,
    pub request_id: u128,
    pub actor: Pubkey,
    pub kind: ActionKind,
    pub decision: Decision,
    pub reason_code: u16,
    pub mandate_version: u32,
    pub invest_mandate_version: u32,
    pub data_slot: u64,
    pub venue_id: u8,
    pub market_id: [u8; 16],
    pub checks: Vec<Check>,
    pub route_snapshot_hash: [u8; 32],
    pub created_slot: u64,
}
