use anchor_lang::prelude::*;

#[error_code]
pub enum MarkovError {
    #[msg("global config: caller is not the admin")]
    NotAdmin,
    #[msg("account: caller is not the owner")]
    NotOwner,
    #[msg("account is closed")]
    AccountClosed,
    #[msg("mandate version must be exactly active_version + 1")]
    BadMandateVersion,
    #[msg("mandate field out of range")]
    InvalidMandate,
    #[msg("too many approved markets (max 32)")]
    TooManyMarkets,
    #[msg("too many allowlisted mints (max 16)")]
    TooManyMints,
    #[msg("too many checks on a receipt (max 12)")]
    TooManyChecks,
    #[msg("permission: actor has no grant for this account")]
    NoPermission,
    #[msg("permission: revoked")]
    PermissionRevoked,
    #[msg("permission: expired")]
    PermissionExpired,
    #[msg("permission: scope not granted")]
    ScopeDenied,
    #[msg("permission: per-action cap exceeded")]
    ActorCapExceeded,
    #[msg("permission: daily cap exceeded")]
    ActorDailyCapExceeded,
    #[msg("an Allow decision was supplied whose observed values violate the active mandate")]
    AllowViolatesMandate,
    #[msg("an Allow decision was supplied while the account or the program is paused")]
    AllowWhilePaused,
    #[msg("decision kind does not fit this instruction")]
    WrongDecisionKind,
    #[msg("instructions sysvar: this instruction is not where the flow requires it")]
    BadInstructionIndex,
    #[msg("instructions sysvar: the adjacent instruction is missing")]
    MissingAdjacentInstruction,
    #[msg("instructions sysvar: the adjacent instruction is not from an allowed program")]
    AdjacentProgramNotAllowed,
    #[msg("instructions sysvar: the adjacent instruction must not be this program")]
    AdjacentIsSelf,
    #[msg("a Reject or Skip receipt must not be followed by a venue instruction")]
    RejectFollowedByVenue,
    #[msg("venue id is not configured")]
    UnknownVenue,
    #[msg("receipt already finalized")]
    AlreadyFinalized,
    #[msg("receipt is not old enough to close (30 days)")]
    ReceiptTooYoung,
    #[msg("mandate not active for this account")]
    NoActiveMandate,
    #[msg("invest mandate not active for this account")]
    NoActiveInvestMandate,
    #[msg("supplied checks do not match what the program recomputed")]
    ChecksMismatch,
    #[msg("arithmetic overflow")]
    Math,
    #[msg("reserve token account does not belong to the owner or has the wrong mint")]
    WrongReserveAccount,
    #[msg("invest: execution mode requires an official price reference")]
    ReferenceRequired,
    #[msg("invest amount outside the per-execution bounds")]
    InvestAmountOutOfBounds,
    #[msg("day_epoch must be the clock's UTC day")]
    WrongDayEpoch,
}
