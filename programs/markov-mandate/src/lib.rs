//! `markov-mandate` — the lock.
//!
//! An owner deposits USDC-d into a mandate they still control. A house
//! operator may propose bounded actions against an allowlisted venue. This
//! program is the last gate: every proposal walks the ladder in
//! `docs/10-PROGRAM-SPEC.md` §3, and **every** outcome — allowed or blocked —
//! emits a receipt that commits.
//!
//! Four rules hold everywhere in this file:
//!   1. `owner_withdraw` has no state check. Active, Paused, Revoked, expired:
//!      the owner leaves.
//!   2. `unpause` is owner-only. The emergency key may pause and revoke and
//!      nothing else, because restoring operator authority is not protective.
//!   3. A gate refusal emits a `RefusalReceipt` and returns `Ok(())`. If a
//!      refusal unwound the transaction there would be no log, and the whole
//!      proof surface would be fiction. The single exception is
//!      `PostCheckFailed` after a CPI, which must revert.
//!   4. `BlockReason` discriminants are append-only: 0–10 are exactly what the
//!      predecessor `5o8E…` emitted on devnet, 11–16 are appended (ADR-004).
#![allow(unexpected_cfgs)]
#![forbid(unsafe_code)]

use anchor_lang::prelude::*;

pub mod cpi;
pub mod errors;
pub mod gates;
pub mod mark;
pub mod receipts;
pub mod state;

use crate::errors::MandateError;
use crate::gates::{Intent, MarkInput};
use crate::receipts::{ActionReceipt, OwnerAction, OwnerActionKind, RefusalReceipt};
use crate::state::{Mandate, MandateState, Policy, Registry, MAX_ADAPTERS};
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};
use markov_types::BlockReason;
use pyth_solana_receiver_sdk::price_update::PriceUpdateV2;

declare_id!("25CdYaZeB18QvUR7cTyZPgTZPNREb7t6xL8zmk1eXAU6");

/// The house book. One strategy in Gate B.
pub const BOOK_ONE: [u8; 16] = *b"BOOK_ONE\0\0\0\0\0\0\0\0";

#[program]
pub mod markov_mandate {
    use super::*;

    /// Create the single registry PDA. The admin is a documented single key on
    /// devnet; the accepted risk is written down in SECURITY.md.
    pub fn init_registry(ctx: Context<InitRegistry>) -> Result<()> {
        let r = &mut ctx.accounts.registry;
        r.admin = ctx.accounts.admin.key();
        r.global_halt = false;
        r.adapters = [Pubkey::default(); MAX_ADAPTERS];
        r.adapters_len = 0;
        r.bump = ctx.bumps.registry;
        Ok(())
    }

    /// Replace the adapter allowlist. Admin only. Adding an adapter never
    /// widens an existing mandate: gate 5 requires the venue to be in the
    /// mandate's policy **and** here.
    pub fn set_adapters(ctx: Context<AdminOnly>, adapters: Vec<Pubkey>) -> Result<()> {
        require!(adapters.len() <= MAX_ADAPTERS, MandateError::InvalidPolicy);
        let r = &mut ctx.accounts.registry;
        r.adapters = [Pubkey::default(); MAX_ADAPTERS];
        for (i, a) in adapters.iter().enumerate() {
            r.adapters[i] = *a;
        }
        r.adapters_len = adapters.len() as u8;
        Ok(())
    }

    /// The circuit. When open, every `execute_venue_action` refuses with
    /// `GlobalHalt` — and `owner_withdraw` still works.
    pub fn set_global_halt(ctx: Context<AdminOnly>, halted: bool) -> Result<()> {
        ctx.accounts.registry.global_halt = halted;
        Ok(())
    }

    pub fn create_mandate(ctx: Context<CreateMandate>, args: CreateMandateArgs) -> Result<()> {
        args.policy.validate()?;
        let now = Clock::get()?.unix_timestamp;
        require!(args.policy.expiry_ts > now, MandateError::InvalidPolicy);
        require!(
            args.policy.token_allowed(&ctx.accounts.mint.key()),
            MandateError::WrongMint
        );

        let m = &mut ctx.accounts.mandate;
        m.owner = ctx.accounts.owner.key();
        m.operator = args.operator;
        m.emergency = args.emergency;
        m.strategy_id = args.strategy_id;
        m.state = MandateState::Active;
        m.policy = args.policy;
        m.vault = ctx.accounts.vault.key();
        m.mint = ctx.accounts.mint.key();
        m.mark_account = args.mark_account;
        m.feed_id = args.feed_id;
        m.day_epoch = Mandate::utc_day(now);
        m.day_notional_used = 0;
        m.day_spend_used = 0;
        m.action_seq = 0;
        m.recent_intents = [[0u8; 32]; crate::state::RECENT_INTENTS];
        m.recent_intents_len = 0;
        m.recent_intents_next = 0;
        m.created_at = now;
        m.nonce = args.nonce;
        m.bump = ctx.bumps.mandate;
        m.vault_bump = ctx.bumps.vault;
        m.reserve = [0u8; 128];

        let clock = Clock::get()?;
        emit_cpi!(owner_action(
            OwnerActionKind::Create,
            m,
            ctx.accounts.owner.key(),
            0,
            now,
            clock.slot
        ));
        Ok(())
    }

    /// Owner tops the vault up. Legal in Active and Paused; refused once the
    /// mandate is revoked, because a revoked mandate is on its way out.
    pub fn fund(ctx: Context<Fund>, amount: u64) -> Result<()> {
        require!(amount > 0, MandateError::InvalidAmount);
        let m = &ctx.accounts.mandate;
        require!(
            m.state != MandateState::Revoked,
            MandateError::AlreadyRevoked
        );
        require!(
            m.policy.token_allowed(&ctx.accounts.mint.key()),
            MandateError::WrongMint
        );
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.key(),
                Transfer {
                    from: ctx.accounts.owner_ata.to_account_info(),
                    to: ctx.accounts.vault.to_account_info(),
                    authority: ctx.accounts.owner.to_account_info(),
                },
            ),
            amount,
        )?;
        let clock = Clock::get()?;
        emit_cpi!(owner_action(
            OwnerActionKind::Fund,
            &ctx.accounts.mandate,
            ctx.accounts.owner.key(),
            amount,
            clock.unix_timestamp,
            clock.slot
        ));
        Ok(())
    }

    /// Tighten-only. A widening amendment is a hard error, not a silent no-op,
    /// so an owner can never be told "amended" when nothing narrowed.
    pub fn amend_policy(ctx: Context<OwnerOnly>, new_policy: Policy) -> Result<()> {
        let m = &mut ctx.accounts.mandate;
        require!(
            m.state != MandateState::Revoked,
            MandateError::AlreadyRevoked
        );
        m.policy.assert_tightens(&new_policy)?;
        // Rolling counters are the mandate's, not the policy's, so they survive.
        m.policy = new_policy;
        let clock = Clock::get()?;
        emit_cpi!(owner_action(
            OwnerActionKind::Amend,
            m,
            ctx.accounts.owner.key(),
            0,
            clock.unix_timestamp,
            clock.slot
        ));
        Ok(())
    }

    /// Owner or emergency key. Active → Paused.
    pub fn pause(ctx: Context<OwnerOrEmergency>) -> Result<()> {
        let caller = ctx.accounts.caller.key();
        let m = &mut ctx.accounts.mandate;
        require!(
            m.is_owner(&caller) || m.is_emergency(&caller),
            MandateError::NotOwnerOrEmergency
        );
        require!(m.state == MandateState::Active, MandateError::NotActive);
        m.state = MandateState::Paused;
        let clock = Clock::get()?;
        emit_cpi!(owner_action(
            OwnerActionKind::Pause,
            m,
            caller,
            0,
            clock.unix_timestamp,
            clock.slot
        ));
        Ok(())
    }

    /// **Owner only.** The emergency key cannot restore operator authority.
    pub fn unpause(ctx: Context<OwnerOnly>) -> Result<()> {
        let m = &mut ctx.accounts.mandate;
        require!(m.state == MandateState::Paused, MandateError::NotPaused);
        m.state = MandateState::Active;
        let clock = Clock::get()?;
        emit_cpi!(owner_action(
            OwnerActionKind::Unpause,
            m,
            ctx.accounts.owner.key(),
            0,
            clock.unix_timestamp,
            clock.slot
        ));
        Ok(())
    }

    /// Owner or emergency key. Terminal.
    pub fn revoke(ctx: Context<OwnerOrEmergency>) -> Result<()> {
        let caller = ctx.accounts.caller.key();
        let m = &mut ctx.accounts.mandate;
        require!(
            m.is_owner(&caller) || m.is_emergency(&caller),
            MandateError::NotOwnerOrEmergency
        );
        require!(
            m.state != MandateState::Revoked,
            MandateError::AlreadyRevoked
        );
        m.state = MandateState::Revoked;
        let clock = Clock::get()?;
        emit_cpi!(owner_action(
            OwnerActionKind::Revoke,
            m,
            caller,
            0,
            clock.unix_timestamp,
            clock.slot
        ));
        Ok(())
    }

    /// **No state check, by design.** Active, Paused, Revoked, expired, after
    /// the operator key is compromised: the owner can always leave. Any code
    /// path that could disable this is a release blocker.
    pub fn owner_withdraw(ctx: Context<OwnerWithdraw>, amount: u64) -> Result<()> {
        require!(amount > 0, MandateError::InvalidAmount);
        let m = &ctx.accounts.mandate;
        let owner = m.owner;
        let strategy_id = m.strategy_id;
        let nonce_bytes = m.nonce.to_le_bytes();
        let bump = m.bump;
        let seeds: &[&[u8]] = &[
            Mandate::SEED,
            owner.as_ref(),
            strategy_id.as_ref(),
            nonce_bytes.as_ref(),
            &[bump],
        ];
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.key(),
                Transfer {
                    from: ctx.accounts.vault.to_account_info(),
                    to: ctx.accounts.destination.to_account_info(),
                    authority: ctx.accounts.mandate.to_account_info(),
                },
                &[seeds],
            ),
            amount,
        )?;
        let clock = Clock::get()?;
        emit_cpi!(owner_action(
            OwnerActionKind::Withdraw,
            &ctx.accounts.mandate,
            ctx.accounts.owner.key(),
            amount,
            clock.unix_timestamp,
            clock.slot
        ));
        Ok(())
    }

    /// Rent back to the owner once the vault is empty.
    pub fn close_mandate(ctx: Context<CloseMandate>) -> Result<()> {
        require!(ctx.accounts.vault.amount == 0, MandateError::VaultNotEmpty);
        let clock = Clock::get()?;
        emit_cpi!(owner_action(
            OwnerActionKind::Close,
            &ctx.accounts.mandate,
            ctx.accounts.owner.key(),
            0,
            clock.unix_timestamp,
            clock.slot
        ));
        let m = &ctx.accounts.mandate;
        let owner = m.owner;
        let strategy_id = m.strategy_id;
        let nonce_bytes = m.nonce.to_le_bytes();
        let seeds: &[&[u8]] = &[
            Mandate::SEED,
            owner.as_ref(),
            strategy_id.as_ref(),
            nonce_bytes.as_ref(),
            &[m.bump],
        ];
        token::close_account(CpiContext::new_with_signer(
            ctx.accounts.token_program.key(),
            token::CloseAccount {
                account: ctx.accounts.vault.to_account_info(),
                destination: ctx.accounts.owner.to_account_info(),
                authority: ctx.accounts.mandate.to_account_info(),
            },
            &[seeds],
        ))
    }

    /// The operator proposes; the ladder decides; a receipt records it either
    /// way. Gates 1–12 are in `gates::evaluate`; 13 and 14 are here, around
    /// the CPI.
    pub fn execute_venue_action<'info>(
        ctx: Context<'info, ExecuteVenueAction<'info>>,
        intent: Intent,
    ) -> Result<()> {
        let clock = Clock::get()?;
        let now = clock.unix_timestamp;
        ctx.accounts.mandate.rollover(now);

        let venue = ctx.accounts.venue_program.key();
        let mint = ctx.accounts.mint.key();
        let signer = ctx.accounts.operator.key();

        // The mark is read before the ladder because gates 11 and 12 both need
        // it; a mark that cannot be bound is `None`, never a fallback price.
        let bound = mark::read_bound_mark(
            &ctx.accounts.price_update,
            &ctx.accounts.mandate.mark_account,
            &ctx.accounts.mandate.feed_id,
            ctx.accounts.mandate.policy.max_mark_age_secs,
            &clock,
        )
        .ok();
        let mark_input = bound.as_ref().map(|b| MarkInput {
            price_e6: b.price_e6(),
        });

        let seq = ctx.accounts.mandate.action_seq.saturating_add(1);
        if let Some(refusal) = gates::evaluate(
            &ctx.accounts.registry,
            &ctx.accounts.mandate,
            &signer,
            &venue,
            &mint,
            &intent,
            mark_input,
            now,
        ) {
            // A refusal is a result, not an error: record it and commit.
            ctx.accounts.mandate.action_seq = seq;
            emit_cpi!(RefusalReceipt {
                seq,
                intent_id: intent.intent_id,
                mandate: ctx.accounts.mandate.key(),
                operator: signer,
                strategy_id: ctx.accounts.mandate.strategy_id,
                venue,
                action: intent.action as u8,
                notional: intent.notional,
                reason: refusal.reason,
                gate_index: refusal.gate_index,
                forced: intent.forced,
                ts: now,
                slot: clock.slot,
            });
            return Ok(());
        }

        let bound = bound.ok_or(MandateError::PostCheckFailed)?;
        let before = snapshot(&ctx.accounts.vault, &ctx.accounts.mandate);

        let owner = ctx.accounts.mandate.owner;
        let strategy_id = ctx.accounts.mandate.strategy_id;
        let nonce_bytes = ctx.accounts.mandate.nonce.to_le_bytes();
        let bump = ctx.accounts.mandate.bump;
        let seeds: &[&[u8]] = &[
            Mandate::SEED,
            owner.as_ref(),
            strategy_id.as_ref(),
            nonce_bytes.as_ref(),
            &[bump],
        ];
        let args = cpi::venue::VenueExecuteArgs {
            action: intent.action as u8,
            market: intent.market,
            side: intent.side as u8,
            notional: intent.notional,
            limit_price: intent.limit_price,
            max_slippage_bps: intent.max_slippage_bps,
        };
        let metas = ctx
            .remaining_accounts
            .iter()
            .map(|a| AccountMeta {
                pubkey: a.key(),
                is_signer: a.key() == ctx.accounts.mandate.key(),
                is_writable: a.is_writable,
            })
            .collect::<Vec<_>>();
        let mut infos = ctx.remaining_accounts.to_vec();
        infos.push(ctx.accounts.mandate.to_account_info());
        infos.push(ctx.accounts.venue_program.to_account_info());

        // Gate 13: the venue said no. That is a refusal, not a crash.
        if cpi::venue::venue_execute(
            &ctx.accounts.venue_program.to_account_info(),
            &infos,
            metas,
            &args,
            &[seeds],
        )
        .is_err()
        {
            ctx.accounts.mandate.action_seq = seq;
            emit_cpi!(RefusalReceipt {
                seq,
                intent_id: intent.intent_id,
                mandate: ctx.accounts.mandate.key(),
                operator: signer,
                strategy_id: ctx.accounts.mandate.strategy_id,
                venue,
                action: intent.action as u8,
                notional: intent.notional,
                reason: BlockReason::VenueRejected,
                gate_index: 13,
                forced: intent.forced,
                ts: now,
                slot: clock.slot,
            });
            return Ok(());
        }

        // The venue must say what it filled. A CPI returns no value, so this
        // reads the fill it reported; if there is none, the program refuses
        // rather than writing the limit price into a receipt and calling it a
        // fill (ADR-007). Gate 13 covers this: the venue did not deliver a
        // result we can describe.
        let Some(fill) = cpi::venue::reported_fill(&venue) else {
            ctx.accounts.mandate.action_seq = seq;
            emit_cpi!(RefusalReceipt {
                seq,
                intent_id: intent.intent_id,
                mandate: ctx.accounts.mandate.key(),
                operator: signer,
                strategy_id: ctx.accounts.mandate.strategy_id,
                venue,
                action: intent.action as u8,
                notional: intent.notional,
                reason: BlockReason::VenueRejected,
                gate_index: 13,
                forced: intent.forced,
                ts: now,
                slot: clock.slot,
            });
            return Ok(());
        };
        // A venue may fill less than asked, never more.
        require!(fill.notional <= intent.notional, MandateError::PostCheckFailed);

        // Gate 14: the only refusal allowed to be an `Err`. A state we cannot
        // describe is a state we do not keep.
        ctx.accounts.vault.reload()?;
        let after = snapshot(&ctx.accounts.vault, &ctx.accounts.mandate);
        require!(
            cpi::venue::post_checks_pass(&before, &after, fill.notional),
            MandateError::PostCheckFailed
        );

        let m = &mut ctx.accounts.mandate;
        m.action_seq = seq;
        // The counter spends what was actually transacted, which gate 9 has
        // already bounded because a fill can only be smaller than the intent.
        m.day_notional_used = m
            .day_notional_used
            .checked_add(fill.notional)
            .ok_or(MandateError::Math)?;
        m.day_spend_used = m
            .day_spend_used
            .checked_add(intent.spend)
            .ok_or(MandateError::Math)?;
        m.remember_intent(intent.intent_id);

        emit_cpi!(ActionReceipt {
            seq,
            intent_id: intent.intent_id,
            mandate: m.key(),
            owner: m.owner,
            operator: signer,
            strategy_id: m.strategy_id,
            venue,
            market: intent.market,
            action: intent.action as u8,
            side: intent.side as u8,
            notional: fill.notional,
            // The price the venue reported filling at — never the limit.
            fill_price: fill.price,
            fee: fill.fee,
            mark_price: bound.price_e6(),
            mark_publish_time: bound.publish_time,
            spend: intent.spend,
            forced: intent.forced,
            ts: now,
            slot: clock.slot,
            // Enforced off-chain in v0 (ADR-05); the page must say so.
            net_delta_usd_e6: 0,
            gross_usd_e6: 0,
        });
        Ok(())
    }
}

fn snapshot(vault: &TokenAccount, m: &Mandate) -> cpi::venue::VaultSnapshot {
    let mut policy_bytes = Vec::new();
    // Serialising cannot fail for a fixed-size struct; a failure hashes to
    // empty, which differs from any real policy and so fails the post-check.
    let _ = m.policy.serialize(&mut policy_bytes);
    cpi::venue::VaultSnapshot {
        balance: vault.amount,
        owner: vault.owner,
        has_delegate: vault.delegate.is_some(),
        mandate_owner: m.owner,
        mandate_operator: m.operator,
        policy_hash: solana_sha256_hasher::hash(&policy_bytes).to_bytes(),
    }
}

/// Build the owner receipt. The caller emits it with `emit_cpi!` so every
/// receipt in this program reaches the indexer by the same path.
fn owner_action(
    kind: OwnerActionKind,
    m: &Account<'_, Mandate>,
    actor: Pubkey,
    amount: u64,
    now: i64,
    slot: u64,
) -> OwnerAction {
    OwnerAction {
        kind,
        mandate: m.key(),
        owner: m.owner,
        actor,
        strategy_id: m.strategy_id,
        mint: m.mint,
        amount,
        ts: now,
        slot,
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug)]
pub struct CreateMandateArgs {
    pub operator: Pubkey,
    pub emergency: Pubkey,
    pub strategy_id: [u8; 16],
    pub nonce: u64,
    pub policy: Policy,
    /// The Pyth `PriceUpdateV2` account this mandate marks against.
    pub mark_account: Pubkey,
    pub feed_id: [u8; 32],
}

#[derive(Accounts)]
pub struct InitRegistry<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        init,
        payer = admin,
        space = 8 + Registry::INIT_SPACE,
        seeds = [Registry::SEED],
        bump
    )]
    pub registry: Box<Account<'info, Registry>>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AdminOnly<'info> {
    pub admin: Signer<'info>,
    #[account(
        mut,
        seeds = [Registry::SEED],
        bump = registry.bump,
        constraint = registry.admin == admin.key() @ MandateError::NotRegistryAdmin
    )]
    pub registry: Box<Account<'info, Registry>>,
}

#[derive(Accounts)]
#[instruction(args: CreateMandateArgs)]
#[event_cpi]
pub struct CreateMandate<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    pub mint: Box<Account<'info, Mint>>,
    #[account(
        init,
        payer = owner,
        space = 8 + Mandate::INIT_SPACE,
        seeds = [
            Mandate::SEED,
            owner.key().as_ref(),
            args.strategy_id.as_ref(),
            args.nonce.to_le_bytes().as_ref()
        ],
        bump
    )]
    pub mandate: Box<Account<'info, Mandate>>,
    #[account(
        init,
        payer = owner,
        seeds = [Mandate::VAULT_SEED, mandate.key().as_ref()],
        bump,
        token::mint = mint,
        token::authority = mandate
    )]
    pub vault: Box<Account<'info, TokenAccount>>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[event_cpi]
pub struct Fund<'info> {
    pub owner: Signer<'info>,
    #[account(
        mut,
        has_one = owner @ MandateError::NotOwner,
        has_one = vault @ MandateError::WrongVault,
        has_one = mint @ MandateError::WrongMint
    )]
    pub mandate: Box<Account<'info, Mandate>>,
    pub mint: Box<Account<'info, Mint>>,
    #[account(mut, token::mint = mint, token::authority = owner)]
    pub owner_ata: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub vault: Box<Account<'info, TokenAccount>>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
#[event_cpi]
pub struct OwnerOnly<'info> {
    pub owner: Signer<'info>,
    #[account(mut, has_one = owner @ MandateError::NotOwner)]
    pub mandate: Box<Account<'info, Mandate>>,
}

#[derive(Accounts)]
#[event_cpi]
pub struct OwnerOrEmergency<'info> {
    pub caller: Signer<'info>,
    #[account(mut)]
    pub mandate: Box<Account<'info, Mandate>>,
}

#[derive(Accounts)]
#[event_cpi]
pub struct OwnerWithdraw<'info> {
    pub owner: Signer<'info>,
    #[account(
        mut,
        has_one = owner @ MandateError::NotOwner,
        has_one = vault @ MandateError::WrongVault
    )]
    pub mandate: Box<Account<'info, Mandate>>,
    #[account(mut)]
    pub vault: Box<Account<'info, TokenAccount>>,
    /// The owner's own token account. Never the operator's.
    #[account(mut, token::mint = mandate.mint, token::authority = owner)]
    pub destination: Box<Account<'info, TokenAccount>>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
#[event_cpi]
pub struct CloseMandate<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(
        mut,
        close = owner,
        has_one = owner @ MandateError::NotOwner,
        has_one = vault @ MandateError::WrongVault
    )]
    pub mandate: Box<Account<'info, Mandate>>,
    #[account(mut)]
    pub vault: Box<Account<'info, TokenAccount>>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
#[event_cpi]
pub struct ExecuteVenueAction<'info> {
    /// The house agent. Propose-only: it never signs to the venue, and it
    /// cannot reach any owner instruction.
    pub operator: Signer<'info>,
    #[account(seeds = [Registry::SEED], bump = registry.bump)]
    pub registry: Box<Account<'info, Registry>>,
    #[account(mut, has_one = vault @ MandateError::WrongVault, has_one = mint @ MandateError::WrongMint)]
    pub mandate: Box<Account<'info, Mandate>>,
    pub mint: Box<Account<'info, Mint>>,
    #[account(mut)]
    pub vault: Box<Account<'info, TokenAccount>>,
    /// The Pyth price update. Anchor checks it is owned by the receiver
    /// program; `mark::read_bound_mark` checks it is *this mandate's* account,
    /// carries the right feed and is fully verified and fresh.
    pub price_update: Box<Account<'info, PriceUpdateV2>>,
    /// CHECK: gate 5 refuses unless this program is in the policy and in the
    /// registry; it is only ever the target of a CPI signed by the mandate PDA.
    pub venue_program: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
}
