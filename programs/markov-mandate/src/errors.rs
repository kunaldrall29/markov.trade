//! Hard errors. These are **not** `BlockReason`s.
//!
//! A `BlockReason` is a refusal the program records and commits: the gate
//! ladder emits a `RefusalReceipt` and returns `Ok(())` so the receipt is
//! durable. A `MandateError` is a state we cannot describe — a bad account, a
//! widening amendment, a broken invariant after a CPI — and it reverts.

use anchor_lang::prelude::*;

#[error_code]
pub enum MandateError {
    #[msg("only the owner may do this")]
    NotOwner,
    #[msg("only the owner or the emergency key may do this")]
    NotOwnerOrEmergency,
    #[msg("only the registry admin may do this")]
    NotRegistryAdmin,
    #[msg("mandate is revoked; this instruction is not legal in a terminal state")]
    AlreadyRevoked,
    #[msg("mandate is not paused")]
    NotPaused,
    #[msg("mandate is not active")]
    NotActive,
    #[msg("policy was not tightened: every cap may only shrink, every allowlist may only shrink, expiry may only shorten")]
    PolicyNotTightened,
    #[msg("policy is invalid")]
    InvalidPolicy,
    #[msg("amount must be greater than zero")]
    InvalidAmount,
    #[msg("vault must be empty before the mandate can be closed")]
    VaultNotEmpty,
    #[msg("arithmetic overflow")]
    Math,
    #[msg("wrong mint for this mandate")]
    WrongMint,
    #[msg("wrong vault for this mandate")]
    WrongVault,
    #[msg("post-check failed after the venue call: the vault, the owner, the operator or the policy moved, or the venue reported filling more than was asked")]
    PostCheckFailed,
}
