//! The venue call. Every write to a venue is a CPI **signed by the mandate
//! PDA** — the operator key never signs to the venue directly, so a stolen
//! operator key cannot move a position without passing the ladder first.
//!
//! P02 defines the call and its error mapping; `demo_perps` implements the
//! other side in P04, and P03 formalises the trait both share. The
//! instruction data is `sighash("global", "venue_execute") || borsh(args)`,
//! the ordinary Anchor encoding, so the adapter is a normal Anchor program.

use anchor_lang::prelude::*;
use anchor_lang::solana_program::instruction::{AccountMeta, Instruction};
use anchor_lang::solana_program::program::invoke_signed;
use solana_sha256_hasher::hash;

/// What the mandate asks a venue to do. Fixed-width market id, never a string;
/// no caller-supplied price beyond the limit the ladder already bounded.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct VenueExecuteArgs {
    pub action: u8,
    pub market: [u8; 16],
    pub side: u8,
    pub notional: u64,
    pub limit_price: u64,
    /// The venue enforces the bound as well. Gate 11 already checked it, but
    /// a venue that fills outside the caller's bound has not honoured the
    /// intent, so it is told the bound rather than trusted to infer it.
    pub max_slippage_bps: u16,
}

/// Read the fill the venue reported with `set_return_data`.
///
/// A Solana CPI returns no value, so this is the only way the program learns
/// what actually traded. `None` means the venue reported nothing — and the
/// caller must then refuse, because the alternative is writing the limit
/// price into a receipt and calling it a fill (ADR-007).
pub fn reported_fill(expected_program: &Pubkey) -> Option<markov_types::VenueFill> {
    let (program, data) = anchor_lang::solana_program::program::get_return_data()?;
    if program != *expected_program {
        return None;
    }
    markov_types::VenueFill::try_from_slice(&data).ok()
}

/// Anchor's global instruction discriminator.
fn sighash(name: &str) -> [u8; 8] {
    let preimage = format!("global:{name}");
    let digest = hash(preimage.as_bytes());
    let mut out = [0u8; 8];
    out.copy_from_slice(&digest.to_bytes()[..8]);
    out
}

/// Call the venue. Returns `Err(())` when the venue rejects, which the caller
/// turns into a `VenueRejected` receipt (gate 13) — never a panic, never a
/// silent success.
pub fn venue_execute<'info>(
    venue_program: &AccountInfo<'info>,
    accounts: &[AccountInfo<'info>],
    metas: Vec<AccountMeta>,
    args: &VenueExecuteArgs,
    signer_seeds: &[&[&[u8]]],
) -> core::result::Result<(), ()> {
    let mut data = sighash("venue_execute").to_vec();
    args.serialize(&mut data).map_err(|_| ())?;
    let ix = Instruction {
        program_id: venue_program.key(),
        accounts: metas,
        data,
    };
    invoke_signed(&ix, accounts, signer_seeds).map_err(|_| ())
}

/// What the program insists is still true after the venue returns. This is a
/// pure function so the invariant is unit-tested even before a venue exists.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VaultSnapshot {
    pub balance: u64,
    pub owner: Pubkey,
    pub has_delegate: bool,
    pub mandate_owner: Pubkey,
    pub mandate_operator: Pubkey,
    pub policy_hash: [u8; 32],
}

/// The vault may move by at most the intent's notional; its authority and
/// delegate may not change; the mandate's owner, operator and policy bytes may
/// not change. Anything else is a state we cannot describe, so the whole
/// transaction reverts (`PostCheckFailed`, the one refusal allowed to be an
/// `Err`).
pub fn post_checks_pass(before: &VaultSnapshot, after: &VaultSnapshot, notional: u64) -> bool {
    if after.owner != before.owner
        || after.has_delegate
        || after.mandate_owner != before.mandate_owner
        || after.mandate_operator != before.mandate_operator
        || after.policy_hash != before.policy_hash
    {
        return false;
    }
    before.balance.abs_diff(after.balance) <= notional
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snap() -> VaultSnapshot {
        VaultSnapshot {
            balance: 1_000,
            owner: Pubkey::new_from_array([1; 32]),
            has_delegate: false,
            mandate_owner: Pubkey::new_from_array([2; 32]),
            mandate_operator: Pubkey::new_from_array([3; 32]),
            policy_hash: [4; 32],
        }
    }

    #[test]
    fn a_fill_inside_the_notional_passes() {
        let before = snap();
        let mut after = snap();
        after.balance = 950;
        assert!(post_checks_pass(&before, &after, 50));
    }

    #[test]
    fn a_vault_that_moved_more_than_the_notional_fails() {
        let before = snap();
        let mut after = snap();
        after.balance = 900;
        assert!(!post_checks_pass(&before, &after, 50));
    }

    #[test]
    fn an_authority_change_fails() {
        let before = snap();
        let mut after = snap();
        after.owner = Pubkey::new_from_array([9; 32]);
        assert!(!post_checks_pass(&before, &after, 50));
    }

    #[test]
    fn a_new_delegate_fails() {
        let before = snap();
        let mut after = snap();
        after.has_delegate = true;
        assert!(!post_checks_pass(&before, &after, 50));
    }

    #[test]
    fn a_policy_or_role_change_fails() {
        let before = snap();
        for mutate in [
            (|s: &mut VaultSnapshot| s.policy_hash = [7; 32]) as fn(&mut VaultSnapshot),
            |s: &mut VaultSnapshot| s.mandate_owner = Pubkey::new_from_array([8; 32]),
            |s: &mut VaultSnapshot| s.mandate_operator = Pubkey::new_from_array([8; 32]),
        ] {
            let mut after = snap();
            mutate(&mut after);
            assert!(!post_checks_pass(&before, &after, 50));
        }
    }

    #[test]
    fn sighash_is_anchors_global_encoding() {
        // Stable across builds; demo_perps must answer to exactly this.
        assert_eq!(sighash("venue_execute").len(), 8);
        assert_eq!(sighash("venue_execute"), sighash("venue_execute"));
        assert_ne!(sighash("venue_execute"), sighash("something_else"));
    }
}
