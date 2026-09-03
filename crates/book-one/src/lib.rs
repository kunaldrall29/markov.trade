//! The house agent's parts, as a library so the binary is not the only way to
//! run them: the replay harness drives `core` directly, and P07's runtime
//! reuses the same modules rather than a copy of them.
//!
//! `VENUE=shadow` is the Gate B default and submits nothing. Nothing in here
//! reads a clock or a socket except `tick` and `markov_marks`.

pub mod config;
pub mod core;
pub mod paper;
pub mod sidecar;
pub mod tick;
