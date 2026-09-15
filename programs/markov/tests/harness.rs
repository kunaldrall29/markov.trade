//! LiteSVM harness for the control-plane program. The built `.so` is loaded
//! from `target/deploy/markov.so`; the instructions sysvar is populated by
//! the SVM from the transaction itself, so adjacency is tested for real.
#![allow(
    dead_code,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::new_without_default,
    clippy::result_large_err
)]

use anchor_lang::{InstructionData, ToAccountMetas};
use litesvm::LiteSVM;
use markov::state::*;
use solana_account::Account;
use solana_instruction::{AccountMeta, Instruction};
use solana_keypair::Keypair;
use solana_message::{Message, VersionedMessage};
use solana_pubkey::Pubkey;
use solana_signer::Signer;
use solana_transaction::versioned::VersionedTransaction;

pub const NOW: i64 = 1_789_430_400; // 2026-09-15T00:00Z, a Tuesday
pub const SLOT: u64 = 500_000_000;
pub const MARKET: [u8; 16] = *b"SOL-PERP\0\0\0\0\0\0\0\0";
pub const VENUE_ID: u8 = 1;
pub const USD: u64 = 1_000_000;

/// Programs that stand in for a venue, the delegation `collect` and the swap
/// in adjacency tests. They are real builtins the SVM executes, so the
/// transaction genuinely carries their instructions: the program under test
/// only ever compares program ids.
pub const SYSTEM_PROGRAM: Pubkey = solana_pubkey::pubkey!("11111111111111111111111111111111");
pub const COMPUTE_BUDGET_PROGRAM: Pubkey =
    solana_pubkey::pubkey!("ComputeBudget111111111111111111111111111111");
/// The venue and the delegation `collect` stand-ins are `test_noop`, a real
/// executable program; the swap stand-in is the compute-budget builtin. (The
/// system program cannot stand in: its id is the zero pubkey, which the
/// config treats as unset.)
pub const VENUE_PROGRAM: Pubkey = test_noop::ID;
pub const INSTRUCTIONS_SYSVAR: Pubkey =
    solana_pubkey::pubkey!("Sysvar1nstructions1111111111111111111111111");

pub struct Env {
    pub svm: LiteSVM,
    pub program: Pubkey,
    pub admin: Keypair,
    pub owner: Keypair,
    pub actor: Keypair,
    pub stranger: Keypair,
    pub usdc_mint: Pubkey,
    pub config: Pubkey,
    pub account: Pubkey,
    pub event_authority: Pubkey,
    pub owner_usdc: Pubkey,
}

fn token_data(mint: &Pubkey, owner: &Pubkey, amount: u64) -> Vec<u8> {
    let mut b = vec![0u8; 165];
    b[0..32].copy_from_slice(mint.as_ref());
    b[32..64].copy_from_slice(owner.as_ref());
    b[64..72].copy_from_slice(&amount.to_le_bytes());
    b[108] = 1;
    b
}

pub fn set_clock(svm: &mut LiteSVM, unix_timestamp: i64, slot: u64) {
    let mut clock: anchor_lang::solana_program::clock::Clock = svm.get_sysvar();
    clock.unix_timestamp = unix_timestamp;
    clock.slot = slot;
    svm.set_sysvar(&clock);
}

pub fn caps() -> GlobalCaps {
    GlobalCaps {
        per_account_gross_notional_usd: 2_000 * USD,
        per_position_leverage_bps: 20_000,
        global_daily_invest_spend_usd: 2_000 * USD,
        invest_min_per_execution_usd: 5 * USD,
        invest_max_per_execution_usd: 100 * USD,
        freshness_slots: 2,
    }
}

pub fn mandate_fields() -> MandateFields {
    MandateFields {
        max_leverage_bps: 20_000,
        max_notional_usd: 1_000 * USD,
        min_safety_buffer_bps: 2_000,
        max_daily_loss_usd: 50 * USD,
        approved_markets: vec![MARKET, *b"BTC-PERP\0\0\0\0\0\0\0\0"],
        tier_caps_bps: [20_000, 20_000, 10_000, 5_000, 0],
        allowed_venues: 1 << VENUE_ID,
    }
}

pub fn invest_fields(mint: Pubkey) -> InvestMandateFields {
    InvestMandateFields {
        allowlist: vec![mint],
        per_period_budget_usd: 25 * USD,
        period_seconds: 7 * 86_400,
        monthly_ceiling_usd: 100 * USD,
        reserve_floor_usd: 50 * USD,
        max_cost_bps: 40,
        max_reference_deviation_bps: 100,
        single_asset_cap_bps: 5_000,
        execution_mode: InvestExecutionMode::AlwaysOn,
        window_start_utc: 0,
        window_end_utc: 86_400,
        days_mask: 0x7f,
    }
}

pub fn ok_checks() -> Vec<Check> {
    vec![
        Check {
            rule: rule::LEVERAGE,
            observed: 15_000,
            limit: 0,
            pass: true,
        },
        Check {
            rule: rule::NOTIONAL,
            observed: 500 * USD as i64,
            limit: 0,
            pass: true,
        },
        Check {
            rule: rule::SAFETY_BUFFER,
            observed: 3_000,
            limit: 0,
            pass: true,
        },
        Check {
            rule: rule::DAILY_LOSS,
            observed: 5 * USD as i64,
            limit: 0,
            pass: true,
        },
        Check {
            rule: rule::SLIPPAGE,
            observed: 12,
            limit: 20,
            pass: true,
        },
    ]
}

pub fn decision_args(
    request_id: u128,
    decision: Decision,
    checks: Vec<Check>,
) -> markov::DecisionArgs {
    markov::DecisionArgs {
        request_id,
        kind: ReceiptKind::TradeOpen,
        decision,
        reason_code: reason::NONE,
        checks,
        data_slot: SLOT,
        venue_id: VENUE_ID,
        market_id: MARKET,
        route_snapshot_hash: [1u8; 32],
        pre_state_hash: [2u8; 32],
        notional_usd: 500 * USD,
        day_epoch: NOW.div_euclid(86_400),
    }
}

/// The stand-in venue leg: a real instruction of a real program.
pub fn venue_leg(_from: &Pubkey) -> Instruction {
    Instruction {
        program_id: test_noop::ID,
        accounts: vec![],
        data: test_noop::instruction::Noop { tag: 1 }.data(),
    }
}

/// The stand-in collect leg: the same program, a different tag.
pub fn collect_leg() -> Instruction {
    Instruction {
        program_id: test_noop::ID,
        accounts: vec![],
        data: test_noop::instruction::Noop { tag: 2 }.data(),
    }
}

/// The stand-in swap leg: a compute-budget instruction (heap frame).
pub fn swap_leg() -> Instruction {
    Instruction {
        program_id: COMPUTE_BUDGET_PROGRAM,
        accounts: vec![],
        data: vec![1, 0x00, 0x00, 0x01, 0x00],
    }
}

/// A leg from a program that is not the venue for `VENUE_ID`.
pub fn wrong_leg() -> Instruction {
    swap_leg()
}

impl Env {
    pub fn new() -> Env {
        let mut svm = LiteSVM::new();
        let program = markov::ID;
        let so = include_bytes!("../../../target/deploy/markov.so");
        svm.add_program(program, so).unwrap();
        svm.add_program(
            test_noop::ID,
            include_bytes!("../../../target/deploy/test_noop.so"),
        )
        .unwrap();
        let admin = Keypair::new();
        let owner = Keypair::new();
        let actor = Keypair::new();
        let stranger = Keypair::new();
        for k in [&admin, &owner, &actor, &stranger] {
            svm.airdrop(&k.pubkey(), 10_000_000_000).unwrap();
        }
        set_clock(&mut svm, NOW, SLOT);
        let usdc_mint = Pubkey::new_unique();
        let (config, _) = Pubkey::find_program_address(&[GlobalConfig::SEED], &program);
        let (account, _) =
            Pubkey::find_program_address(&[MarkovAccount::SEED, owner.pubkey().as_ref()], &program);
        let (event_authority, _) = Pubkey::find_program_address(&[b"__event_authority"], &program);
        let owner_usdc = Pubkey::new_unique();
        svm.set_account(
            owner_usdc,
            Account {
                lamports: 2_039_280,
                data: token_data(&usdc_mint, &owner.pubkey(), 200 * USD),
                owner: anchor_spl::token::ID,
                executable: false,
                rent_epoch: 0,
            },
        )
        .unwrap();
        Env {
            svm,
            program,
            admin,
            owner,
            actor,
            stranger,
            usdc_mint,
            config,
            account,
            event_authority,
            owner_usdc,
        }
    }

    pub fn send(
        &mut self,
        ixs: Vec<Instruction>,
        payer: &Keypair,
        signers: &[&Keypair],
    ) -> Result<litesvm::types::TransactionMetadata, litesvm::types::FailedTransactionMetadata>
    {
        // A fresh blockhash per send: identical instructions re-sent under the
        // same hash would be deduplicated as "already processed".
        self.svm.expire_blockhash();
        let blockhash = self.svm.latest_blockhash();
        let msg = Message::new_with_blockhash(&ixs, Some(&payer.pubkey()), &blockhash);
        let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), signers).unwrap();
        self.svm.send_transaction(tx)
    }

    pub fn ix(&self, accounts: impl ToAccountMetas, data: impl InstructionData) -> Instruction {
        Instruction {
            program_id: self.program,
            accounts: accounts.to_account_metas(None),
            data: data.data(),
        }
    }

    pub fn init_config(&mut self) {
        let ix = self.ix(
            markov::accounts::InitializeConfig {
                admin: self.admin.pubkey(),
                config: self.config,
                system_program: SYSTEM_PROGRAM,
            },
            markov::instruction::InitializeConfig {
                caps: caps(),
                usdc_mint: self.usdc_mint,
            },
        );
        let admin = self.admin.insecure_clone();
        self.send(vec![ix], &admin, &[&admin]).unwrap();
        let ix = self.ix(
            markov::accounts::AdminOnly {
                admin: self.admin.pubkey(),
                config: self.config,
            },
            markov::instruction::SetVenueProgram {
                venue_id: VENUE_ID,
                program: VENUE_PROGRAM,
            },
        );
        self.send(vec![ix], &admin, &[&admin]).unwrap();
        let ix = self.ix(
            markov::accounts::AdminOnly {
                admin: self.admin.pubkey(),
                config: self.config,
            },
            markov::instruction::SetInvestPrograms {
                collect: test_noop::ID,
                swap: COMPUTE_BUDGET_PROGRAM,
            },
        );
        self.send(vec![ix], &admin, &[&admin]).unwrap();
    }

    pub fn admin_ix(&self, data: impl InstructionData) -> Instruction {
        self.ix(
            markov::accounts::AdminOnly {
                admin: self.admin.pubkey(),
                config: self.config,
            },
            data,
        )
    }

    pub fn create_account(&mut self) {
        let ix = self.ix(
            markov::accounts::CreateAccount {
                owner: self.owner.pubkey(),
                account: self.account,
                system_program: SYSTEM_PROGRAM,
            },
            markov::instruction::CreateAccount {
                execution_mode: ExecutionMode::OwnerSigned,
            },
        );
        let owner = self.owner.insecure_clone();
        self.send(vec![ix], &owner, &[&owner]).unwrap();
    }

    pub fn mandate_pda(&self, version: u32) -> Pubkey {
        Pubkey::find_program_address(
            &[Mandate::SEED, self.account.as_ref(), &version.to_le_bytes()],
            &self.program,
        )
        .0
    }
    pub fn invest_pda(&self, version: u32) -> Pubkey {
        Pubkey::find_program_address(
            &[
                InvestMandate::SEED,
                self.account.as_ref(),
                &version.to_le_bytes(),
            ],
            &self.program,
        )
        .0
    }
    pub fn permission_pda(&self, actor: &Pubkey) -> Pubkey {
        Pubkey::find_program_address(
            &[Permission::SEED, self.account.as_ref(), actor.as_ref()],
            &self.program,
        )
        .0
    }
    pub fn receipt_pda(&self, request_id: u128) -> Pubkey {
        Pubkey::find_program_address(
            &[
                ActionReceipt::SEED,
                self.account.as_ref(),
                &request_id.to_le_bytes(),
            ],
            &self.program,
        )
        .0
    }
    pub fn ledger_pda(&self, day_epoch: i64) -> Pubkey {
        Pubkey::find_program_address(
            &[
                DailyLedger::SEED,
                self.account.as_ref(),
                &day_epoch.to_le_bytes(),
            ],
            &self.program,
        )
        .0
    }

    pub fn set_mandate(&mut self, version: u32, fields: MandateFields) -> Result<(), String> {
        let ix = self.ix(
            markov::accounts::SetMandate {
                owner: self.owner.pubkey(),
                account: self.account,
                mandate: self.mandate_pda(version),
                system_program: SYSTEM_PROGRAM,
                event_authority: self.event_authority,
                program: self.program,
            },
            markov::instruction::SetMandate { version, fields },
        );
        let owner = self.owner.insecure_clone();
        self.send(vec![ix], &owner, &[&owner])
            .map(|_| ())
            .map_err(|e| meta_err(&e))
    }

    pub fn set_invest_mandate(
        &mut self,
        version: u32,
        fields: InvestMandateFields,
    ) -> Result<(), String> {
        let ix = self.ix(
            markov::accounts::SetInvestMandate {
                owner: self.owner.pubkey(),
                account: self.account,
                invest_mandate: self.invest_pda(version),
                system_program: SYSTEM_PROGRAM,
                event_authority: self.event_authority,
                program: self.program,
            },
            markov::instruction::SetInvestMandate { version, fields },
        );
        let owner = self.owner.insecure_clone();
        self.send(vec![ix], &owner, &[&owner])
            .map(|_| ())
            .map_err(|e| meta_err(&e))
    }

    pub fn set_permission(
        &mut self,
        actor: Pubkey,
        scopes: u32,
        per_action: u64,
        daily: u64,
        expires_slot: u64,
    ) -> Result<(), String> {
        let ix = self.ix(
            markov::accounts::SetPermission {
                owner: self.owner.pubkey(),
                account: self.account,
                permission: self.permission_pda(&actor),
                system_program: SYSTEM_PROGRAM,
            },
            markov::instruction::SetPermission {
                actor,
                kind: ActorKind::McpClient,
                scopes,
                per_action_cap_usd: per_action,
                daily_cap_usd: daily,
                expires_slot,
            },
        );
        let owner = self.owner.insecure_clone();
        self.send(vec![ix], &owner, &[&owner])
            .map(|_| ())
            .map_err(|e| meta_err(&e))
    }

    pub fn owner_verb(&mut self, data: impl InstructionData) -> Result<(), String> {
        let ix = self.ix(
            markov::accounts::OwnerOnly {
                owner: self.owner.pubkey(),
                account: self.account,
            },
            data,
        );
        let owner = self.owner.insecure_clone();
        self.send(vec![ix], &owner, &[&owner])
            .map(|_| ())
            .map_err(|e| meta_err(&e))
    }

    /// `record_decision` from `signer` (owner or actor), with the given trailing instructions.
    pub fn record_decision_ix(&self, signer: &Pubkey, args: &markov::DecisionArgs) -> Instruction {
        let is_owner = *signer == self.owner.pubkey();
        let mut metas = markov::accounts::RecordDecision {
            signer: *signer,
            config: self.config,
            account: self.account,
            mandate: self.mandate_pda(self.active_mandate_version()),
            permission: if is_owner {
                None
            } else {
                Some(self.permission_pda(signer))
            },
            receipt: self.receipt_pda(args.request_id),
            ledger: self.ledger_pda(args.day_epoch),
            instructions: INSTRUCTIONS_SYSVAR,
            system_program: SYSTEM_PROGRAM,
            event_authority: self.event_authority,
            program: self.program,
        }
        .to_account_metas(None);
        // Anchor encodes a `None` optional account as the program id, non-writable.
        for m in metas.iter_mut() {
            if m.pubkey == self.program && m.is_writable {
                m.is_writable = false;
            }
        }
        Instruction {
            program_id: self.program,
            accounts: metas,
            data: markov::instruction::RecordDecision { args: args.clone() }.data(),
        }
    }

    pub fn active_mandate_version(&self) -> u32 {
        let acc = self.svm.get_account(&self.account).unwrap();
        let a: MarkovAccount =
            anchor_lang::AccountDeserialize::try_deserialize(&mut acc.data.as_slice()).unwrap();
        a.active_mandate_version
    }

    pub fn receipt(&self, request_id: u128) -> Option<ActionReceipt> {
        let acc = self.svm.get_account(&self.receipt_pda(request_id))?;
        if acc.data.is_empty() {
            return None;
        }
        anchor_lang::AccountDeserialize::try_deserialize(&mut acc.data.as_slice()).ok()
    }

    pub fn permission(&self, actor: &Pubkey) -> Permission {
        let acc = self.svm.get_account(&self.permission_pda(actor)).unwrap();
        anchor_lang::AccountDeserialize::try_deserialize(&mut acc.data.as_slice()).unwrap()
    }

    pub fn invest_mandate(&self, version: u32) -> InvestMandate {
        let acc = self.svm.get_account(&self.invest_pda(version)).unwrap();
        anchor_lang::AccountDeserialize::try_deserialize(&mut acc.data.as_slice()).unwrap()
    }

    pub fn ledger(&self, day_epoch: i64) -> Option<DailyLedger> {
        let acc = self.svm.get_account(&self.ledger_pda(day_epoch))?;
        anchor_lang::AccountDeserialize::try_deserialize(&mut acc.data.as_slice()).ok()
    }

    /// Standard setup: config, account, mandate v1, invest mandate v1.
    pub fn ready() -> Env {
        let mut e = Env::new();
        e.init_config();
        e.create_account();
        e.set_mandate(1, mandate_fields()).unwrap();
        let mint = e.usdc_mint;
        e.set_invest_mandate(1, invest_fields(mint)).unwrap();
        e
    }
}

pub fn err_contains(r: &Result<(), String>, needle: &str) -> bool {
    matches!(r, Err(e) if e.contains(needle))
}

pub fn meta_err(e: &litesvm::types::FailedTransactionMetadata) -> String {
    format!("{} :: {}", e.err, e.meta.logs.join(" | "))
}

pub fn writable(pubkey: Pubkey) -> AccountMeta {
    AccountMeta::new(pubkey, false)
}
