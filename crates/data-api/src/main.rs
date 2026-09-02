//! Read-only axum API: /health, /v1/receipts, /v1/book/stats, /v1/mandates, /v1/paper, /v1/facts.
//!
//! Scaffold from P01. The binary exists so every later prompt has a place to
//! put its code and CI already builds it. It does nothing yet on purpose.
#![forbid(unsafe_code)]
#![warn(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

fn main() {
    // Fail closed: a scaffold that is started by mistake must not pretend to run.
    eprintln!("data-api: scaffold only, nothing to run yet (see docs/SESSION_LOG.md)");
    std::process::exit(2);
}
