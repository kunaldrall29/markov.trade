//! Instructions-sysvar introspection, guarded against wrapping: the current
//! index is read from the sysvar, the adjacent instruction is loaded by
//! absolute index, and the adjacent program must never be this program.
use anchor_lang::prelude::*;
use solana_instructions_sysvar::{load_current_index_checked, load_instruction_at_checked};

use crate::errors::MarkovError;

/// The program id of instruction `current + offset`, or `None` past the end.
pub fn adjacent_program(sysvar: &AccountInfo, offset: i64) -> Result<Option<Pubkey>> {
    let current = i64::from(load_current_index_checked(sysvar)?);
    let target = current.checked_add(offset).ok_or(MarkovError::Math)?;
    if target < 0 {
        return Err(MarkovError::BadInstructionIndex.into());
    }
    match load_instruction_at_checked(target as usize, sysvar) {
        Ok(ix) => Ok(Some(ix.program_id)),
        Err(ProgramError::InvalidArgument) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

/// An Allow must be followed, at `current + 1`, by the venue's program.
pub fn require_next_is(sysvar: &AccountInfo, expected: &Pubkey) -> Result<()> {
    let next = adjacent_program(sysvar, 1)?.ok_or(MarkovError::MissingAdjacentInstruction)?;
    require_keys_neq!(next, crate::ID, MarkovError::AdjacentIsSelf);
    require_keys_eq!(next, *expected, MarkovError::AdjacentProgramNotAllowed);
    Ok(())
}

/// A Reject, RequireApproval or Skip must not be followed by any configured
/// venue or invest program: a refusal receipt cannot ride with an execution.
pub fn require_no_venue_next(sysvar: &AccountInfo, forbidden: &[Pubkey]) -> Result<()> {
    if let Some(next) = adjacent_program(sysvar, 1)? {
        require!(
            !forbidden
                .iter()
                .any(|p| *p != Pubkey::default() && *p == next),
            MarkovError::RejectFollowedByVenue
        );
    }
    Ok(())
}

/// `finalize_decision` runs after the venue leg: the previous instruction
/// must exist and must not be this program.
pub fn require_prev_is_not_self(sysvar: &AccountInfo) -> Result<()> {
    let prev = adjacent_program(sysvar, -1)?.ok_or(MarkovError::MissingAdjacentInstruction)?;
    require_keys_neq!(prev, crate::ID, MarkovError::AdjacentIsSelf);
    Ok(())
}
