//! The guard's golden tests.
//!
//! Every veto reason has a fixture in `src/fixtures/`, and every fixture is a
//! named test. The fixtures are JSON rather than Rust literals for two
//! reasons: the API serves the same shape, so a drift between them shows up
//! here; and a reviewer can read what a veto looks like without reading Rust.
//!
//! `every_fixture_has_a_test` closes the obvious hole — adding a fixture
//! without a test would otherwise silently prove nothing.

use markov_guard::{
    evaluate, ActionKind, BlockReason, Enforcement, GuardReason, GuardState, Intent, MandateState,
    OffChainReason, PolicyView, Side, Verdict,
};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Fixture {
    name: String,
    /// Why this case exists, in plain words. Not asserted — read by people.
    #[allow(dead_code)]
    why: String,
    expect: Expect,
    intent: Intent,
    state: GuardState,
    policy: PolicyView,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum Expect {
    Allow,
    Skip,
    /// Both halves are asserted. The enforcement tag is what the dashboard
    /// reads to decide whether it may say "enforced" without a qualifier, so a
    /// fixture that got it wrong would licence a false claim.
    Veto {
        reason: String,
        enforcement: String,
    },
}

fn check(json: &str) {
    let f: Fixture = serde_json::from_str(json).expect("fixture parses");
    let got = evaluate(&f.intent, &f.state, &f.policy);
    match (&f.expect, got) {
        (Expect::Allow, Verdict::Allow(i)) => {
            assert_eq!(
                i, f.intent,
                "{}: allow must return the intent unchanged",
                f.name
            )
        }
        (Expect::Skip, Verdict::Skip) => {}
        (
            Expect::Veto {
                reason,
                enforcement,
            },
            Verdict::Veto(r),
        ) => {
            assert_eq!(r.name(), reason, "{}: wrong reason", f.name);
            let tag = match r.enforcement() {
                Enforcement::OnChain => "OnChain",
                Enforcement::OffChainV0 => "OffChainV0",
            };
            assert_eq!(tag, enforcement, "{}: wrong enforcement tag", f.name);
        }
        (_, got) => panic!(
            "{}: expected {:?}, got {got:?}",
            f.name,
            ExpectDebug(&f.expect)
        ),
    }
}

struct ExpectDebug<'a>(&'a Expect);
impl core::fmt::Debug for ExpectDebug<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self.0 {
            Expect::Allow => write!(f, "Allow"),
            Expect::Skip => write!(f, "Skip"),
            Expect::Veto { reason, .. } => write!(f, "Veto({reason})"),
        }
    }
}

macro_rules! fixtures {
    ($($name:ident => $file:literal),* $(,)?) => {
        $(
            #[test]
            fn $name() {
                check(include_str!(concat!("../src/fixtures/", $file, ".json")));
            }
        )*

        /// Every JSON in the fixture directory is covered by a test above.
        #[test]
        fn every_fixture_has_a_test() {
            let covered: &[&str] = &[$($file),*];
            let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/src/fixtures");
            let mut found: Vec<String> = std::fs::read_dir(dir)
                .expect("fixture directory")
                .map(|e| e.expect("entry").file_name().to_string_lossy().into_owned())
                .filter(|n| n.ends_with(".json"))
                .map(|n| n.trim_end_matches(".json").to_string())
                .collect();
            found.sort();
            let mut want: Vec<String> = covered.iter().map(|s| s.to_string()).collect();
            want.sort();
            assert_eq!(found, want, "a fixture exists with no test, or the reverse");
        }
    };
}

fixtures![
    fixture_allow => "allow",
    fixture_skip => "skip",
    fixture_paused => "paused",
    fixture_revoked => "revoked",
    fixture_expired => "expired",
    fixture_stale_oracle => "stale_oracle",
    fixture_action_not_allowed => "action_not_allowed",
    fixture_over_tx_cap => "over_tx_cap",
    fixture_over_daily_cap => "over_daily_cap",
    fixture_over_spend_cap => "over_spend_cap",
    fixture_over_spend_daily_cap => "over_spend_daily_cap",
    fixture_delta_band_exceeded => "delta_band_exceeded",
    fixture_gross_exceeded => "gross_exceeded",
    fixture_slippage_exceeded => "slippage_exceeded",
    fixture_daily_loss_halt => "daily_loss_halt",
    fixture_guard_internal => "guard_internal",
];

// ---------------------------------------------------------------------------
// The acceptance tests from P05, beyond the per-reason fixtures.
// ---------------------------------------------------------------------------

/// A change to one input, for the tables below.
type BreakState = Box<dyn Fn(&mut GuardState)>;
/// A change to any of the three inputs.
type Change = Box<dyn Fn(&mut Intent, &mut GuardState, &mut PolicyView)>;

fn base() -> (Intent, GuardState, PolicyView) {
    let f: Fixture =
        serde_json::from_str(include_str!("../src/fixtures/allow.json")).expect("allow fixture");
    (f.intent, f.state, f.policy)
}

/// No path returns `Allow` on an input the guard cannot make sense of.
///
/// The two kinds are different and are labelled differently: a mark that is
/// absent or old is a *feed* problem the chain also refuses (`StaleOracle`),
/// while arithmetic that overflows or a mark stamped in the future is an input
/// the guard cannot reason about at all (`GuardInternal`).
#[test]
fn fail_closed_on_missing_input() {
    let cases: Vec<(&str, BreakState, GuardReason)> = vec![
        (
            "no mark",
            Box::new(|s: &mut GuardState| s.mark_e6 = None),
            GuardReason::OnChain(BlockReason::StaleOracle),
        ),
        (
            "no publish time",
            Box::new(|s: &mut GuardState| s.mark_publish_time = None),
            GuardReason::OnChain(BlockReason::StaleOracle),
        ),
        (
            "a mark published in the future",
            Box::new(|s: &mut GuardState| s.mark_publish_time = Some(s.now_unix + 60)),
            GuardReason::OffChainV0(OffChainReason::GuardInternal),
        ),
        (
            "now_unix at the minimum, so the age cannot be computed",
            Box::new(|s: &mut GuardState| {
                s.now_unix = i64::MIN;
                s.mark_publish_time = Some(1);
            }),
            GuardReason::OffChainV0(OffChainReason::GuardInternal),
        ),
        (
            "the day's notional overflows",
            Box::new(|s: &mut GuardState| s.day_notional_used = u64::MAX),
            GuardReason::OffChainV0(OffChainReason::GuardInternal),
        ),
        (
            "the day's spend overflows",
            Box::new(|s: &mut GuardState| s.day_spend_used = u64::MAX),
            GuardReason::OffChainV0(OffChainReason::GuardInternal),
        ),
        (
            "the book is already at the edge of i128",
            Box::new(|s: &mut GuardState| s.net_delta = i128::MAX),
            GuardReason::OffChainV0(OffChainReason::GuardInternal),
        ),
        (
            "gross is already at the edge of u128",
            Box::new(|s: &mut GuardState| s.gross = u128::MAX),
            GuardReason::OffChainV0(OffChainReason::GuardInternal),
        ),
        (
            "the mark is zero, which is not a price",
            Box::new(|s: &mut GuardState| s.mark_e6 = Some(0)),
            GuardReason::OffChainV0(OffChainReason::GuardInternal),
        ),
    ];

    for (what, break_it, want) in cases {
        let (intent, mut state, policy) = base();
        break_it(&mut state);
        let got = evaluate(&intent, &state, &policy);
        assert_eq!(got, Verdict::Veto(want), "{what}: wrong verdict");
    }
}

/// The ladder runs in the order `docs/11-AGENT-SPEC.md` §4 specifies, which is
/// the order of the program's own gates.
///
/// Everything is broken at once, then repaired one rule at a time. Each repair
/// must reveal exactly the next reason down the ladder — which pins the order,
/// not merely the set of rules. A reordering that still refuses would pass a
/// weaker test; it fails this one.
#[test]
fn mirrors_onchain_ladder_order() {
    let (_, mut state, mut policy) = base();
    let mut intent = Intent {
        action: ActionKind::Increase,
        side: Side::Long,
        notional: 50_001,
        limit_price_e6: 101_000_000,
        spend: 10_001,
    };
    state.state = MandateState::Paused;
    state.mark_publish_time = Some(state.now_unix - 200);
    state.day_notional_used = 195_000;
    state.day_spend_used = 39_500;
    state.net_delta = 95_000;
    state.gross = 295_000;
    state.equity = 940_000;
    policy.allowed_actions = PolicyView::actions_mask(&[ActionKind::Open]);

    let mut repairs: Vec<(&str, Change)> = vec![
        (
            "unpause",
            Box::new(|_: &mut Intent, s: &mut GuardState, _: &mut PolicyView| {
                s.state = MandateState::Active
            }),
        ),
        (
            "fresh mark",
            Box::new(|_: &mut Intent, s: &mut GuardState, _: &mut PolicyView| {
                s.mark_publish_time = Some(s.now_unix - 5)
            }),
        ),
        (
            "allow the action",
            Box::new(|_: &mut Intent, _: &mut GuardState, p: &mut PolicyView| {
                p.allowed_actions = PolicyView::actions_mask(&[
                    ActionKind::Open,
                    ActionKind::Increase,
                    ActionKind::Reduce,
                    ActionKind::Close,
                    ActionKind::Flatten,
                ])
            }),
        ),
        (
            "shrink the trade",
            Box::new(|i: &mut Intent, _: &mut GuardState, _: &mut PolicyView| i.notional = 10_000),
        ),
        (
            "clear the day's notional",
            Box::new(|_: &mut Intent, s: &mut GuardState, _: &mut PolicyView| {
                s.day_notional_used = 0
            }),
        ),
        (
            "shrink the spend",
            Box::new(|i: &mut Intent, _: &mut GuardState, _: &mut PolicyView| i.spend = 1_000),
        ),
        (
            "clear the day's spend",
            Box::new(|_: &mut Intent, s: &mut GuardState, _: &mut PolicyView| s.day_spend_used = 0),
        ),
        (
            "flatten the delta",
            Box::new(|_: &mut Intent, s: &mut GuardState, _: &mut PolicyView| s.net_delta = 0),
        ),
        (
            "clear the gross",
            Box::new(|_: &mut Intent, s: &mut GuardState, _: &mut PolicyView| s.gross = 0),
        ),
        (
            "tighten the limit",
            Box::new(|i: &mut Intent, _: &mut GuardState, _: &mut PolicyView| {
                i.limit_price_e6 = 100_200_000
            }),
        ),
        (
            "restore equity",
            Box::new(|_: &mut Intent, s: &mut GuardState, _: &mut PolicyView| {
                s.equity = s.session_start_equity
            }),
        ),
    ];

    let expected = [
        GuardReason::OnChain(BlockReason::Paused),
        GuardReason::OnChain(BlockReason::StaleOracle),
        GuardReason::OnChain(BlockReason::ActionNotAllowed),
        GuardReason::OnChain(BlockReason::OverTxCap),
        GuardReason::OnChain(BlockReason::OverDailyCap),
        GuardReason::OnChain(BlockReason::OverSpendCap),
        GuardReason::OnChain(BlockReason::OverSpendDailyCap),
        GuardReason::OffChainV0(OffChainReason::DeltaBandExceeded),
        GuardReason::OffChainV0(OffChainReason::GrossExceeded),
        GuardReason::OnChain(BlockReason::SlippageExceeded),
        GuardReason::OffChainV0(OffChainReason::DailyLossHalt),
    ];
    assert_eq!(repairs.len(), expected.len(), "one repair per rung");

    for (i, want) in expected.iter().enumerate() {
        assert_eq!(
            evaluate(&intent, &state, &policy),
            Verdict::Veto(*want),
            "rung {i} ({}) reported the wrong reason",
            want.name()
        );
        let (_, repair) = &mut repairs[i];
        repair(&mut intent, &mut state, &mut policy);
    }

    assert_eq!(
        evaluate(&intent, &state, &policy),
        Verdict::Allow(intent),
        "every rung repaired, so the intent must be allowed"
    );
}

/// `Allow` requires every rule to pass, not most of them. Each single
/// perturbation of the allow fixture must stop being an allow.
#[test]
fn allow_only_when_all_pass() {
    let (base_intent, base_state, base_policy) = base();
    assert!(matches!(
        evaluate(&base_intent, &base_state, &base_policy),
        Verdict::Allow(_)
    ));

    let breaks: Vec<(&str, Change)> = vec![
        (
            "paused",
            Box::new(|_: &mut Intent, s: &mut GuardState, _: &mut PolicyView| {
                s.state = MandateState::Paused
            }),
        ),
        (
            "revoked",
            Box::new(|_: &mut Intent, s: &mut GuardState, _: &mut PolicyView| {
                s.state = MandateState::Revoked
            }),
        ),
        (
            "expired",
            Box::new(|_: &mut Intent, s: &mut GuardState, _: &mut PolicyView| {
                s.state = MandateState::Expired
            }),
        ),
        (
            "stale mark",
            Box::new(|_: &mut Intent, s: &mut GuardState, p: &mut PolicyView| {
                s.mark_publish_time = Some(s.now_unix - p.max_mark_age_secs - 1)
            }),
        ),
        (
            "action not allowed",
            Box::new(|_: &mut Intent, _: &mut GuardState, p: &mut PolicyView| {
                p.allowed_actions = 0
            }),
        ),
        (
            "over the per-trade cap",
            Box::new(|i: &mut Intent, _: &mut GuardState, p: &mut PolicyView| {
                i.notional = p.per_tx_cap + 1
            }),
        ),
        (
            "over the daily cap",
            Box::new(|i: &mut Intent, s: &mut GuardState, p: &mut PolicyView| {
                s.day_notional_used = p.daily_cap - i.notional + 1
            }),
        ),
        (
            "over the spend cap",
            Box::new(|i: &mut Intent, _: &mut GuardState, p: &mut PolicyView| {
                i.spend = p.spend_cap + 1
            }),
        ),
        (
            "over the daily spend cap",
            Box::new(|i: &mut Intent, s: &mut GuardState, p: &mut PolicyView| {
                s.day_spend_used = p.spend_daily_cap - i.spend + 1
            }),
        ),
        (
            "outside the delta band",
            Box::new(|i: &mut Intent, s: &mut GuardState, p: &mut PolicyView| {
                s.net_delta =
                    i128::try_from(p.delta_band).expect("band fits") - i128::from(i.notional) + 1
            }),
        ),
        (
            "over the gross ceiling",
            Box::new(|i: &mut Intent, s: &mut GuardState, p: &mut PolicyView| {
                s.gross = p.max_gross - u128::from(i.notional) + 1
            }),
        ),
        (
            "outside the slippage bound",
            Box::new(|i: &mut Intent, s: &mut GuardState, _: &mut PolicyView| {
                i.limit_price_e6 = s.mark_e6.expect("a mark") * 2
            }),
        ),
        (
            "past the daily loss stop",
            Box::new(|_: &mut Intent, s: &mut GuardState, p: &mut PolicyView| {
                s.equity = s.session_start_equity
                    - (s.session_start_equity / 10_000 * u64::from(p.daily_loss_bps))
            }),
        ),
    ];

    for (what, break_it) in breaks {
        let (mut i, mut s, mut p) = (base_intent, base_state, base_policy);
        break_it(&mut i, &mut s, &mut p);
        match evaluate(&i, &s, &p) {
            Verdict::Veto(_) => {}
            other => panic!("{what}: still allowed ({other:?})"),
        }
    }
}
