//! `book-core`, the deterministic proposer. P08 placeholder: the only answer
//! is `Skip`, which is also the answer the full core (P06) gives whenever
//! nothing in docs/11 §3 fires. Pure: no clock, no I/O.

use markov_guard::Intent;

use crate::sidecar::Features;

pub fn propose(_feats: &Features) -> Intent {
    Intent::SKIP
}
