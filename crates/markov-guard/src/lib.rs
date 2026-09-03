//! The pure risk guard: `(Intent, GuardState, PolicyView) -> Verdict`.
//!
//! Plain words, the whole guard:
//!
//! The guard is one function. Every tick, the core proposes something and the
//! guard decides whether it may happen. It is handed everything it needs — the
//! time, the mark and when the mark was published, the current positions, the
//! day's counters, the policy's limits — and it reads nothing for itself. It
//! has no clock, no socket, no log, and no memory of the last tick.
//!
//! It answers one of three ways. **Skip** means the core proposed nothing and
//! nothing was wrong. **Allow** means every rule passed. **Veto** means one
//! rule failed, and the answer names which.
//!
//! The rules run in a fixed order that mirrors the program's own ladder, so
//! that if the guard and the chain ever disagree, the disagreement is visible
//! rather than silent: is the mandate live, is the mark fresh, is this action
//! permitted, is it too big for one trade, too big for the day, too much
//! spend, does it push the book past its delta band or its gross ceiling, is
//! the price too far from the mark, and has the day already lost enough to
//! stop.
//!
//! Three of those — the delta band, the gross ceiling and the daily-loss halt
//! — are enforced **here and nowhere else** in v0. The type says so, so that
//! no surface can call them "enforced" without the qualifier.
//!
//! Anything missing, contradictory, or too large to compute is a veto. There
//! is no path that allows on a `None`.
#![forbid(unsafe_code)]
#![warn(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![no_std]

pub use markov_types::{ActionKind, BlockReason, Side};

/// Where a veto is enforced. The API and the dashboard read this rather than
/// guessing from the name, because calling an off-chain rule "enforced"
/// without the qualifier is a B15 failure (`docs/11` §4).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Enforcement {
    /// The program refuses this too. The guard is saving a transaction, not
    /// providing the guarantee.
    OnChain,
    /// v0 has no on-chain counterpart (ADR-005). If the agent is bypassed,
    /// nothing enforces this.
    OffChainV0,
}

/// The reasons with no on-chain counterpart in v0.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OffChainReason {
    DeltaBandExceeded,
    GrossExceeded,
    DailyLossHalt,
    /// An input was missing, contradictory, or overflowed. The guard cannot
    /// describe the state, so it refuses to act in it.
    GuardInternal,
}

impl OffChainReason {
    pub const fn name(self) -> &'static str {
        match self {
            OffChainReason::DeltaBandExceeded => "DeltaBandExceeded",
            OffChainReason::GrossExceeded => "GrossExceeded",
            OffChainReason::DailyLossHalt => "DailyLossHalt",
            OffChainReason::GuardInternal => "GuardInternal",
        }
    }
}

/// Why the guard said no.
///
/// The variant *is* the enforcement tag: an on-chain reason is one the program
/// would also refuse, and an off-chain one is v0-agent-only. Nothing has to
/// keep a list in sync.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GuardReason {
    OnChain(BlockReason),
    OffChainV0(OffChainReason),
}

impl GuardReason {
    pub const fn enforcement(self) -> Enforcement {
        match self {
            GuardReason::OnChain(_) => Enforcement::OnChain,
            GuardReason::OffChainV0(_) => Enforcement::OffChainV0,
        }
    }

    /// The name exactly as it appears on a tick row and in the API.
    pub const fn name(self) -> &'static str {
        match self {
            GuardReason::OnChain(r) => r.name(),
            GuardReason::OffChainV0(r) => r.name(),
        }
    }

    pub const fn is_off_chain(self) -> bool {
        matches!(self, GuardReason::OffChainV0(_))
    }
}

/// The mandate's lifecycle, as the agent last read it. `Expired` is derived
/// from the policy's end date by the caller, not from a clock in here.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub enum MandateState {
    Active,
    Paused,
    Revoked,
    Expired,
}

/// What the core proposed for this tick.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Intent {
    pub action: ActionKind,
    pub side: Side,
    /// Notional in mint base units (0 for Skip).
    pub notional: u64,
    /// The worst price the intent will accept, scaled 1e6.
    pub limit_price_e6: u64,
    /// Settlement-mint base units this action may spend.
    pub spend: u64,
}

impl Intent {
    pub const SKIP: Intent = Intent {
        action: ActionKind::Skip,
        side: Side::Long,
        notional: 0,
        limit_price_e6: 0,
        spend: 0,
    };
}

/// Everything the guard is allowed to know, passed in — never read.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GuardState {
    pub now_unix: i64,
    pub slot: u64,
    pub state: MandateState,
    /// `None` means the mark could not be read or bound this tick.
    pub mark_e6: Option<u64>,
    /// When the venue or oracle says that mark was published.
    pub mark_publish_time: Option<i64>,
    /// Signed net exposure in settlement base units. Long is positive.
    pub net_delta: i128,
    /// Sum of absolute exposure in settlement base units.
    pub gross: u128,
    pub vault_balance: u64,
    pub day_notional_used: u64,
    pub day_spend_used: u64,
    /// Equity at the session's start, and now, in settlement base units.
    pub session_start_equity: u64,
    pub equity: u64,
}

/// The policy limits, as the agent last read them from the mandate.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyView {
    pub max_mark_age_secs: i64,
    /// Bit `n` set means `ActionKind` with discriminant `n` is allowed.
    pub allowed_actions: u8,
    pub per_tx_cap: u64,
    pub daily_cap: u64,
    pub spend_cap: u64,
    pub spend_daily_cap: u64,
    pub max_slippage_bps: u16,
    /// Ceiling on `|net_delta|` after the fill.
    pub delta_band: u128,
    /// Ceiling on gross exposure after the fill.
    pub max_gross: u128,
    /// Stop for the day once equity has fallen this far below the session's
    /// start. 500 = 5%.
    pub daily_loss_bps: u16,
}

impl PolicyView {
    /// Build an action mask from a slice of the kinds the policy allows.
    pub fn actions_mask(kinds: &[ActionKind]) -> u8 {
        let mut mask = 0u8;
        let mut i = 0;
        while i < kinds.len() {
            mask |= 1u8 << (kinds[i] as u8);
            i += 1;
        }
        mask
    }

    const fn allows(&self, action: ActionKind) -> bool {
        self.allowed_actions & (1u8 << (action as u8)) != 0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    Allow(Intent),
    Veto(GuardReason),
    Skip,
}

const fn on(r: BlockReason) -> Verdict {
    Verdict::Veto(GuardReason::OnChain(r))
}

const fn off(r: OffChainReason) -> Verdict {
    Verdict::Veto(GuardReason::OffChainV0(r))
}

const INTERNAL: Verdict = off(OffChainReason::GuardInternal);

/// Evaluate one tick.
///
/// The order below is the order in `docs/11-AGENT-SPEC.md` §4, which is the
/// order of the program's own ladder. Do not reorder it to make a test pass:
/// the point of the order is that a divergence from the chain shows up as the
/// *wrong reason*, which is visible, rather than as the same verdict reached
/// differently, which is not.
pub fn evaluate(intent: &Intent, s: &GuardState, p: &PolicyView) -> Verdict {
    // 1. Is the mandate live? Checked for every intent, including Skip, so a
    //    paused book says "paused" on its tape rather than going quiet.
    match s.state {
        MandateState::Paused => return on(BlockReason::Paused),
        MandateState::Revoked => return on(BlockReason::Revoked),
        MandateState::Expired => return on(BlockReason::Expired),
        MandateState::Active => {}
    }

    // 2. Is the mark fresh? Also checked for Skip, so a feed gap is reported
    //    as a feed gap instead of looking like a quiet market.
    let (Some(mark_e6), Some(published)) = (s.mark_e6, s.mark_publish_time) else {
        return on(BlockReason::StaleOracle);
    };
    let Some(age) = s.now_unix.checked_sub(published) else {
        return INTERNAL;
    };
    if age < 0 {
        // The mark is stamped in the future: this clock and that one disagree
        // and the guard cannot tell which is wrong. Not a stale feed — an
        // input it cannot reason about.
        return INTERNAL;
    }
    if age > p.max_mark_age_secs {
        return on(BlockReason::StaleOracle);
    }

    // Nothing proposed, and nothing wrong. This is the common case: the book
    // is supposed to be flat most of the time.
    if intent.action == ActionKind::Skip {
        return Verdict::Skip;
    }

    // 3. Is this action permitted at all?
    if !p.allows(intent.action) {
        return on(BlockReason::ActionNotAllowed);
    }

    // 4. Too big for one trade?
    if intent.notional > p.per_tx_cap {
        return on(BlockReason::OverTxCap);
    }

    // 5. Too big for the day?
    let Some(day_after) = s.day_notional_used.checked_add(intent.notional) else {
        return INTERNAL;
    };
    if day_after > p.daily_cap {
        return on(BlockReason::OverDailyCap);
    }

    // 6. Spend budgets, per action and per day.
    if intent.spend > p.spend_cap {
        return on(BlockReason::OverSpendCap);
    }
    let Some(spend_after) = s.day_spend_used.checked_add(intent.spend) else {
        return INTERNAL;
    };
    if spend_after > p.spend_daily_cap {
        return on(BlockReason::OverSpendDailyCap);
    }

    // 7. Would the fill push the book past its delta band? Off-chain only.
    let Some(delta_after) = projected_delta(intent, s) else {
        return INTERNAL;
    };
    if delta_after.unsigned_abs() > p.delta_band {
        return off(OffChainReason::DeltaBandExceeded);
    }

    // 8. Past its gross ceiling? Off-chain only.
    let Some(gross_after) = projected_gross(intent, s) else {
        return INTERNAL;
    };
    if gross_after > p.max_gross {
        return off(OffChainReason::GrossExceeded);
    }

    // 9. Is the limit further from the mark than the policy tolerates? The
    //    program checks this too, against the mark it binds itself.
    let Some(slip_bps) = slippage_bps(intent.limit_price_e6, mark_e6) else {
        return INTERNAL;
    };
    if slip_bps > u128::from(p.max_slippage_bps) {
        return on(BlockReason::SlippageExceeded);
    }

    // 10. Has the day already lost enough to stop? Off-chain only, and the
    //     last rule, so the tape shows what would otherwise have been allowed.
    if daily_loss_breached(s, p.daily_loss_bps) {
        return off(OffChainReason::DailyLossHalt);
    }

    Verdict::Allow(*intent)
}

/// Signed exposure after the fill. Opening and increasing add exposure on the
/// intent's side; reducing, closing and flattening move it toward zero.
fn projected_delta(intent: &Intent, s: &GuardState) -> Option<i128> {
    let size = i128::from(intent.notional);
    let signed = match intent.side {
        Side::Long => size,
        Side::Short => size.checked_neg()?,
    };
    match intent.action {
        ActionKind::Open | ActionKind::Increase => s.net_delta.checked_add(signed),
        ActionKind::Reduce => s.net_delta.checked_sub(signed),
        // A close or a flatten ends at zero exposure by definition; the venue
        // decides the size, so the guard does not pretend to know it.
        ActionKind::Close | ActionKind::Flatten => Some(0),
        ActionKind::Skip => Some(s.net_delta),
    }
}

/// Gross exposure after the fill.
fn projected_gross(intent: &Intent, s: &GuardState) -> Option<u128> {
    let size = u128::from(intent.notional);
    match intent.action {
        ActionKind::Open | ActionKind::Increase => s.gross.checked_add(size),
        ActionKind::Reduce => Some(s.gross.saturating_sub(size)),
        ActionKind::Close | ActionKind::Flatten => Some(0),
        ActionKind::Skip => Some(s.gross),
    }
}

/// Distance from the mark, in basis points. A zero mark is not a price.
fn slippage_bps(limit_e6: u64, mark_e6: u64) -> Option<u128> {
    if mark_e6 == 0 {
        return None;
    }
    let diff = u128::from(limit_e6.abs_diff(mark_e6));
    diff.checked_mul(10_000)?.checked_div(u128::from(mark_e6))
}

/// True once equity has fallen `bps` below the session's start.
///
/// A session that started with nothing cannot lose a percentage of it, so the
/// rule does not fire; the caps above are what bound that book.
fn daily_loss_breached(s: &GuardState, bps: u16) -> bool {
    if s.session_start_equity == 0 || s.equity >= s.session_start_equity {
        return false;
    }
    let lost = u128::from(s.session_start_equity - s.equity);
    let limit = u128::from(s.session_start_equity)
        .saturating_mul(u128::from(bps))
        .saturating_div(10_000);
    lost >= limit
}
