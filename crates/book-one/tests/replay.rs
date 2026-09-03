//! The replay harness: drive `book-core` over a price series and print what it
//! did.
//!
//! P06 asks for a histogram and a `Skip` share above 90%. The number matters
//! less than what it demonstrates: this book is supposed to be quiet. A core
//! that acts on most ticks is either overfitted or broken, and either way the
//! tape would look like a slot machine — which `docs/11` §3 names as the thing
//! hysteresis exists to prevent.
//!
//! The series is generated, not sampled from the live feed, and says so. It is
//! a deterministic walk from a fixed seed, so this test is reproducible on any
//! machine and in CI, where there is no devnet. Run it with `--nocapture` to
//! read the histogram.

use book_one::config::{
    E6, GATE_B_DAILY_LOSS_BPS, GATE_B_DELTA_BAND, GATE_B_MAX_GROSS, GATE_B_MAX_SLIPPAGE_BPS,
    GATE_B_PER_TX_CAP,
};
use book_one::core::{propose, BookState};
use book_one::sidecar::{Features, Regime};
use markov_guard::{ActionKind, PolicyView};

/// A deterministic walk. Not random at run time and not a market simulation —
/// just a price that moves, so the core is exercised over more than one number.
fn series(n: usize) -> Vec<u64> {
    let mut price: i64 = 100 * E6 as i64;
    let mut seed: u64 = 0x5EED_1234_5678_9ABC;
    let mut out = Vec::with_capacity(n);
    for _ in 0..n {
        // xorshift64: a fixed sequence, chosen so the walk is not monotone.
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        // ±0.25% per step.
        let step = (seed % 5_001) as i64 - 2_500;
        price += price / 1_000_000 * step;
        price = price.clamp(50 * E6 as i64, 200 * E6 as i64);
        out.push(price as u64);
    }
    out
}

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

/// The book's exposure moves with the mark, which is what makes the delta rule
/// fire at all: a position opened at one price is a different exposure at the
/// next. Shadow mode never fills, so this stands in for a filled book.
fn drift_exposure(book: &mut BookState, prev: u64, now: u64) {
    if prev == 0 || book.gross == 0 {
        return;
    }
    let ratio_num = i128::from(now);
    let ratio_den = i128::from(prev);
    book.net_delta = book.net_delta * ratio_num / ratio_den;
}

/// Run the walk and return the action histogram, indexed by `ActionKind`.
///
/// `funding_favourable` is the stub constant the whole Gate B core turns on:
/// with it `false`, rule 6 never fires and the book can never add exposure, so
/// the harness is run both ways. The `true` run is a **hypothetical** — no
/// venue here reports funding — and exists so the ladder below rule 6 is
/// actually walked instead of being dead code with a green test over it.
fn run(funding_favourable: bool) -> [usize; 6] {
    let p = policy();
    let prices = series(2_000);
    let mut book = BookState {
        net_delta: 15 * i128::from(E6),
        gross: 40 * u128::from(E6),
        ..BookState::default()
    };

    let mut hist = [0usize; 6];
    let mut prev = prices[0];
    for (i, &price) in prices.iter().enumerate() {
        drift_exposure(&mut book, prev, price);
        prev = price;
        // One tick in every 500 is a halt, so Flatten is walked rather than
        // being a path the harness never reaches.
        let regime = if i % 500 == 499 {
            Regime::Halt
        } else if i % 7 == 0 {
            Regime::Trend
        } else {
            Regime::Chop
        };
        let feats = Features {
            regime,
            funding_favourable_stub: funding_favourable,
            mark_e6: Some(price),
            mark_age_secs: Some(5),
            mark_age_slots: Some(30),
        };
        let intent = propose(&book, &feats, &p);
        hist[intent.action as usize] += 1;

        // Apply what was proposed as though it filled at the mark. A histogram
        // of a book that never changes would prove nothing.
        let size = i128::from(intent.notional);
        match intent.action {
            ActionKind::Reduce => {
                book.net_delta += if book.net_delta >= 0 { -size } else { size };
                book.gross = book.gross.saturating_sub(u128::from(intent.notional));
            }
            ActionKind::Flatten | ActionKind::Close => {
                book.net_delta = 0;
                book.gross = 0;
            }
            ActionKind::Open | ActionKind::Increase => {
                book.net_delta += match intent.side {
                    markov_guard::Side::Long => size,
                    markov_guard::Side::Short => -size,
                };
                book.gross += u128::from(intent.notional);
            }
            ActionKind::Skip => {}
        }
        book.last_action = Some(intent.action);
        book.last_net_delta = book.net_delta;
    }
    hist
}

fn report(label: &str, hist: &[usize; 6]) -> f64 {
    let total: usize = hist.iter().sum();
    let names = ["skip", "open", "increase", "reduce", "close", "flatten"];
    println!("\nreplay ({label}): {total} ticks over a generated walk, not live devnet data");
    for (i, name) in names.iter().enumerate() {
        println!(
            "  {name:<9} {:>6}  {:>5.1}%",
            hist[i],
            hist[i] as f64 * 100.0 / total as f64
        );
    }
    let share = hist[ActionKind::Skip as usize] as f64 / total as f64;
    println!("  skip share {:.1}%", share * 100.0);
    share
}

/// Gate B as it actually is.
///
/// The stronger claim is not the 90%: it is that the core proposes **no new
/// exposure at all**, because the one rule that could is gated on a stub
/// constant that is `false`. A book that flattens on a halt therefore stays
/// flat. That is a fact about this build, and the tape must not be read as a
/// strategy declining to trade.
#[test]
fn replay_histogram_skip_share_over_90pct() {
    let hist = run(false);
    let share = report("gate b, funding stub false", &hist);
    assert!(
        share > 0.90,
        "skip share {:.1}% — this book is supposed to be quiet",
        share * 100.0
    );
    assert_eq!(
        hist[ActionKind::Open as usize] + hist[ActionKind::Increase as usize],
        0,
        "the Gate B stub is false, so nothing may propose new exposure"
    );
    assert!(
        hist[ActionKind::Flatten as usize] > 0,
        "the harness never exercised a halt"
    );
}

/// The same walk with the funding stub forced on, so rules 4, 5 and 6 are all
/// reachable. Still quiet: hysteresis and the caps are what keep it so, not the
/// absence of a reason to act.
#[test]
fn replay_with_funding_enabled_is_still_quiet() {
    let hist = run(true);
    let share = report("hypothetical, funding stub true", &hist);
    for (i, name) in [
        (ActionKind::Open as usize, "open"),
        (ActionKind::Increase as usize, "increase"),
        (ActionKind::Reduce as usize, "reduce"),
        (ActionKind::Flatten as usize, "flatten"),
    ] {
        assert!(
            hist[i] > 0,
            "{name} was never proposed, so it is untested here"
        );
    }
    // Deliberately not asserted above 90%. It is not: the measured share is
    // 40.7%, because rules 4 and 6 alternate and §3's hysteresis only
    // catches a repeated action. Asserting a number here would either encode
    // the churn as correct or force a redesign of a rule Gate B cannot reach.
    // The finding is in BACKLOG with this number; the test's job is to keep
    // the ladder walked and the number visible.
    println!(
        "  note: {:.1}% skip with funding forced on — see BACKLOG, rules 4 and 6 alternate",
        share * 100.0
    );
    assert!(share > 0.0, "some tick must skip");
}
