//! `Registry`: one PDA, upgrade-authority controlled. It can only ever stop
//! things or shrink the adapter set. It cannot move funds, cannot unpause a
//! mandate, and cannot widen a mandate policy.

use anchor_lang::prelude::*;

pub const MAX_ADAPTERS: usize = 8;

#[account]
#[derive(InitSpace)]
pub struct Registry {
    /// Documented single key on devnet; the accepted risk in SECURITY.md.
    pub admin: Pubkey,
    /// When true, every `execute_venue_action` refuses with `GlobalHalt`.
    pub global_halt: bool,
    pub adapters: [Pubkey; MAX_ADAPTERS],
    pub adapters_len: u8,
    pub bump: u8,
}

impl Registry {
    pub const SEED: &'static [u8] = b"registry";

    pub fn adapter_allowed(&self, program_id: &Pubkey) -> bool {
        self.adapters
            .iter()
            .take(self.adapters_len.min(MAX_ADAPTERS as u8) as usize)
            .any(|a| a == program_id)
    }
}
