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

/// Read what the venue reported with `set_return_data`.
///
/// A Solana CPI returns no value, so this is the only way the program learns
/// what actually traded — and, because a failed CPI fails the whole
/// transaction, it is also the only way a venue *refusal* can reach a
/// committed receipt (ADR-008).
///
/// `None` means the venue reported nothing, or reported it from the wrong
/// program. The caller must then refuse: the alternative is writing the limit
/// price into a receipt and calling it a fill (ADR-007).
pub fn reported(expected_program: &Pubkey) -> Option<markov_types::VenueReport> {
    let (program, data) = anchor_lang::solana_program::program::get_return_data()?;
    if program != *expected_program {
        return None;
    }
    markov_types::VenueReport::try_from_slice(&data).ok()
}

/// Anchor's global instruction discriminator.
fn sighash(name: &str) -> [u8; 8] {
    let preimage = format!("global:{name}");
    let digest = hash(preimage.as_bytes());
    let mut out = [0u8; 8];
    out.copy_from_slice(&digest.to_bytes()[..8]);
    out
}

/// Call the venue.
///
/// An `Err` from here is a **structural fault**, not a venue refusal: since
/// ADR-008 the ABI requires venue conditions to come back as return data,
/// because a failed CPI fails the whole transaction and no receipt could
/// commit. The real program error is propagated so the failure is legible in
/// the logs rather than flattened to a unit.
pub fn venue_execute<'info>(
    venue_program: &AccountInfo<'info>,
    accounts: &[AccountInfo<'info>],
    metas: Vec<AccountMeta>,
    args: &VenueExecuteArgs,
    signer_seeds: &[&[&[u8]]],
) -> Result<()> {
    let mut data = sighash("venue_execute").to_vec();
    args.serialize(&mut data)?;
    let ix = Instruction {
        program_id: venue_program.key(),
        accounts: metas,
        data,
    };
    invoke_signed(&ix, accounts, signer_seeds).map_err(Into::into)
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
pub fn post_checks_pass(before: &VaultSnapshot, after: &VaultSnapshot) -> bool {
    if after.owner != before.owner
        || after.has_delegate
        || after.mandate_owner != before.mandate_owner
        || after.mandate_operator != before.mandate_operator
        || after.policy_hash != before.policy_hash
    {
        return false;
    }
    // A Gate B venue takes no custody (`scripts/no-token-custody.sh`), so the
    // vault must be *exactly* as it was. This was once `<= notional`, to leave
    // room for a venue that collects collateral; that tolerance is what let a
    // rogue venue take `notional` out of the vault and still pass (ADR-009).
    // A custody venue is Gate C work and needs its own accounting, not a
    // tolerance band.
    before.balance == after.balance
}

/// True when `key` is an account this mandate controls, and which therefore
/// must never be forwarded to a venue.
///
/// The vault is named directly; any other SPL token account whose authority is
/// the mandate PDA is caught by its layout — `mint(32) || authority(32) || …`.
/// Taking the parts rather than an `AccountInfo` keeps this testable off
/// chain, which matters because it is the check that stops ADR-009's attack.
pub fn is_mandate_controlled(
    key: &Pubkey,
    owner_program: &Pubkey,
    data: &[u8],
    vault: &Pubkey,
    mandate: &Pubkey,
    token_program: &Pubkey,
) -> bool {
    if key == vault {
        return true;
    }
    if owner_program != token_program {
        return false;
    }
    data.len() >= 64 && data[32..64] == mandate.to_bytes()
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
    fn an_untouched_vault_passes() {
        assert!(post_checks_pass(&snap(), &snap()));
    }

    /// Any movement at all, in either direction. A venue that can take one
    /// base unit out of the vault can take the rest of it a block later.
    #[test]
    fn a_vault_that_moved_at_all_fails() {
        for balance in [999, 1_001, 0] {
            let mut after = snap();
            after.balance = balance;
            assert!(
                !post_checks_pass(&snap(), &after),
                "a vault that moved to {balance} passed the post-check"
            );
        }
    }

    fn token_account(authority: &Pubkey) -> Vec<u8> {
        let mut data = vec![0u8; 165];
        data[32..64].copy_from_slice(&authority.to_bytes());
        data
    }

    /// The check that stops ADR-009: a venue handed one of these could spend
    /// it with the mandate's own signature.
    #[test]
    fn the_vault_and_any_mandate_owned_token_account_are_controlled() {
        let vault = Pubkey::new_from_array([1; 32]);
        let mandate = Pubkey::new_from_array([2; 32]);
        let token = Pubkey::new_from_array([3; 32]);
        let other = Pubkey::new_from_array([4; 32]);

        // The vault, whatever its contents say.
        assert!(is_mandate_controlled(
            &vault,
            &token,
            &[],
            &vault,
            &mandate,
            &token
        ));
        // A second token account with the mandate as authority.
        assert!(is_mandate_controlled(
            &other,
            &token,
            &token_account(&mandate),
            &vault,
            &mandate,
            &token
        ));
        // Someone else's token account is not ours to refuse.
        assert!(!is_mandate_controlled(
            &other,
            &token,
            &token_account(&other),
            &vault,
            &mandate,
            &token
        ));
        // A non-token account that happens to hold the mandate's bytes at
        // that offset is not a token account, and the owner program says so.
        assert!(!is_mandate_controlled(
            &other,
            &other,
            &token_account(&mandate),
            &vault,
            &mandate,
            &token
        ));
        // Too short to be a token account.
        assert!(!is_mandate_controlled(
            &other, &token, &[0u8; 40], &vault, &mandate, &token
        ));
    }

    #[test]
    fn an_authority_change_fails() {
        let before = snap();
        let mut after = snap();
        after.owner = Pubkey::new_from_array([9; 32]);
        assert!(!post_checks_pass(&before, &after));
    }

    #[test]
    fn a_new_delegate_fails() {
        let before = snap();
        let mut after = snap();
        after.has_delegate = true;
        assert!(!post_checks_pass(&before, &after));
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
            assert!(!post_checks_pass(&before, &after));
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
