//! demo_perps: the mock venue behind the real adapter trait, zero token custody (P04).
//!
//! P01 scaffold. The only instruction is `ping`, which exists so the pinned
//! toolchain, the SBF build and the LiteSVM harness are proven end to end.
//! Every real instruction lands in its owning prompt against `docs/10` and
//! ADR-004; `ping` is removed then.
#![allow(unexpected_cfgs)]
#![forbid(unsafe_code)]

use anchor_lang::prelude::*;

declare_id!("3Zcd8XsFWBTVku5GxQjwEBC7sLrJhF8vadyTnTr56hxB");

#[program]
pub mod demo_perps {
    use super::*;

    /// Scaffold-only. Logs one line so a devnet transaction can prove the
    /// pinned toolchain deployed and the harness decodes logs.
    pub fn ping(_ctx: Context<Ping>) -> Result<()> {
        msg!("demo_perps: scaffold ping");
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Ping {}
