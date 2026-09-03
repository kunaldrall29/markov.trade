//! Types shared by the programs and the services. Must compile for SBF and host.
//!
//! `BlockReason` is append-only. Discriminants 0–10 are exactly what the
//! deployed predecessor `5o8E…` emitted on devnet (decoded from 20 on-chain
//! `ActionRefused` payloads, `docs/FACTS.md`); 11–16 are appended for Gate B in
//! the order fixed by ADR-004. Never rename, reorder, or reuse a variant.
#![forbid(unsafe_code)]
#![warn(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

/// Machine-readable refusal reason. Discriminant = on-chain byte.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "anchor",
    derive(
        anchor_lang::AnchorSerialize,
        anchor_lang::AnchorDeserialize,
        anchor_lang::InitSpace
    )
)]
// The wire byte IS the explicit discriminant: 0-10 are exactly what the
// predecessor emitted on devnet, so the encoding must not renumber them.
#[cfg_attr(feature = "anchor", borsh(use_discriminant = true))]
pub enum BlockReason {
    OverTxCap = 0,
    OverDailyCap = 1,
    OverSpendCap = 2,
    OverSpendDailyCap = 3,
    ProgramNotAllowed = 4,
    TokenNotAllowed = 5,
    SlippageExceeded = 6,
    Expired = 7,
    Paused = 8,
    Revoked = 9,
    Unauthorized = 10,
    // appended in Gate B (ADR-004); order is final once first emitted
    StaleOracle = 11,
    ActionNotAllowed = 12,
    DuplicateIntent = 13,
    GlobalHalt = 14,
    VenueRejected = 15,
    PostCheckFailed = 16,
}

impl BlockReason {
    /// Every variant, in discriminant order. The append-only test reads this.
    pub const ALL: [BlockReason; 17] = [
        BlockReason::OverTxCap,
        BlockReason::OverDailyCap,
        BlockReason::OverSpendCap,
        BlockReason::OverSpendDailyCap,
        BlockReason::ProgramNotAllowed,
        BlockReason::TokenNotAllowed,
        BlockReason::SlippageExceeded,
        BlockReason::Expired,
        BlockReason::Paused,
        BlockReason::Revoked,
        BlockReason::Unauthorized,
        BlockReason::StaleOracle,
        BlockReason::ActionNotAllowed,
        BlockReason::DuplicateIntent,
        BlockReason::GlobalHalt,
        BlockReason::VenueRejected,
        BlockReason::PostCheckFailed,
    ];

    /// The name exactly as it appears on a receipt, in mono, verbatim.
    pub const fn name(self) -> &'static str {
        match self {
            BlockReason::OverTxCap => "OverTxCap",
            BlockReason::OverDailyCap => "OverDailyCap",
            BlockReason::OverSpendCap => "OverSpendCap",
            BlockReason::OverSpendDailyCap => "OverSpendDailyCap",
            BlockReason::ProgramNotAllowed => "ProgramNotAllowed",
            BlockReason::TokenNotAllowed => "TokenNotAllowed",
            BlockReason::SlippageExceeded => "SlippageExceeded",
            BlockReason::Expired => "Expired",
            BlockReason::Paused => "Paused",
            BlockReason::Revoked => "Revoked",
            BlockReason::Unauthorized => "Unauthorized",
            BlockReason::StaleOracle => "StaleOracle",
            BlockReason::ActionNotAllowed => "ActionNotAllowed",
            BlockReason::DuplicateIntent => "DuplicateIntent",
            BlockReason::GlobalHalt => "GlobalHalt",
            BlockReason::VenueRejected => "VenueRejected",
            BlockReason::PostCheckFailed => "PostCheckFailed",
        }
    }

    /// Decode an on-chain byte. Unknown bytes are `None`, never a guess.
    pub const fn from_u8(b: u8) -> Option<BlockReason> {
        if (b as usize) < BlockReason::ALL.len() {
            Some(BlockReason::ALL[b as usize])
        } else {
            None
        }
    }
}

/// What the book may do in one tick. `Skip` is the default everywhere.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "anchor",
    derive(
        anchor_lang::AnchorSerialize,
        anchor_lang::AnchorDeserialize,
        anchor_lang::InitSpace
    )
)]
// The wire byte IS the explicit discriminant: 0-10 are exactly what the
// predecessor emitted on devnet, so the encoding must not renumber them.
#[cfg_attr(feature = "anchor", borsh(use_discriminant = true))]
pub enum ActionKind {
    Skip = 0,
    Open = 1,
    Increase = 2,
    Reduce = 3,
    Close = 4,
    Flatten = 5,
}

/// Which way a position leans. Shared so the agent, the program and the
/// indexer spell it the same way.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "anchor",
    derive(
        anchor_lang::AnchorSerialize,
        anchor_lang::AnchorDeserialize,
        anchor_lang::InitSpace
    )
)]
// The wire byte IS the explicit discriminant: 0-10 are exactly what the
// predecessor emitted on devnet, so the encoding must not renumber them.
#[cfg_attr(feature = "anchor", borsh(use_discriminant = true))]
pub enum Side {
    Long = 0,
    Short = 1,
}

impl Side {
    pub const fn name(self) -> &'static str {
        match self {
            Side::Long => "long",
            Side::Short => "short",
        }
    }
}

impl ActionKind {
    pub const fn name(self) -> &'static str {
        match self {
            ActionKind::Skip => "skip",
            ActionKind::Open => "open",
            ActionKind::Increase => "increase",
            ActionKind::Reduce => "reduce",
            ActionKind::Close => "close",
            ActionKind::Flatten => "flatten",
        }
    }
}

/// What a venue write actually did, as it crosses the CPI boundary.
///
/// A Solana CPI cannot return a value, so a venue reports its fill with
/// `set_return_data` and the caller reads it with `get_return_data`. This is
/// the only way the mandate program learns a real fill price. If it is
/// missing, the program refuses rather than filling the receipt in with the
/// limit price — a receipt that states a price nobody traded at is a lie with
/// a signature on it (ADR-007).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "anchor",
    derive(anchor_lang::AnchorSerialize, anchor_lang::AnchorDeserialize, anchor_lang::InitSpace)
)]
pub struct VenueFill {
    /// Scaled 1e6 per unit.
    pub price: u64,
    /// Settlement-mint base units actually transacted.
    pub notional: u64,
    pub fee: u64,
}

/// Where a mark came from. On chain, so the page can say it rather than guess.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "anchor",
    derive(anchor_lang::AnchorSerialize, anchor_lang::AnchorDeserialize, anchor_lang::InitSpace),
    borsh(use_discriminant = true)
)]
pub enum MarkSourceKind {
    /// Relayed from a Pyth `PriceUpdateV2` account, verified on chain.
    Pyth = 0,
    /// Posted by the house mark-poster. Devnet only, and the page must say so.
    House = 1,
}

impl MarkSourceKind {
    pub const fn name(self) -> &'static str {
        match self {
            MarkSourceKind::Pyth => "pyth",
            MarkSourceKind::House => "house",
        }
    }
}

#[cfg(test)]
mod venue_fill_tests {
    use super::*;

    #[test]
    fn mark_source_names_are_what_the_api_publishes() {
        assert_eq!(MarkSourceKind::Pyth as u8, 0);
        assert_eq!(MarkSourceKind::House as u8, 1);
        assert_eq!(MarkSourceKind::Pyth.name(), "pyth");
        assert_eq!(MarkSourceKind::House.name(), "house");
    }
}

#[cfg(test)]
mod side_tests {
    use super::*;

    #[test]
    fn side_and_action_names_are_stable() {
        assert_eq!(Side::Long as u8, 0);
        assert_eq!(Side::Short as u8, 1);
        assert_eq!(ActionKind::Skip as u8, 0);
        assert_eq!(ActionKind::Flatten as u8, 5);
        assert_eq!(ActionKind::Open.name(), "open");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discriminants_match_the_chain() {
        // The eleven emitted by 5o8E…, in the on-chain order (docs/FACTS.md).
        let chain = [
            (0u8, "OverTxCap"),
            (1, "OverDailyCap"),
            (2, "OverSpendCap"),
            (3, "OverSpendDailyCap"),
            (4, "ProgramNotAllowed"),
            (5, "TokenNotAllowed"),
            (6, "SlippageExceeded"),
            (7, "Expired"),
            (8, "Paused"),
            (9, "Revoked"),
            (10, "Unauthorized"),
        ];
        for (b, name) in chain {
            let r = BlockReason::from_u8(b);
            assert_eq!(r.map(|r| r.name()), Some(name));
            assert_eq!(r.map(|r| r as u8), Some(b));
        }
    }

    #[test]
    fn all_is_in_discriminant_order_and_dense() {
        for (i, r) in BlockReason::ALL.iter().enumerate() {
            assert_eq!(*r as usize, i);
        }
        assert_eq!(BlockReason::from_u8(17), None);
    }
}
