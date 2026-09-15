//! Reason codes are frozen in `docs/FACTS.md` §8.

use anchor_lang::prelude::*;

#[error_code]
pub enum MarkovError {
    #[msg("unauthorized")]
    Unauthorized = 6000,
    #[msg("account paused")]
    AccountPaused,
    #[msg("global paused")]
    GlobalPaused,
    #[msg("replay: request_id already used")]
    Replay,
    #[msg("mandate version mismatch")]
    MandateVersion,
    #[msg("observed values violate mandate for an Allow")]
    AllowWouldViolate,
    #[msg("missing or disallowed adjacent venue instruction")]
    AdjacentVenue,
    #[msg("receipt is append-only")]
    ReceiptImmutable,
    #[msg("receipt too young to close")]
    ReceiptTooYoung,
    #[msg("arithmetic overflow")]
    Overflow,
    #[msg("invalid caps")]
    InvalidCaps,
    #[msg("too many markets")]
    TooManyMarkets,
    #[msg("too many mints")]
    TooManyMints,
    #[msg("permission revoked or expired")]
    PermissionDenied,
    #[msg("owner-only instruction")]
    OwnerOnly,
    #[msg("config already initialized")]
    ConfigExists,
    #[msg("invest mandate paused")]
    InvestPaused,
    #[msg("budget or ceiling exceeded")]
    InvestBudget,
    #[msg("asset not on the invest allowlist")]
    AssetNotAllowlisted,
    #[msg("cost above mandate limit")]
    CostLimit,
    #[msg("wrong remaining accounts")]
    BadAccounts,
}
