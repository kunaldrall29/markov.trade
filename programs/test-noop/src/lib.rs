//! TEST ONLY. One instruction, no accounts, no effect. It exists so the
//! control-plane program's instructions-sysvar tests can place a real,
//! executable, non-builtin program at *i+1* and *i+2*. Never deployed.
#![allow(unexpected_cfgs)]
#![forbid(unsafe_code)]

use anchor_lang::prelude::*;

declare_id!("GNu1ontKSAYN81UMqPZPiWMnRQ4RDAytjopHskwpQZDB");

#[program]
pub mod test_noop {
    use super::*;

    pub fn noop(_ctx: Context<Noop>, tag: u8) -> Result<()> {
        msg!("test_noop: {}", tag);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Noop {}
