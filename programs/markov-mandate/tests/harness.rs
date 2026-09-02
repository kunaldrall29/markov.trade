//! Shared LiteSVM harness for the mandate tests.
//!
//! Accounts that the program does not create (mints, the owner's token
//! account, the Pyth price update) are written straight into the SVM, so the
//! tests exercise the program rather than a pile of setup transactions.
#![allow(dead_code, clippy::unwrap_used, clippy::expect_used)]

use anchor_lang::{Discriminator, InstructionData, ToAccountMetas};
use litesvm::LiteSVM;
use markov_mandate::state::{action_bits, Mandate, Policy, Registry};
use solana_account::Account;
use solana_instruction::Instruction;
use solana_keypair::Keypair;
use solana_message::{Message, VersionedMessage};
use solana_pubkey::Pubkey;
use solana_signer::Signer;
use solana_transaction::versioned::VersionedTransaction;

pub const PYTH_RECEIVER: Pubkey =
    solana_pubkey::pubkey!("rec5EKMGg6MxZYaMdyBfgwp4d5rB9T1VQH5pJv5LtFJ");
pub const FEED_ID: [u8; 32] = [0xef; 32];
/// A Wednesday, so the UTC-day rollover tests are readable.
pub const NOW: i64 = 1_788_400_000;
pub const MARK_PRICE_E6: u64 = 100_000_000;

pub struct Env {
    pub svm: LiteSVM,
    pub program: Pubkey,
    pub venue: Pubkey,
    pub payer: Keypair,
    pub owner: Keypair,
    pub operator: Keypair,
    pub emergency: Keypair,
    pub stranger: Keypair,
    pub mint: Pubkey,
    pub mandate: Pubkey,
    pub vault: Pubkey,
    pub registry: Pubkey,
    pub owner_ata: Pubkey,
    pub price_update: Pubkey,
    pub event_authority: Pubkey,
}

/// SPL Mint, packed by hand (82 bytes). Hand-rolled on purpose: the published
/// `spl-token` crate and Anchor disagree about the address newtype, and the
/// layout is fixed and public.
fn mint_data(authority: &Pubkey, decimals: u8) -> Vec<u8> {
    let mut b = vec![0u8; 82];
    b[0..4].copy_from_slice(&1u32.to_le_bytes()); // COption::Some
    b[4..36].copy_from_slice(authority.as_ref());
    b[36..44].copy_from_slice(&1_000_000_000u64.to_le_bytes()); // supply
    b[44] = decimals;
    b[45] = 1; // is_initialized
    b[46..50].copy_from_slice(&0u32.to_le_bytes()); // freeze_authority: None
    b
}

/// SPL token account, packed by hand (165 bytes).
fn token_data(mint: &Pubkey, owner: &Pubkey, amount: u64) -> Vec<u8> {
    let mut b = vec![0u8; 165];
    b[0..32].copy_from_slice(mint.as_ref());
    b[32..64].copy_from_slice(owner.as_ref());
    b[64..72].copy_from_slice(&amount.to_le_bytes());
    b[72..76].copy_from_slice(&0u32.to_le_bytes()); // delegate: None
    b[108] = 1; // AccountState::Initialized
    b[109..113].copy_from_slice(&0u32.to_le_bytes()); // is_native: None
    b[121..129].copy_from_slice(&0u64.to_le_bytes()); // delegated_amount
    b[129..133].copy_from_slice(&0u32.to_le_bytes()); // close_authority: None
    b
}

/// The `amount` field of a packed SPL token account.
fn token_amount_of(data: &[u8]) -> u64 {
    u64::from_le_bytes(data[64..72].try_into().unwrap())
}

/// A Pyth `PriceUpdateV2`, fully verified, published `age_secs` ago.
pub fn price_update_data(feed_id: [u8; 32], price: i64, publish_time: i64, slot: u64) -> Vec<u8> {
    let mut d = pyth_solana_receiver_sdk::price_update::PriceUpdateV2::DISCRIMINATOR.to_vec();
    d.extend_from_slice(&[7u8; 32]); // write_authority
    d.push(1); // VerificationLevel::Full
    d.extend_from_slice(&feed_id);
    d.extend_from_slice(&price.to_le_bytes());
    d.extend_from_slice(&1_500_000u64.to_le_bytes()); // conf
    d.extend_from_slice(&(-6i32).to_le_bytes()); // exponent: price is already 1e6
    d.extend_from_slice(&publish_time.to_le_bytes());
    d.extend_from_slice(&(publish_time - 50).to_le_bytes());
    d.extend_from_slice(&price.to_le_bytes()); // ema
    d.extend_from_slice(&1_400_000u64.to_le_bytes());
    d.extend_from_slice(&slot.to_le_bytes());
    d.resize(pyth_solana_receiver_sdk::price_update::PriceUpdateV2::LEN, 0);
    d
}

pub fn set_clock(svm: &mut LiteSVM, unix_timestamp: i64) {
    let mut clock: anchor_lang::solana_program::clock::Clock = svm.get_sysvar();
    clock.unix_timestamp = unix_timestamp;
    svm.set_sysvar(&clock);
}

pub fn policy(venue: Pubkey, mint: Pubkey, expiry_ts: i64) -> Policy {
    let mut p = Policy {
        venues: [Pubkey::default(); 4],
        venues_len: 1,
        tokens: [Pubkey::default(); 4],
        tokens_len: 1,
        allowed_actions: action_bits::ALL,
        per_tx_cap: 50,
        daily_cap: 200,
        spend_per_call: 5,
        spend_daily: 20,
        max_slippage_bps: 50,
        max_mark_age_secs: 150,
        expiry_ts,
    };
    p.venues[0] = venue;
    p.tokens[0] = mint;
    p
}

impl Env {
    /// Boots the SVM, writes the mint, the owner's funded token account and a
    /// fresh mark, creates the registry with the venue allowlisted, and
    /// creates one Active mandate with `vault_amount` already in the vault.
    pub fn new(vault_amount: u64) -> Env {
        let mut svm = LiteSVM::new();
        let program = markov_mandate::ID;
        let venue = demo_perps::ID;
        svm.add_program(
            program,
            include_bytes!(concat!(env!("CARGO_TARGET_TMPDIR"), "/../deploy/markov_mandate.so")),
        )
        .unwrap();
        svm.add_program(
            venue,
            include_bytes!(concat!(env!("CARGO_TARGET_TMPDIR"), "/../deploy/demo_perps.so")),
        )
        .unwrap();
        set_clock(&mut svm, NOW);

        let payer = Keypair::new();
        let owner = Keypair::new();
        let operator = Keypair::new();
        let emergency = Keypair::new();
        let stranger = Keypair::new();
        for k in [&payer, &owner, &operator, &emergency, &stranger] {
            svm.airdrop(&k.pubkey(), 10_000_000_000).unwrap();
        }

        let mint = Pubkey::new_unique();
        svm.set_account(
            mint,
            Account {
                lamports: 10_000_000,
                data: mint_data(&payer.pubkey(), 6),
                owner: anchor_spl::token::ID,
                executable: false,
                rent_epoch: 0,
            },
        )
        .unwrap();

        let owner_ata = Pubkey::new_unique();
        svm.set_account(
            owner_ata,
            Account {
                lamports: 10_000_000,
                data: token_data(&mint, &owner.pubkey(), 1_000_000),
                owner: anchor_spl::token::ID,
                executable: false,
                rent_epoch: 0,
            },
        )
        .unwrap();

        let price_update = Pubkey::new_unique();
        svm.set_account(
            price_update,
            Account {
                lamports: 10_000_000,
                data: price_update_data(FEED_ID, MARK_PRICE_E6 as i64, NOW - 10, 1),
                owner: PYTH_RECEIVER,
                executable: false,
                rent_epoch: 0,
            },
        )
        .unwrap();

        let (registry, _) = Pubkey::find_program_address(&[Registry::SEED], &program);
        let (event_authority, _) = Pubkey::find_program_address(&[b"__event_authority"], &program);
        let strategy_id = markov_mandate::BOOK_ONE;
        let nonce: u64 = 0;
        let (mandate, _) = Pubkey::find_program_address(
            &[
                Mandate::SEED,
                owner.pubkey().as_ref(),
                strategy_id.as_ref(),
                &nonce.to_le_bytes(),
            ],
            &program,
        );
        let (vault, _) =
            Pubkey::find_program_address(&[Mandate::VAULT_SEED, mandate.as_ref()], &program);

        let mut env = Env {
            svm,
            program,
            venue,
            payer,
            owner,
            operator,
            emergency,
            stranger,
            mint,
            mandate,
            vault,
            registry,
            owner_ata,
            price_update,
            event_authority,
        };

        env.init_registry();
        env.set_adapters(vec![venue]);
        env.create_mandate(policy(venue, mint, NOW + 86_400));
        if vault_amount > 0 {
            env.fund(vault_amount).expect("fund");
        }
        env
    }

    pub fn send(&mut self, ix: Instruction, signers: &[&Keypair]) -> litesvm::types::TransactionResult {
        // Two identical instructions from the same signer on the same
        // blockhash hash to the same signature, which LiteSVM rejects as
        // AlreadyProcessed. A real cluster moves the blockhash on; here we do
        // it explicitly so a test can send the same intent twice.
        self.svm.expire_blockhash();
        let blockhash = self.svm.latest_blockhash();
        let msg = Message::new_with_blockhash(&[ix], Some(&signers[0].pubkey()), &blockhash);
        let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), signers).unwrap();
        self.svm.send_transaction(tx)
    }

    fn ix(&self, data: Vec<u8>, metas: Vec<anchor_lang::solana_program::instruction::AccountMeta>) -> Instruction {
        Instruction {
            program_id: self.program,
            accounts: metas
                .into_iter()
                .map(|m| solana_instruction::AccountMeta {
                    pubkey: m.pubkey,
                    is_signer: m.is_signer,
                    is_writable: m.is_writable,
                })
                .collect(),
            data,
        }
    }

    pub fn init_registry(&mut self) {
        let ix = self.ix(
            markov_mandate::instruction::InitRegistry {}.data(),
            markov_mandate::accounts::InitRegistry {
                admin: self.payer.pubkey(),
                registry: self.registry,
                system_program: anchor_lang::solana_program::system_program::ID,
            }
            .to_account_metas(None),
        );
        let payer = self.payer.insecure_clone();
        self.send(ix, &[&payer]).expect("init_registry");
    }

    pub fn set_adapters(&mut self, adapters: Vec<Pubkey>) {
        let ix = self.ix(
            markov_mandate::instruction::SetAdapters { adapters }.data(),
            markov_mandate::accounts::AdminOnly {
                admin: self.payer.pubkey(),
                registry: self.registry,
            }
            .to_account_metas(None),
        );
        let payer = self.payer.insecure_clone();
        self.send(ix, &[&payer]).expect("set_adapters");
    }

    pub fn set_global_halt(&mut self, halted: bool) {
        let ix = self.ix(
            markov_mandate::instruction::SetGlobalHalt { halted }.data(),
            markov_mandate::accounts::AdminOnly {
                admin: self.payer.pubkey(),
                registry: self.registry,
            }
            .to_account_metas(None),
        );
        let payer = self.payer.insecure_clone();
        self.send(ix, &[&payer]).expect("set_global_halt");
    }

    pub fn create_mandate(&mut self, policy: Policy) {
        let args = markov_mandate::CreateMandateArgs {
            operator: self.operator.pubkey(),
            emergency: self.emergency.pubkey(),
            strategy_id: markov_mandate::BOOK_ONE,
            nonce: 0,
            policy,
            mark_account: self.price_update,
            feed_id: FEED_ID,
        };
        let ix = self.ix(
            markov_mandate::instruction::CreateMandate { args }.data(),
            markov_mandate::accounts::CreateMandate {
                owner: self.owner.pubkey(),
                mint: self.mint,
                mandate: self.mandate,
                vault: self.vault,
                token_program: anchor_spl::token::ID,
                system_program: anchor_lang::solana_program::system_program::ID,
                event_authority: self.event_authority,
                program: self.program,
            }
            .to_account_metas(None),
        );
        let owner = self.owner.insecure_clone();
        self.send(ix, &[&owner]).expect("create_mandate");
    }

    pub fn fund(&mut self, amount: u64) -> litesvm::types::TransactionResult {
        let ix = self.ix(
            markov_mandate::instruction::Fund { amount }.data(),
            markov_mandate::accounts::Fund {
                owner: self.owner.pubkey(),
                mandate: self.mandate,
                mint: self.mint,
                owner_ata: self.owner_ata,
                vault: self.vault,
                token_program: anchor_spl::token::ID,
                event_authority: self.event_authority,
                program: self.program,
            }
            .to_account_metas(None),
        );
        let owner = self.owner.insecure_clone();
        self.send(ix, &[&owner])
    }

    pub fn owner_only(&mut self, data: Vec<u8>, signer: &Keypair) -> litesvm::types::TransactionResult {
        let ix = self.ix(
            data,
            markov_mandate::accounts::OwnerOnly {
                owner: signer.pubkey(),
                mandate: self.mandate,
                event_authority: self.event_authority,
                program: self.program,
            }
            .to_account_metas(None),
        );
        self.send(ix, &[signer])
    }

    pub fn owner_or_emergency(
        &mut self,
        data: Vec<u8>,
        signer: &Keypair,
    ) -> litesvm::types::TransactionResult {
        let ix = self.ix(
            data,
            markov_mandate::accounts::OwnerOrEmergency {
                caller: signer.pubkey(),
                mandate: self.mandate,
                event_authority: self.event_authority,
                program: self.program,
            }
            .to_account_metas(None),
        );
        self.send(ix, &[signer])
    }

    pub fn unpause(&mut self, signer: &Keypair) -> litesvm::types::TransactionResult {
        self.owner_only(markov_mandate::instruction::Unpause {}.data(), signer)
    }
    pub fn pause(&mut self, signer: &Keypair) -> litesvm::types::TransactionResult {
        self.owner_or_emergency(markov_mandate::instruction::Pause {}.data(), signer)
    }
    pub fn revoke(&mut self, signer: &Keypair) -> litesvm::types::TransactionResult {
        self.owner_or_emergency(markov_mandate::instruction::Revoke {}.data(), signer)
    }
    pub fn amend(&mut self, policy: Policy, signer: &Keypair) -> litesvm::types::TransactionResult {
        self.owner_only(
            markov_mandate::instruction::AmendPolicy { new_policy: policy }.data(),
            signer,
        )
    }

    /// `destination` lets a test try to withdraw into somebody else's account.
    pub fn withdraw_to(
        &mut self,
        amount: u64,
        signer: &Keypair,
        destination: Pubkey,
    ) -> litesvm::types::TransactionResult {
        let ix = self.ix(
            markov_mandate::instruction::OwnerWithdraw { amount }.data(),
            markov_mandate::accounts::OwnerWithdraw {
                owner: signer.pubkey(),
                mandate: self.mandate,
                vault: self.vault,
                destination,
                token_program: anchor_spl::token::ID,
                event_authority: self.event_authority,
                program: self.program,
            }
            .to_account_metas(None),
        );
        self.send(ix, &[signer])
    }

    pub fn withdraw(&mut self, amount: u64, signer: &Keypair) -> litesvm::types::TransactionResult {
        let dest = self.owner_ata;
        self.withdraw_to(amount, signer, dest)
    }

    pub fn execute(
        &mut self,
        intent: markov_mandate::gates::Intent,
        signer: &Keypair,
    ) -> litesvm::types::TransactionResult {
        let mut metas = markov_mandate::accounts::ExecuteVenueAction {
            operator: signer.pubkey(),
            registry: self.registry,
            mandate: self.mandate,
            mint: self.mint,
            vault: self.vault,
            price_update: self.price_update,
            venue_program: self.venue,
            token_program: anchor_spl::token::ID,
            event_authority: self.event_authority,
            program: self.program,
        }
        .to_account_metas(None);
        // The venue's own accounts ride as remaining accounts; the mandate PDA
        // is the signer the venue sees.
        metas.push(anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
            self.mandate,
            false,
        ));
        let ix = self.ix(
            markov_mandate::instruction::ExecuteVenueAction { intent }.data(),
            metas,
        );
        self.send(ix, &[signer])
    }

    pub fn mandate_state(&self) -> Mandate {
        let acc = self.svm.get_account(&self.mandate).unwrap();
        Mandate::try_deserialize(&mut &acc.data[..]).unwrap()
    }

    pub fn vault_amount(&self) -> u64 {
        let acc = self.svm.get_account(&self.vault).unwrap();
        token_amount_of(&acc.data)
    }

    pub fn token_amount(&self, addr: &Pubkey) -> u64 {
        let acc = self.svm.get_account(addr).unwrap();
        token_amount_of(&acc.data)
    }
}

use anchor_lang::AccountDeserialize;

pub fn intent(action: markov_types::ActionKind, notional: u64, id: u8) -> markov_mandate::gates::Intent {
    markov_mandate::gates::Intent {
        intent_id: [id; 32],
        action,
        market: *b"SOL-PERP\0\0\0\0\0\0\0\0",
        notional,
        side: markov_types::Side::Long,
        limit_price: MARK_PRICE_E6,
        max_slippage_bps: 50,
        spend: 1,
        forced: false,
    }
}

/// Decode the `RefusalReceipt`s a transaction emitted, from the CPI-event
/// instruction data in its inner instructions. This is the same path the
/// indexer will use — the receipt is data, not a log line.
pub fn refusals(meta: &litesvm::types::TransactionMetadata) -> Vec<markov_mandate::receipts::RefusalReceipt> {
    event_of::<markov_mandate::receipts::RefusalReceipt>(meta)
}

pub fn actions(meta: &litesvm::types::TransactionMetadata) -> Vec<markov_mandate::receipts::ActionReceipt> {
    event_of::<markov_mandate::receipts::ActionReceipt>(meta)
}

pub fn owner_actions(meta: &litesvm::types::TransactionMetadata) -> Vec<markov_mandate::receipts::OwnerAction> {
    event_of::<markov_mandate::receipts::OwnerAction>(meta)
}

fn event_of<T: anchor_lang::Event + Discriminator + AnchorDeserialize>(
    meta: &litesvm::types::TransactionMetadata,
) -> Vec<T> {
    let mut out = Vec::new();
    for inner in meta.inner_instructions.iter() {
        for ix in inner.iter() {
            let data = &ix.instruction.data;
            // 8 bytes of Anchor's event-CPI marker, then the event discriminator.
            if data.len() < 16 {
                continue;
            }
            if data[8..16] == T::DISCRIMINATOR[..] {
                if let Ok(ev) = T::deserialize(&mut &data[16..]) {
                    out.push(ev);
                }
            }
        }
    }
    out
}

use anchor_lang::AnchorDeserialize;
