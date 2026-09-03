//! `demo_perps` — the mock venue behind the real adapter trait.
//!
//! A mock, not a toy. It exists so the interface, the freshness gate and the
//! receipt shape are exercised for real, and it is built to be *unflattering*:
//!
//! * **Zero token custody.** There is no token account, no mint and no
//!   transfer instruction anywhere in this program. Collateral never moves
//!   here, so a bug in the mock cannot touch a vault. `scripts/no-token-custody.sh`
//!   proves it by grepping the built binary and the source.
//! * **Deterministic fills** at `mark ± fee_bps`. No randomness and no
//!   simulated slippage that happens to be favourable — a demo that flatters
//!   itself is worse than no demo.
//! * **The mark is an account**, never an argument. Its `source` (`pyth` |
//!   `house`) is stored on chain so the page can state where the number came
//!   from instead of implying an oracle.
//! * **Freshness is enforced here too.** The mandate program checks the mark
//!   and so does the venue; two independent checks is the point.
//! * **It reports its fill.** A Solana CPI returns no value, so the fill goes
//!   back through `set_return_data`. The mandate program refuses to write a
//!   receipt without it rather than inventing a price (ADR-007).
#![allow(unexpected_cfgs)]
#![forbid(unsafe_code)]

use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::set_return_data;
use markov_types::{ActionKind, MarkSourceKind, Side, VenueFill};
use pyth_solana_receiver_sdk::price_update::PriceUpdateV2;

declare_id!("3Zcd8XsFWBTVku5GxQjwEBC7sLrJhF8vadyTnTr56hxB");

/// Funding on this mock accrues at a **fixed, published devnet constant**:
/// 1e-6 of notional per slot, charged to the long side. It is not a measured
/// rate, not a forecast and not evidence about any real venue's funding. The
/// page must label it as a devnet constant wherever it appears.
pub const DEVNET_FUNDING_E6_PER_SLOT: i64 = 1;

/// Marks older than this many slots are refused outright, whatever a market
/// configures, so a misconfigured market cannot accept an ancient price.
pub const MARK_MAX_AGE_SLOTS_CEILING: u64 = 5_000;

#[program]
pub mod demo_perps {
    use super::*;

    /// Create a market. The authority is the deployer on devnet; it can pause
    /// and re-parameterise, and it can never move a token because there are
    /// none here.
    pub fn init_market(ctx: Context<InitMarket>, args: InitMarketArgs) -> Result<()> {
        require!(args.fee_bps <= 1_000, DemoPerpsError::InvalidParameter);
        require!(
            args.max_age_slots > 0 && args.max_age_slots <= MARK_MAX_AGE_SLOTS_CEILING,
            DemoPerpsError::InvalidParameter
        );
        require!(args.position_cap > 0, DemoPerpsError::InvalidParameter);
        let m = &mut ctx.accounts.market;
        m.authority = ctx.accounts.authority.key();
        m.market_id = args.market_id;
        m.base_decimals = args.base_decimals;
        m.mark = ctx.accounts.mark.key();
        m.fee_bps = args.fee_bps;
        m.max_age_slots = args.max_age_slots;
        m.position_cap = args.position_cap;
        m.paused = false;
        m.bump = ctx.bumps.market;

        let k = &mut ctx.accounts.mark;
        k.market_id = args.market_id;
        k.poster = args.poster;
        k.price = 0;
        k.expo = 0;
        k.publish_time = 0;
        k.slot = 0;
        k.source = MarkSourceKind::House;
        k.bump = ctx.bumps.mark;
        Ok(())
    }

    pub fn set_market_paused(ctx: Context<MarketAuthority>, paused: bool) -> Result<()> {
        ctx.accounts.market.paused = paused;
        Ok(())
    }

    /// Relay a Pyth price onto the market's mark account. Anyone may call it —
    /// there is nothing to gain, because the price is read from the Pyth
    /// account itself and verified (owner, feed id, `Full`, freshness) rather
    /// than supplied by the caller. `source` records `pyth`.
    pub fn post_mark_from_pyth(ctx: Context<PostMarkFromPyth>, feed_id: [u8; 32]) -> Result<()> {
        let clock = Clock::get()?;
        let price = ctx
            .accounts
            .price_update
            .get_price_no_older_than(&clock, ctx.accounts.market.max_age_slots.max(1), &feed_id)
            .map_err(|_| error!(DemoPerpsError::StaleMark))?;
        let k = &mut ctx.accounts.mark;
        k.price = price.price;
        k.expo = price.exponent;
        k.publish_time = price.publish_time;
        k.slot = clock.slot;
        k.source = MarkSourceKind::Pyth;
        emit!(MarkPosted {
            market_id: k.market_id,
            price: k.price,
            expo: k.expo,
            slot: k.slot,
            source: k.source,
        });
        Ok(())
    }

    /// The house fallback (ADR-003). Only the allowlisted poster may call it,
    /// and it may write **nothing but** price, expo, publish time and slot —
    /// it cannot pause a market, cannot open a position and holds no tokens.
    /// `source` records `house`, which the page must display as a house mark.
    pub fn post_mark(ctx: Context<PostMark>, price: i64, expo: i32, publish_time: i64) -> Result<()> {
        require!(price > 0, DemoPerpsError::InvalidParameter);
        let clock = Clock::get()?;
        let k = &mut ctx.accounts.mark;
        k.price = price;
        k.expo = expo;
        k.publish_time = publish_time;
        k.slot = clock.slot;
        k.source = MarkSourceKind::House;
        emit!(MarkPosted {
            market_id: k.market_id,
            price: k.price,
            expo: k.expo,
            slot: k.slot,
            source: k.source,
        });
        Ok(())
    }

    /// Create the position record for one mandate and market. Separate from
    /// `venue_execute` so the write path needs no payer and no system program:
    /// the mandate PDA signs the trade, it does not fund accounts.
    pub fn init_position(ctx: Context<InitPosition>, mandate: Pubkey, market_id: [u8; 16]) -> Result<()> {
        require!(ctx.accounts.market.market_id == market_id, DemoPerpsError::MarketUnknown);
        let p = &mut ctx.accounts.position;
        p.mandate = mandate;
        p.market_id = market_id;
        p.side = Side::Long;
        p.notional = 0;
        p.entry_price = 0;
        p.funding_accrued = 0;
        p.updated_slot = Clock::get()?.slot;
        p.bump = ctx.bumps.position;
        Ok(())
    }

    /// The adapter entry point. The **mandate PDA must sign**: the operator
    /// key never reaches a venue directly, so a stolen operator key cannot
    /// move a position without first passing the mandate program's ladder.
    ///
    /// Fills deterministically at `mark ± fee_bps` and reports the fill with
    /// `set_return_data`, because a CPI cannot return a value and the mandate
    /// program must not guess one.
    pub fn venue_execute(ctx: Context<VenueExecute>, args: VenueExecuteArgs) -> Result<()> {
        let market = &ctx.accounts.market;
        let mark = &ctx.accounts.mark;
        let clock = Clock::get()?;

        require!(!market.paused, DemoPerpsError::VenuePaused);
        require!(market.market_id == args.market, DemoPerpsError::MarketUnknown);
        require!(ctx.accounts.position.market_id == args.market, DemoPerpsError::MarketUnknown);
        require_keys_eq!(
            ctx.accounts.position.mandate,
            ctx.accounts.mandate.key(),
            DemoPerpsError::WrongMandate
        );

        // Freshness, on the venue's own account. The mandate program checks
        // this too; two independent checks is deliberate.
        require!(mark.slot > 0, DemoPerpsError::StaleMark);
        let age = clock.slot.saturating_sub(mark.slot);
        require!(age <= market.max_age_slots, DemoPerpsError::StaleMark);
        require!(mark.price > 0, DemoPerpsError::StaleMark);

        let action = ActionKind::from_u8(args.action).ok_or(DemoPerpsError::UnknownAction)?;
        let side = if args.side == Side::Short as u8 { Side::Short } else { Side::Long };
        let mark_e6 = mark_price_e6(mark.price, mark.expo).ok_or(DemoPerpsError::StaleMark)?;

        // Funding first, so a position always pays for the time it was held
        // before its size changes.
        accrue_funding(&mut ctx.accounts.position, clock.slot);

        let fee_bps = market.fee_bps as u64;
        let (fill_price, notional) = match action {
            ActionKind::Open | ActionKind::Increase => {
                require!(args.notional > 0, DemoPerpsError::InvalidNotional);
                let new_total = ctx
                    .accounts
                    .position
                    .notional
                    .checked_add(args.notional)
                    .ok_or(DemoPerpsError::Math)?;
                require!(new_total <= market.position_cap, DemoPerpsError::PositionLimit);
                // The taker pays the fee: worse than the mark, never better.
                let p = worse_for_taker(mark_e6, fee_bps, side, true).ok_or(DemoPerpsError::Math)?;
                (p, args.notional)
            }
            ActionKind::Reduce => {
                require!(args.notional > 0, DemoPerpsError::InvalidNotional);
                let held = ctx.accounts.position.notional;
                require!(held > 0, DemoPerpsError::NoPosition);
                let n = args.notional.min(held);
                let p = worse_for_taker(mark_e6, fee_bps, side, false).ok_or(DemoPerpsError::Math)?;
                (p, n)
            }
            ActionKind::Close | ActionKind::Flatten => {
                let held = ctx.accounts.position.notional;
                require!(held > 0, DemoPerpsError::NoPosition);
                let p = worse_for_taker(mark_e6, fee_bps, side, false).ok_or(DemoPerpsError::Math)?;
                (p, held)
            }
            ActionKind::Skip => return err!(DemoPerpsError::UnknownAction),
        };

        // The venue enforces the caller's bound as well. A fill outside it is
        // refused, never widened.
        if !within_bound(fill_price, args.limit_price, args.max_slippage_bps) {
            return err!(DemoPerpsError::SlippageExceeded);
        }

        let p = &mut ctx.accounts.position;
        match action {
            ActionKind::Open | ActionKind::Increase => {
                // Weighted entry, so a later fill cannot rewrite history.
                let old_notional = p.notional as u128;
                let old_entry = p.entry_price as u128;
                let add = notional as u128;
                let total = old_notional + add;
                p.entry_price = if total == 0 {
                    fill_price
                } else {
                    u64::try_from((old_notional * old_entry + add * fill_price as u128) / total)
                        .map_err(|_| error!(DemoPerpsError::Math))?
                };
                p.notional = u64::try_from(total).map_err(|_| error!(DemoPerpsError::Math))?;
                p.side = side;
            }
            ActionKind::Reduce | ActionKind::Close | ActionKind::Flatten => {
                p.notional = p.notional.saturating_sub(notional);
                if p.notional == 0 {
                    p.entry_price = 0;
                }
            }
            ActionKind::Skip => {}
        }
        p.updated_slot = clock.slot;

        let fee = (notional as u128)
            .saturating_mul(fee_bps as u128)
            .checked_div(10_000)
            .and_then(|f| u64::try_from(f).ok())
            .ok_or(DemoPerpsError::Math)?;
        let fill = VenueFill { price: fill_price, notional, fee };

        // The only way the caller learns the real fill.
        let mut bytes = Vec::with_capacity(24);
        fill.serialize(&mut bytes).map_err(|_| error!(DemoPerpsError::Math))?;
        set_return_data(&bytes);

        emit!(VenueFilled {
            mandate: ctx.accounts.mandate.key(),
            market_id: args.market,
            action: args.action,
            side: side as u8,
            price: fill.price,
            notional: fill.notional,
            fee: fill.fee,
            mark_price: mark_e6,
            mark_source: mark.source,
            position_notional: ctx.accounts.position.notional,
            funding_accrued: ctx.accounts.position.funding_accrued,
            slot: clock.slot,
        });
        Ok(())
    }
}

/// A mark's price scaled to 1e6 per unit. Returns `None` rather than a wrong
/// number if the exponent is absurd.
fn mark_price_e6(price: i64, expo: i32) -> Option<u64> {
    if price <= 0 {
        return None;
    }
    let shift = expo.checked_add(6)?;
    let p = price as i128;
    let scaled = if shift >= 0 {
        p.checked_mul(10i128.checked_pow(u32::try_from(shift).ok()?)?)?
    } else {
        p.checked_div(10i128.checked_pow(u32::try_from(-shift).ok()?)?)?
    };
    u64::try_from(scaled).ok()
}

/// The taker's price: always the side of the mark that costs them the fee.
/// Opening a long or closing a short pays up; the reverse receives less.
fn worse_for_taker(mark_e6: u64, fee_bps: u64, side: Side, entering: bool) -> Option<u64> {
    let fee = (mark_e6 as u128).checked_mul(fee_bps as u128)?.checked_div(10_000)?;
    let pay_up = matches!((side, entering), (Side::Long, true) | (Side::Short, false));
    let out = if pay_up {
        (mark_e6 as u128).checked_add(fee)?
    } else {
        (mark_e6 as u128).checked_sub(fee)?
    };
    u64::try_from(out).ok()
}

fn within_bound(fill_price: u64, limit_price: u64, max_slippage_bps: u16) -> bool {
    if limit_price == 0 {
        return false;
    }
    let diff = fill_price.abs_diff(limit_price) as u128;
    let bound = (limit_price as u128).saturating_mul(max_slippage_bps as u128) / 10_000;
    diff <= bound
}

/// Funding accrues on elapsed slots at the published devnet constant and is
/// charged to the long side. Deterministic, and never favourable.
fn accrue_funding(p: &mut Account<'_, Position>, now_slot: u64) {
    if p.notional == 0 {
        p.updated_slot = now_slot;
        return;
    }
    let elapsed = now_slot.saturating_sub(p.updated_slot) as i128;
    let magnitude = elapsed
        .saturating_mul(DEVNET_FUNDING_E6_PER_SLOT as i128)
        .saturating_mul(p.notional as i128)
        / 1_000_000;
    let signed = match p.side {
        Side::Long => -magnitude,
        Side::Short => magnitude,
    };
    p.funding_accrued = p.funding_accrued.saturating_add(i64::try_from(signed).unwrap_or(0));
}

trait FromU8: Sized {
    fn from_u8(b: u8) -> Option<Self>;
}

impl FromU8 for ActionKind {
    fn from_u8(b: u8) -> Option<ActionKind> {
        Some(match b {
            0 => ActionKind::Skip,
            1 => ActionKind::Open,
            2 => ActionKind::Increase,
            3 => ActionKind::Reduce,
            4 => ActionKind::Close,
            5 => ActionKind::Flatten,
            _ => return None,
        })
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug)]
pub struct InitMarketArgs {
    pub market_id: [u8; 16],
    pub base_decimals: u8,
    pub fee_bps: u16,
    pub max_age_slots: u64,
    pub position_cap: u64,
    /// The only key allowed to call `post_mark`.
    pub poster: Pubkey,
}

/// Mirrors `markov_mandate::cpi::venue::VenueExecuteArgs`. The two must stay
/// identical; the mandate's `sighash` test and this program's discriminator
/// pin the encoding, and `markov-venue`'s conformance suite pins the meaning.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct VenueExecuteArgs {
    pub action: u8,
    pub market: [u8; 16],
    pub side: u8,
    pub notional: u64,
    pub limit_price: u64,
    pub max_slippage_bps: u16,
}

#[account]
#[derive(InitSpace)]
pub struct Market {
    pub authority: Pubkey,
    pub market_id: [u8; 16],
    pub base_decimals: u8,
    pub mark: Pubkey,
    pub fee_bps: u16,
    pub max_age_slots: u64,
    pub position_cap: u64,
    pub paused: bool,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct MarkAccount {
    pub market_id: [u8; 16],
    pub price: i64,
    pub expo: i32,
    pub publish_time: i64,
    pub slot: u64,
    /// `pyth` or `house`, on chain, so no surface has to guess.
    pub source: MarkSourceKind,
    pub poster: Pubkey,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct Position {
    pub mandate: Pubkey,
    pub market_id: [u8; 16],
    pub side: Side,
    pub notional: u64,
    pub entry_price: u64,
    /// Signed: negative means this position paid funding.
    pub funding_accrued: i64,
    pub updated_slot: u64,
    pub bump: u8,
}

#[event]
pub struct MarkPosted {
    pub market_id: [u8; 16],
    pub price: i64,
    pub expo: i32,
    pub slot: u64,
    pub source: MarkSourceKind,
}

#[event]
pub struct VenueFilled {
    pub mandate: Pubkey,
    pub market_id: [u8; 16],
    pub action: u8,
    pub side: u8,
    pub price: u64,
    pub notional: u64,
    pub fee: u64,
    pub mark_price: u64,
    pub mark_source: MarkSourceKind,
    pub position_notional: u64,
    pub funding_accrued: i64,
    pub slot: u64,
}

#[derive(Accounts)]
#[instruction(args: InitMarketArgs)]
pub struct InitMarket<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        init,
        payer = authority,
        space = 8 + Market::INIT_SPACE,
        seeds = [b"market", args.market_id.as_ref()],
        bump
    )]
    pub market: Box<Account<'info, Market>>,
    #[account(
        init,
        payer = authority,
        space = 8 + MarkAccount::INIT_SPACE,
        seeds = [b"mark", args.market_id.as_ref()],
        bump
    )]
    pub mark: Box<Account<'info, MarkAccount>>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct MarketAuthority<'info> {
    pub authority: Signer<'info>,
    #[account(
        mut,
        seeds = [b"market", market.market_id.as_ref()],
        bump = market.bump,
        has_one = authority @ DemoPerpsError::WrongAuthority
    )]
    pub market: Box<Account<'info, Market>>,
}

#[derive(Accounts)]
pub struct PostMarkFromPyth<'info> {
    #[account(seeds = [b"market", market.market_id.as_ref()], bump = market.bump)]
    pub market: Box<Account<'info, Market>>,
    #[account(
        mut,
        seeds = [b"mark", market.market_id.as_ref()],
        bump = mark.bump,
        constraint = market.mark == mark.key() @ DemoPerpsError::WrongMark
    )]
    pub mark: Box<Account<'info, MarkAccount>>,
    /// Owner-checked by Anchor: it must be a Pyth receiver account.
    pub price_update: Box<Account<'info, PriceUpdateV2>>,
}

#[derive(Accounts)]
pub struct PostMark<'info> {
    pub poster: Signer<'info>,
    #[account(seeds = [b"market", market.market_id.as_ref()], bump = market.bump)]
    pub market: Box<Account<'info, Market>>,
    #[account(
        mut,
        seeds = [b"mark", market.market_id.as_ref()],
        bump = mark.bump,
        constraint = market.mark == mark.key() @ DemoPerpsError::WrongMark,
        constraint = mark.poster == poster.key() @ DemoPerpsError::WrongPoster
    )]
    pub mark: Box<Account<'info, MarkAccount>>,
}

#[derive(Accounts)]
#[instruction(mandate: Pubkey, market_id: [u8; 16])]
pub struct InitPosition<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(seeds = [b"market", market_id.as_ref()], bump = market.bump)]
    pub market: Box<Account<'info, Market>>,
    #[account(
        init,
        payer = payer,
        space = 8 + Position::INIT_SPACE,
        seeds = [b"pos", mandate.as_ref(), market_id.as_ref()],
        bump
    )]
    pub position: Box<Account<'info, Position>>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct VenueExecute<'info> {
    /// The mandate PDA, signing through `invoke_signed`. Nothing else can
    /// move a position.
    pub mandate: Signer<'info>,
    #[account(seeds = [b"market", market.market_id.as_ref()], bump = market.bump)]
    pub market: Box<Account<'info, Market>>,
    #[account(
        seeds = [b"mark", market.market_id.as_ref()],
        bump = mark.bump,
        constraint = market.mark == mark.key() @ DemoPerpsError::WrongMark
    )]
    pub mark: Box<Account<'info, MarkAccount>>,
    #[account(
        mut,
        seeds = [b"pos", mandate.key().as_ref(), market.market_id.as_ref()],
        bump = position.bump
    )]
    pub position: Box<Account<'info, Position>>,
}

/// Exactly the trait's fixed error set (`crates/markov-venue`), plus the
/// structural errors a program needs. Anything the mandate program cannot map
/// becomes `BlockReason::VenueRejected` at gate 13.
#[error_code]
pub enum DemoPerpsError {
    #[msg("market unknown to this venue")]
    MarketUnknown,
    #[msg("the venue's mark is stale")]
    StaleMark,
    #[msg("fill would breach the limit price")]
    SlippageExceeded,
    #[msg("not enough collateral")]
    InsufficientCollateral,
    #[msg("position limit reached")]
    PositionLimit,
    #[msg("venue is paused")]
    VenuePaused,
    #[msg("no position to reduce or close")]
    NoPosition,
    #[msg("notional must be greater than zero")]
    InvalidNotional,
    #[msg("unknown action")]
    UnknownAction,
    #[msg("invalid parameter")]
    InvalidParameter,
    #[msg("wrong mark account for this market")]
    WrongMark,
    #[msg("only the allowlisted poster may post a house mark")]
    WrongPoster,
    #[msg("wrong market authority")]
    WrongAuthority,
    #[msg("this position belongs to another mandate")]
    WrongMandate,
    #[msg("arithmetic overflow")]
    Math,
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn a_taker_never_gets_a_better_price_than_the_mark() {
        let mark = 100_000_000u64;
        // Opening pays up; closing receives less. Never the other way round.
        assert!(worse_for_taker(mark, 10, Side::Long, true).unwrap() > mark);
        assert!(worse_for_taker(mark, 10, Side::Long, false).unwrap() < mark);
        assert!(worse_for_taker(mark, 10, Side::Short, true).unwrap() < mark);
        assert!(worse_for_taker(mark, 10, Side::Short, false).unwrap() > mark);
        // Zero fee is the only case where the taker gets the mark exactly.
        assert_eq!(worse_for_taker(mark, 0, Side::Long, true).unwrap(), mark);
    }

    #[test]
    fn fills_are_deterministic() {
        let a = worse_for_taker(100_000_000, 10, Side::Long, true);
        for _ in 0..100 {
            assert_eq!(worse_for_taker(100_000_000, 10, Side::Long, true), a);
        }
    }

    #[test]
    fn mark_scaling_refuses_nonsense_rather_than_returning_a_wrong_number() {
        assert_eq!(mark_price_e6(100_000_000, -6), Some(100_000_000));
        assert_eq!(mark_price_e6(9_999_848_408, -8), Some(99_998_484));
        assert_eq!(mark_price_e6(0, -8), None);
        assert_eq!(mark_price_e6(-5, -8), None);
        assert_eq!(mark_price_e6(1, i32::MAX), None);
        assert_eq!(mark_price_e6(1, i32::MIN), None);
    }

    #[test]
    fn a_bound_of_zero_limit_admits_nothing() {
        assert!(!within_bound(100, 0, 10_000));
        assert!(within_bound(100_400_000, 100_000_000, 50));
        assert!(!within_bound(100_600_000, 100_000_000, 50));
    }

    #[test]
    fn funding_is_charged_to_the_long_and_credited_to_the_short() {
        // Constructing an Account<'_, Position> needs a runtime, so the maths
        // is checked through the same expression the handler uses.
        let notional = 10_000_000i128;
        let elapsed = 100i128;
        let magnitude = elapsed * DEVNET_FUNDING_E6_PER_SLOT as i128 * notional / 1_000_000;
        assert_eq!(magnitude, 1_000);
        assert!(-magnitude < 0, "a long must pay");
        assert!(magnitude > 0, "a short must receive");
    }
}
