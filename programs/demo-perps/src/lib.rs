//! `demo_perps` — the mock venue behind the real adapter trait.
//!
//! It is a mock, not a toy: it exists so the interface, the freshness gate and
//! the receipt shape are exercised for real. **It holds no token custody** —
//! collateral never moves here, so a bug in the mock cannot touch a vault.
//!
//! P02 ships only the entry point the mandate CPIs into, so the mandate's
//! allowed path is exercisable end to end. P04 gives it markets, positions,
//! deterministic fills at `mark ± fee_bps` and the `StaleMark` rejection.
#![allow(unexpected_cfgs)]
#![forbid(unsafe_code)]

use anchor_lang::prelude::*;

declare_id!("3Zcd8XsFWBTVku5GxQjwEBC7sLrJhF8vadyTnTr56hxB");

/// Mirrors `markov_mandate::cpi::venue::VenueExecuteArgs`. P03 makes this one
/// shared definition; until then the two must be kept identical, which the
/// mandate's `sighash` test and this program's discriminator pin down.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct VenueExecuteArgs {
    pub action: u8,
    pub market: [u8; 16],
    pub side: u8,
    pub notional: u64,
    pub limit_price: u64,
}

#[program]
pub mod demo_perps {
    use super::*;

    /// The adapter entry point. The **mandate PDA** must sign: the operator
    /// key never reaches a venue directly.
    ///
    /// P02 scope: accept a well-formed call from a mandate and record it in
    /// the log. No position is opened, no token moves, nothing is filled —
    /// P04 adds all of that, and until then no page may claim a fill.
    pub fn venue_execute(ctx: Context<VenueExecute>, args: VenueExecuteArgs) -> Result<()> {
        require!(args.notional > 0, DemoPerpsError::InvalidNotional);
        require!(args.limit_price > 0, DemoPerpsError::InvalidLimitPrice);
        msg!(
            "demo_perps: accepted action={} notional={} from mandate {} (P02 stub: no position taken)",
            args.action,
            args.notional,
            ctx.accounts.mandate.key()
        );
        Ok(())
    }
}

#[derive(Accounts)]
pub struct VenueExecute<'info> {
    /// The mandate PDA, signing through `invoke_signed`.
    pub mandate: Signer<'info>,
}

#[error_code]
pub enum DemoPerpsError {
    #[msg("notional must be greater than zero")]
    InvalidNotional,
    #[msg("limit price must be greater than zero")]
    InvalidLimitPrice,
}
