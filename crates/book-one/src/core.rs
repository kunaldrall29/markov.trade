//! `book-core`: the deterministic proposer.
//!
//! One page, in plain words:
//!
//! The book wants to be flat. Its target net delta is zero, it tolerates a
//! band either side of that, and it will not hold more gross exposure than its
//! cap. Every tick it is handed the state of the book, a few features, and the
//! policy's limits, and it proposes **at most one action** — usually none.
//!
//! It stops first and asks questions later. If the sidecar says the regime is
//! `Halt`, or the day's loss stop is already active, the answer is `Flatten`
//! when there is anything to flatten and `Skip` when there is not. Nothing
//! below can override that: flatten always wins.
//!
//! Then it refuses to act on a price it does not trust — no mark, or a mark
//! older than the policy allows, and the answer is `Skip`. The guard would
//! veto anyway; proposing into a stale mark would just make the tape noisier.
//!
//! After that there are three ways to act. If net delta has drifted outside
//! the band, reduce the offending side by the smaller of the drift and one
//! clip. If the regime is `Trend` and gross is over half the cap, reduce by a
//! clip. If the regime is `Chop`, gross is under the cap, and funding is
//! favourable, add a clip. Anything else is `Skip`, which is the answer most
//! of the time and is supposed to be.
//!
//! **`funding_favourable` is a stub constant, `false`, in Gate B.** No venue
//! here exposes funding, so the third rule never fires and the book never adds
//! exposure on its own. That is a fact about this build, not a strategy claim,
//! and it must never appear on a page as if it were measured.
//!
//! Finally, hysteresis: if this tick proposes the same action as the last one
//! and the book has moved less than a quarter of a clip since, the answer is
//! `Skip` instead. A dashboard that flickers is a dashboard nobody trusts.
//! `Flatten` is exempt — a halt is not chatter.
//!
//! Pure: no clock, no randomness, no I/O. Same inputs, same intent, always.

use markov_guard::{ActionKind, Intent, PolicyView, Side};

use crate::sidecar::{Features, Regime};

/// The book as the agent last observed it, plus what it did on the previous
/// tick. Everything is passed in; the core reads nothing for itself.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct BookState {
    /// Signed net exposure in mint base units. Long is positive.
    pub net_delta: i128,
    /// Sum of absolute exposure in mint base units.
    pub gross: u128,
    /// Set by the runtime when the guard's daily-loss rule has fired today.
    pub daily_loss_halt_active: bool,
    /// What was proposed on the **previous tick**, and the net delta at that
    /// moment — `docs/11` §3 words hysteresis against the previous tick, so a
    /// `Skip` counts and resets it. `None`/zero at the start of a session.
    pub last_action: Option<ActionKind>,
    pub last_net_delta: i128,
}

/// The clip size: one trade's worth. Never more than the per-trade cap, and
/// never more than a quarter of the gross ceiling, so four clips fill the book
/// rather than one.
pub fn clip(policy: &PolicyView) -> u64 {
    let quarter = policy.max_gross / 4;
    let quarter = u64::try_from(quarter).unwrap_or(u64::MAX);
    policy.per_tx_cap.min(quarter)
}

/// The whole book. `docs/11-AGENT-SPEC.md` §3.
pub fn propose(state: &BookState, feats: &Features, policy: &PolicyView) -> Intent {
    let proposed = propose_before_hysteresis(state, feats, policy);

    // Flatten is never suppressed: a halt is not chatter.
    if proposed.action == ActionKind::Flatten || proposed.action == ActionKind::Skip {
        return proposed;
    }

    // Hysteresis, exactly as `docs/11` §3 words it: the same action, in the
    // previous tick, on a book that has moved less than a quarter of a clip.
    //
    // The literal rule does not catch everything its next line claims for it.
    // With funding enabled, rules 4 and 6 fight — rule 6 fills the book, the
    // drift pushes delta out of the band, rule 4 takes it off, rule 6 fills it
    // again — and because those actions *alternate*, a same-action test never
    // fires. The replay harness measures it: **40.7% Skip** with the funding
    // stub forced on (14.3% before adds were sized to the delta band), against
    // 100% as Gate B actually ships. Broadening the rule to
    // "any action after a recent action" trades that for the opposite failure,
    // a book that locks up permanently once it acts, because the threshold is
    // absolute while the drift is proportional to position size. Neither is
    // right, the condition is unreachable in Gate B (the stub is `false`), and
    // it is not a thing to redesign on the strength of a hypothetical. Recorded
    // in BACKLOG with the measured number, to be settled when a venue reports
    // real funding.
    if state.last_action == Some(proposed.action) {
        let moved = state.net_delta.abs_diff(state.last_net_delta);
        if moved < u128::from(clip(policy) / 4) {
            return Intent::SKIP;
        }
    }
    proposed
}

fn propose_before_hysteresis(state: &BookState, feats: &Features, policy: &PolicyView) -> Intent {
    // 1 and 2. Flatten always wins, and it is checked first so that it does.
    if feats.regime == Regime::Halt || state.daily_loss_halt_active {
        return if state.gross > 0 {
            flatten(state, feats, policy)
        } else {
            Intent::SKIP
        };
    }

    // 3. A price we do not trust is not a price to act on.
    let (Some(mark_e6), Some(age)) = (feats.mark_e6, feats.mark_age_secs) else {
        return Intent::SKIP;
    };
    if age < 0 || age > policy.max_mark_age_secs {
        return Intent::SKIP;
    }

    let c = u128::from(clip(policy));

    // 4. Delta outside the band: reduce the side that is too big.
    let drift = state.net_delta.unsigned_abs();
    if drift > policy.delta_band && state.gross > 0 {
        let size = drift.min(c);
        let side = if state.net_delta > 0 {
            Side::Long
        } else {
            Side::Short
        };
        return sized(ActionKind::Reduce, side, size, mark_e6, policy);
    }

    // 5. Trending, and more than half the cap is on: take some off.
    if feats.regime == Regime::Trend && state.gross > policy.max_gross / 2 {
        let side = net_side(state);
        return sized(
            ActionKind::Reduce,
            side,
            c.min(state.gross),
            mark_e6,
            policy,
        );
    }

    // 6. Chop, room left, and funding pays us to hold it. In Gate B
    //    `funding_favourable_stub` is a constant `false`, so this never fires.
    if feats.regime == Regime::Chop
        && state.gross < policy.max_gross
        && feats.funding_favourable_stub
    {
        // Add on the side that moves net delta toward zero, so the book gets
        // bigger without getting more directional.
        let side = match state.net_delta.signum() {
            1 => Side::Short,
            -1 => Side::Long,
            _ => Side::Long,
        };
        // Never propose what our own guard would veto: an add that pushes net
        // delta outside the band is a wasted transaction and a refusal on the
        // tape. Size it to land on the band's edge at worst, and skip if there
        // is no room at all.
        let toward = i128::from(policy.delta_band.min(u128::from(u64::MAX)) as u64);
        let headroom = match side {
            Side::Long => toward - state.net_delta,
            Side::Short => toward + state.net_delta,
        };
        if headroom <= 0 {
            return Intent::SKIP;
        }
        let headroom = headroom.unsigned_abs();
        let room = policy.max_gross - state.gross;
        let action = if state.gross == 0 {
            // There is nothing to add to yet, so this opens rather than
            // increases. The program distinguishes them; so do we.
            ActionKind::Open
        } else {
            ActionKind::Increase
        };
        return sized(action, side, c.min(room).min(headroom), mark_e6, policy);
    }

    // 7. The usual answer.
    Intent::SKIP
}

fn net_side(state: &BookState) -> Side {
    if state.net_delta < 0 {
        Side::Short
    } else {
        Side::Long
    }
}

fn flatten(state: &BookState, feats: &Features, policy: &PolicyView) -> Intent {
    // A flatten still needs a price to name a limit. Without one it is still
    // proposed, at the last limit the policy would tolerate around a zero
    // mark — which the guard will veto as a stale mark, and the tape will say
    // so. Silently skipping a flatten because the feed blinked is worse.
    let mark = feats.mark_e6.unwrap_or(0);
    let size = u64::try_from(state.gross).unwrap_or(u64::MAX);
    sized(
        ActionKind::Flatten,
        net_side(state),
        u128::from(size),
        mark,
        policy,
    )
}

/// Build the intent, with a limit half the policy's slippage allowance away
/// from the mark, in the direction that is worse for us.
///
/// Half, not all, so that a mark that moves a little between proposing and
/// landing does not put the intent outside the bound the guard checks.
fn sized(action: ActionKind, side: Side, size: u128, mark_e6: u64, policy: &PolicyView) -> Intent {
    let notional = u64::try_from(size).unwrap_or(u64::MAX);
    let bps = u64::from(policy.max_slippage_bps / 2);
    let offset = mark_e6.saturating_mul(bps) / 10_000;
    let buying = matches!(action, ActionKind::Open | ActionKind::Increase);
    // Buying a long or selling a short both accept a worse price the same way.
    let worse_is_up = match (buying, side) {
        (true, Side::Long) | (false, Side::Short) => true,
        (true, Side::Short) | (false, Side::Long) => false,
    };
    let limit_price_e6 = if worse_is_up {
        mark_e6.saturating_add(offset)
    } else {
        mark_e6.saturating_sub(offset)
    };
    Intent {
        action,
        side,
        notional,
        limit_price_e6,
        // Gate B's venue takes no token custody, so an action spends nothing.
        // A custody venue is Gate C work and will set this from its margin
        // requirement, not from a guess made here.
        spend: 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{
        GATE_B_DAILY_LOSS_BPS, GATE_B_DELTA_BAND, GATE_B_MAX_GROSS, GATE_B_MAX_SLIPPAGE_BPS,
        GATE_B_PER_TX_CAP,
    };

    const MARK_E6: u64 = 100_000_000; // $100

    fn policy() -> PolicyView {
        PolicyView {
            max_mark_age_secs: 150,
            allowed_actions: PolicyView::actions_mask(&[
                ActionKind::Open,
                ActionKind::Increase,
                ActionKind::Reduce,
                ActionKind::Close,
                ActionKind::Flatten,
            ]),
            per_tx_cap: GATE_B_PER_TX_CAP,
            daily_cap: GATE_B_PER_TX_CAP * 4,
            spend_cap: GATE_B_PER_TX_CAP,
            spend_daily_cap: GATE_B_PER_TX_CAP * 4,
            max_slippage_bps: GATE_B_MAX_SLIPPAGE_BPS,
            delta_band: GATE_B_DELTA_BAND,
            max_gross: GATE_B_MAX_GROSS,
            daily_loss_bps: GATE_B_DAILY_LOSS_BPS,
        }
    }

    fn feats(regime: Regime) -> Features {
        Features {
            regime,
            funding_favourable_stub: false,
            mark_e6: Some(MARK_E6),
            mark_age_secs: Some(5),
            mark_age_slots: Some(30),
        }
    }

    /// The book's answer, when nothing in particular is happening, is nothing.
    #[test]
    fn default_is_skip() {
        let s = BookState::default();
        assert_eq!(propose(&s, &feats(Regime::Chop), &policy()), Intent::SKIP);
        assert_eq!(propose(&s, &feats(Regime::Trend), &policy()), Intent::SKIP);
        // And with the stub constant as it actually is in Gate B — `false` —
        // the one rule that would add exposure cannot fire.
        assert!(!feats(Regime::Chop).funding_favourable_stub);
    }

    /// Same inputs, same intent. A thousand times, across every regime and a
    /// spread of book states, because "deterministic" is the property the
    /// whole tape rests on.
    #[test]
    fn deterministic_over_1000_runs() {
        let p = policy();
        let states = [
            BookState::default(),
            BookState {
                net_delta: 30 * i128::from(super::super::config::E6),
                gross: 40 * u128::from(super::super::config::E6),
                ..BookState::default()
            },
            BookState {
                net_delta: -(30 * i128::from(super::super::config::E6)),
                gross: 90 * u128::from(super::super::config::E6),
                daily_loss_halt_active: true,
                ..BookState::default()
            },
        ];
        for state in states {
            for regime in [Regime::Chop, Regime::Trend, Regime::Halt] {
                let f = feats(regime);
                let first = propose(&state, &f, &p);
                for i in 0..1_000 {
                    assert_eq!(
                        propose(&state, &f, &p),
                        first,
                        "run {i} differed for {regime:?}"
                    );
                }
            }
        }
    }

    /// A halt flattens, and nothing below it in the ladder can talk it out of
    /// that — including a state where the delta rule would also have fired.
    #[test]
    fn flatten_wins_on_halt() {
        let p = policy();
        let e6 = u128::from(crate::config::E6);
        let state = BookState {
            // Both the delta rule (drift past the band) and the trend rule
            // (gross over half the cap) would fire here.
            net_delta: 60 * i128::try_from(e6).expect("fits"),
            gross: 80 * e6,
            ..BookState::default()
        };
        let got = propose(&state, &feats(Regime::Halt), &p);
        assert_eq!(got.action, ActionKind::Flatten);
        assert_eq!(
            u128::from(got.notional),
            state.gross,
            "flatten the whole book"
        );

        // The daily-loss stop flattens for the same reason.
        let halted = BookState {
            daily_loss_halt_active: true,
            ..state
        };
        assert_eq!(
            propose(&halted, &feats(Regime::Chop), &p).action,
            ActionKind::Flatten
        );

        // Nothing to flatten is a Skip, not an empty order.
        let flat = BookState {
            gross: 0,
            net_delta: 0,
            ..BookState::default()
        };
        assert_eq!(propose(&flat, &feats(Regime::Halt), &p), Intent::SKIP);
    }

    /// The same action twice on a book that has barely moved is chatter, and
    /// the second one is suppressed. A book that *has* moved gets its action.
    #[test]
    fn hysteresis_prevents_chatter() {
        let p = policy();
        let e6 = i128::from(crate::config::E6);
        let drifted = BookState {
            net_delta: 30 * e6, // outside the ±$20 band
            gross: 40 * u128::try_from(e6).expect("fits"),
            ..BookState::default()
        };
        let first = propose(&drifted, &feats(Regime::Chop), &p);
        assert_eq!(
            first.action,
            ActionKind::Reduce,
            "the drift must be acted on"
        );

        // Same action proposed again, book unmoved since: suppressed.
        let unmoved = BookState {
            last_action: Some(ActionKind::Reduce),
            last_net_delta: drifted.net_delta,
            ..drifted
        };
        assert_eq!(
            propose(&unmoved, &feats(Regime::Chop), &p),
            Intent::SKIP,
            "a repeat action on an unmoved book is chatter"
        );

        // Moved more than a quarter of a clip: acted on again.
        let moved = BookState {
            last_action: Some(ActionKind::Reduce),
            last_net_delta: drifted.net_delta - (i128::from(clip(&p)) / 2),
            ..drifted
        };
        assert_eq!(
            propose(&moved, &feats(Regime::Chop), &p).action,
            ActionKind::Reduce,
            "a book that moved deserves the action"
        );

        // A halt is never suppressed, however recently it was proposed.
        let halting = BookState {
            last_action: Some(ActionKind::Flatten),
            last_net_delta: drifted.net_delta,
            ..drifted
        };
        assert_eq!(
            propose(&halting, &feats(Regime::Halt), &p).action,
            ActionKind::Flatten,
            "a halt is not chatter"
        );
    }

    /// A price we cannot trust produces no proposal at all.
    #[test]
    fn an_untrusted_mark_proposes_nothing() {
        let p = policy();
        let drifted = BookState {
            net_delta: 30 * i128::from(crate::config::E6),
            gross: 40 * u128::from(crate::config::E6),
            ..BookState::default()
        };
        for (what, f) in [
            (
                "no mark",
                Features {
                    mark_e6: None,
                    ..feats(Regime::Chop)
                },
            ),
            (
                "no age",
                Features {
                    mark_age_secs: None,
                    ..feats(Regime::Chop)
                },
            ),
            (
                "too old",
                Features {
                    mark_age_secs: Some(151),
                    ..feats(Regime::Chop)
                },
            ),
            (
                "stamped in the future",
                Features {
                    mark_age_secs: Some(-1),
                    ..feats(Regime::Chop)
                },
            ),
        ] {
            assert_eq!(propose(&drifted, &f, &p), Intent::SKIP, "{what}");
        }
    }

    /// The limit is inside the bound the guard checks, on both sides, so a
    /// proposal is never refused for a limit the core itself chose.
    #[test]
    fn the_limit_is_inside_the_slippage_bound() {
        let p = policy();
        let e6 = u128::from(crate::config::E6);
        for net in [
            30 * i128::try_from(e6).expect("fits"),
            -(30 * i128::try_from(e6).expect("fits")),
        ] {
            let state = BookState {
                net_delta: net,
                gross: 40 * e6,
                ..BookState::default()
            };
            let i = propose(&state, &feats(Regime::Chop), &p);
            let bps = i.limit_price_e6.abs_diff(MARK_E6) * 10_000 / MARK_E6;
            assert!(
                bps <= u64::from(p.max_slippage_bps),
                "limit {} is {bps} bps from the mark",
                i.limit_price_e6
            );
        }
    }
}
