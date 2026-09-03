//! **B11**: the `markov-venue` conformance suite, run against this program
//! itself — the real SBF binary in LiteSVM, not a hand-written mirror of it.
//!
//! The suite's assertions are untouched. All that is written here is a
//! `VenueAdapter` + `Fixture` implementation that drives the deployed program
//! through real transactions, which is the point: the same contract that will
//! bind a Gate C venue binds the mock, and nothing about the checks was
//! bent to fit it.
//!
//! Two honest asymmetries, both recorded by the suite rather than hidden:
//!
//! * `InsufficientCollateral` is **not applicable**. This program holds no
//!   token custody at all, so it has no collateral to run out of. A real
//!   venue will produce it; the mock cannot, and `Fixture::starve_collateral`
//!   returns `false` to say so.
//! * Refusals arrive as **return data**, not as program errors (ADR-008), so
//!   the adapter maps the venue's reported code back to a `VenueError`.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use anchor_lang::{
    AccountDeserialize, AnchorDeserialize, AnchorSerialize, Discriminator, InstructionData,
};
use litesvm::LiteSVM;
use markov_types::{MarkSourceKind, Side, VenueReport};
use markov_venue::conformance::{assert_conforms, Fixture, PRICE_E6};
use markov_venue::{
    market_id, Mark, MarketId, Position as VenuePosition, VenueAdapter, VenueError, VenueOutcome,
    WriteRequest,
};
use solana_account::Account;
use solana_instruction::{AccountMeta, Instruction};
use solana_keypair::Keypair;
use solana_message::{Message, VersionedMessage};
use solana_pubkey::Pubkey;
use solana_signer::Signer;
use solana_transaction::versioned::VersionedTransaction;

const NOW: i64 = 1_788_400_000;

/// Anchor's error-code base; `demo_perps`' variants are offset from it.
const E: u32 = 6_000;

fn venue_error_from_code(code: u32) -> Option<VenueError> {
    Some(match code - E {
        0 => VenueError::MarketUnknown,
        1 => VenueError::StaleMark,
        2 => VenueError::SlippageExceeded,
        3 => VenueError::InsufficientCollateral,
        4 => VenueError::PositionLimit,
        5 => VenueError::VenuePaused,
        // NoPosition / InvalidNotional / structural faults are not part of the
        // trait's fixed set; upstream they become VenueRejected at gate 13.
        _ => return None,
    })
}

pub struct LiveDemoPerps {
    svm: LiteSVM,
    payer: Keypair,
    /// A real keypair standing in for the mandate PDA: the program insists on
    /// a signer, and the position account is derived from it.
    mandate: Keypair,
    market_id: MarketId,
    market: Pubkey,
    mark: Pubkey,
    position: Pubkey,
}

impl LiveDemoPerps {
    fn pdas(program: &Pubkey, mandate: &Pubkey, market_id: &MarketId) -> (Pubkey, Pubkey, Pubkey) {
        let (market, _) = Pubkey::find_program_address(&[b"market", market_id.as_ref()], program);
        let (mark, _) = Pubkey::find_program_address(&[b"mark", market_id.as_ref()], program);
        let (position, _) =
            Pubkey::find_program_address(&[b"pos", mandate.as_ref(), market_id.as_ref()], program);
        (market, mark, position)
    }

    fn send(&mut self, ix: Instruction, signers: &[&Keypair]) -> litesvm::types::TransactionResult {
        self.svm.expire_blockhash();
        let bh = self.svm.latest_blockhash();
        let msg = Message::new_with_blockhash(&[ix], Some(&signers[0].pubkey()), &bh);
        let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), signers).unwrap();
        self.svm.send_transaction(tx)
    }

    /// Send a `venue_execute` and translate the outcome the way an adapter
    /// must: a reported fill, a reported refusal, or a structural failure.
    fn execute(&mut self, req: WriteRequest, action: u8) -> Result<VenueOutcome, VenueError> {
        // The suite works with an abstract mandate identity; this program
        // needs a real signer, so the adapter maps one onto the other. A
        // request whose signer *is* its mandate is the legitimate case and is
        // signed by the keypair the position was derived from; anything else
        // is signed by a stranger, and the program's PDA derivation is what
        // stops it reaching this mandate's position.
        let legitimate = req.signer == req.mandate;
        let signer = if legitimate {
            self.mandate.insecure_clone()
        } else {
            // A keypair we control whose pubkey is not the mandate's.
            let other = Keypair::new();
            self.svm.airdrop(&other.pubkey(), 1_000_000_000).unwrap();
            other
        };
        let program = demo_perps::ID;
        let (market, mark, _) = Self::pdas(&program, &self.mandate.pubkey(), &req.market);
        let (_, _, position) = Self::pdas(&program, &signer.pubkey(), &req.market);

        let ix = Instruction {
            program_id: program,
            accounts: vec![
                AccountMeta::new_readonly(signer.pubkey(), true),
                AccountMeta::new_readonly(market, false),
                AccountMeta::new_readonly(mark, false),
                AccountMeta::new(position, false),
            ],
            data: demo_perps::instruction::VenueExecute {
                args: demo_perps::VenueExecuteArgs {
                    action,
                    market: req.market,
                    side: req.side as u8,
                    notional: req.notional,
                    limit_price: req.limit_price,
                    max_slippage_bps: req.max_slippage_bps,
                },
            }
            .data(),
        };
        // The payer must be the signer for a single-signature transaction.
        self.svm.airdrop(&signer.pubkey(), 1_000_000_000).ok();
        match self.send(ix, &[&signer]) {
            Ok(meta) => {
                let data = &meta.return_data.data;
                match VenueReport::try_from_slice(data) {
                    Ok(VenueReport::Filled(f)) => Ok(VenueOutcome::Filled(markov_venue::Fill {
                        price: f.price,
                        notional: f.notional,
                        fee: f.fee,
                    })),
                    Ok(VenueReport::Refused { code }) => {
                        Err(venue_error_from_code(code).unwrap_or(VenueError::VenuePaused))
                    }
                    // No report at all: upstream this is gate 13.
                    Err(_) => Err(VenueError::VenuePaused),
                }
            }
            // A structural failure — a missing market account for an unknown
            // market, or a position that belongs to another signer. The trait
            // names the first case `MarketUnknown`; anything else is a
            // rejection the mandate program turns into `VenueRejected`.
            Err(e) => {
                let logs = e.meta.logs.join(" ");
                if logs.contains("AccountNotInitialized") || logs.contains("MarketUnknown") {
                    Err(VenueError::MarketUnknown)
                } else {
                    Err(VenueError::VenuePaused)
                }
            }
        }
    }
}

impl VenueAdapter for LiveDemoPerps {
    fn venue_program_id(&self) -> Pubkey {
        demo_perps::ID
    }

    fn mark(&self, market: MarketId) -> Result<Mark, VenueError> {
        let (_, mark, _) = Self::pdas(&demo_perps::ID, &self.mandate.pubkey(), &market);
        let acc = self
            .svm
            .get_account(&mark)
            .ok_or(VenueError::MarketUnknown)?;
        if acc.data.is_empty() {
            return Err(VenueError::MarketUnknown);
        }
        let m = demo_perps::MarkAccount::try_deserialize(&mut &acc.data[..])
            .map_err(|_| VenueError::MarketUnknown)?;
        if m.market_id != market {
            return Err(VenueError::MarketUnknown);
        }
        Ok(Mark {
            price: m.price,
            expo: m.expo,
            publish_time: m.publish_time,
            slot: m.slot,
        })
    }

    fn positions(&self, _mandate: Pubkey) -> Result<Vec<VenuePosition>, VenueError> {
        // This adapter instance serves exactly one mandate — the keypair it
        // signs with — so the suite's abstract identity maps to it.
        let (_, _, position) = Self::pdas(&demo_perps::ID, &self.mandate.pubkey(), &self.market_id);
        let Some(acc) = self.svm.get_account(&position) else {
            return Ok(Vec::new());
        };
        if acc.data.is_empty() {
            return Ok(Vec::new());
        }
        let p = demo_perps::Position::try_deserialize(&mut &acc.data[..])
            .map_err(|_| VenueError::MarketUnknown)?;
        if p.notional == 0 {
            return Ok(Vec::new());
        }
        Ok(vec![VenuePosition {
            market: p.market_id,
            side: p.side,
            notional: p.notional,
            entry_price: p.entry_price,
            funding_accrued: p.funding_accrued,
            updated_slot: p.updated_slot,
        }])
    }

    fn open(&mut self, req: WriteRequest) -> Result<VenueOutcome, VenueError> {
        self.execute(req, 1)
    }
    fn increase(&mut self, req: WriteRequest) -> Result<VenueOutcome, VenueError> {
        self.execute(req, 2)
    }
    fn reduce(&mut self, req: WriteRequest) -> Result<VenueOutcome, VenueError> {
        self.execute(req, 3)
    }
    fn close(&mut self, req: WriteRequest) -> Result<VenueOutcome, VenueError> {
        self.execute(req, 4)
    }
}

impl Fixture for LiveDemoPerps {
    fn new_fixture(mandate_key: Pubkey, market_id: MarketId, price: u64) -> Self {
        // The suite hands us a pubkey; the program needs a signer, so the
        // adapter uses its own keypair and ignores the suggested address.
        let _ = mandate_key;
        let mut svm = LiteSVM::new();
        svm.add_program(
            demo_perps::ID,
            include_bytes!(concat!(
                env!("CARGO_TARGET_TMPDIR"),
                "/../deploy/demo_perps.so"
            )),
        )
        .unwrap();
        let mut clock: anchor_lang::solana_program::clock::Clock = svm.get_sysvar();
        clock.unix_timestamp = NOW;
        svm.set_sysvar(&clock);

        let payer = Keypair::new();
        let mandate = Keypair::new();
        svm.airdrop(&payer.pubkey(), 100_000_000_000).unwrap();
        svm.airdrop(&mandate.pubkey(), 100_000_000_000).unwrap();
        let (market, mark, position) = Self::pdas(&demo_perps::ID, &mandate.pubkey(), &market_id);

        let mut me = Self {
            svm,
            payer,
            mandate,
            market_id,
            market,
            mark,
            position,
        };

        let payer = me.payer.insecure_clone();
        // init_market
        let ix = Instruction {
            program_id: demo_perps::ID,
            accounts: vec![
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new(market, false),
                AccountMeta::new(mark, false),
                AccountMeta::new_readonly(anchor_lang::solana_program::system_program::ID, false),
            ],
            data: demo_perps::instruction::InitMarket {
                args: demo_perps::InitMarketArgs {
                    market_id,
                    base_decimals: 9,
                    fee_bps: 10,
                    max_age_slots: 300,
                    position_cap: u64::MAX / 2,
                    poster: payer.pubkey(),
                },
            }
            .data(),
        };
        me.send(ix, &[&payer]).expect("init_market");

        // post_mark
        let ix = Instruction {
            program_id: demo_perps::ID,
            accounts: vec![
                AccountMeta::new_readonly(payer.pubkey(), true),
                AccountMeta::new_readonly(market, false),
                AccountMeta::new(mark, false),
            ],
            data: demo_perps::instruction::PostMark {
                price: price as i64,
                expo: -6,
                publish_time: NOW - 5,
            }
            .data(),
        };
        me.send(ix, &[&payer]).expect("post_mark");

        // init_position for the mandate this adapter signs with
        let mandate_pk = me.mandate.pubkey();
        let ix = Instruction {
            program_id: demo_perps::ID,
            accounts: vec![
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new_readonly(market, false),
                AccountMeta::new(position, false),
                AccountMeta::new_readonly(anchor_lang::solana_program::system_program::ID, false),
            ],
            data: demo_perps::instruction::InitPosition {
                mandate: mandate_pk,
                market_id,
            }
            .data(),
        };
        me.send(ix, &[&payer]).expect("init_position");
        me
    }

    fn market() -> MarketId {
        market_id("SOL-PERP").expect("fits")
    }

    fn unknown_market() -> MarketId {
        market_id("NOPE-PERP").expect("fits")
    }

    fn make_mark_stale(&mut self) -> bool {
        // Replay an old slot into the mark account, which is what P04 asks
        // for: the mark is stale, not the clock.
        let mut acc = self.svm.get_account(&self.mark).unwrap();
        let mut mark = demo_perps::MarkAccount::try_deserialize(&mut &acc.data[..]).unwrap();
        mark.slot = 1;
        mark.source = MarkSourceKind::House;
        let mut data = demo_perps::MarkAccount::DISCRIMINATOR.to_vec();
        mark.serialize(&mut data).unwrap();
        data.resize(acc.data.len(), 0);
        acc.data = data;
        self.svm.set_account(self.mark, acc).unwrap();
        let mut clock: anchor_lang::solana_program::clock::Clock = self.svm.get_sysvar();
        clock.slot = 100_000;
        self.svm.set_sysvar(&clock);
        true
    }

    fn pause(&mut self) -> bool {
        let payer = self.payer.insecure_clone();
        let market = self.market;
        let ix = Instruction {
            program_id: demo_perps::ID,
            accounts: vec![
                AccountMeta::new_readonly(payer.pubkey(), true),
                AccountMeta::new(market, false),
            ],
            data: demo_perps::instruction::SetMarketPaused { paused: true }.data(),
        };
        self.send(ix, &[&payer]).expect("set_market_paused");
        true
    }

    fn starve_collateral(&mut self) -> bool {
        // This program holds no token custody, so there is no collateral to
        // starve and `InsufficientCollateral` is unreachable here by design.
        false
    }

    fn cap_positions_below(&mut self, notional: u64) -> bool {
        let payer = self.payer.insecure_clone();
        let market = self.market;
        let ix = Instruction {
            program_id: demo_perps::ID,
            accounts: vec![
                AccountMeta::new_readonly(payer.pubkey(), true),
                AccountMeta::new(market, false),
            ],
            data: demo_perps::instruction::SetMarketParams {
                fee_bps: 10,
                max_age_slots: 300,
                position_cap: notional.saturating_sub(1),
            }
            .data(),
        };
        self.send(ix, &[&payer]).expect("set_market_params");
        true
    }
}

/// B11. The suite is the one in `crates/markov-venue`, unmodified.
#[test]
fn demo_perps_conforms_to_the_venue_adapter_trait() {
    assert_conforms::<LiveDemoPerps>("demo_perps (live, LiteSVM)");
}

/// The mock must not flatter itself: a taker never gets a better price than
/// the mark, and the same inputs always produce the same fill.
#[test]
fn fills_are_deterministic_and_never_favourable() {
    let market = <LiveDemoPerps as Fixture>::market();
    let mut a = LiveDemoPerps::new_fixture(Pubkey::default(), market, PRICE_E6);
    let mandate = a.mandate.pubkey();
    let req = WriteRequest {
        mandate,
        signer: mandate,
        market,
        side: Side::Long,
        notional: 1_000_000,
        limit_price: PRICE_E6 + PRICE_E6 * 20 / 10_000,
        max_slippage_bps: 50,
    };
    let mut prices = Vec::new();
    for _ in 0..3 {
        match a.open(req).expect("fill") {
            VenueOutcome::Filled(f) => {
                assert!(
                    f.price > PRICE_E6,
                    "a long open filled at or below the mark"
                );
                assert_eq!(
                    f.price,
                    PRICE_E6 + PRICE_E6 * 10 / 10_000,
                    "fill is not mark + fee"
                );
                prices.push(f.price);
            }
            other => panic!("expected a fill, got {other:?}"),
        }
    }
    assert!(
        prices.windows(2).all(|w| w[0] == w[1]),
        "fills drifted: {prices:?}"
    );
}

/// Funding accrues against the long side at the published devnet constant.
#[test]
fn funding_accrues_against_the_long() {
    let market = <LiveDemoPerps as Fixture>::market();
    let mut a = LiveDemoPerps::new_fixture(Pubkey::default(), market, PRICE_E6);
    let mandate = a.mandate.pubkey();
    let req = WriteRequest {
        mandate,
        signer: mandate,
        market,
        side: Side::Long,
        notional: 10_000_000,
        limit_price: PRICE_E6 + PRICE_E6 * 20 / 10_000,
        max_slippage_bps: 50,
    };
    a.open(req).expect("open");
    // Move the clock on, then touch the position so funding is applied.
    let mut clock: anchor_lang::solana_program::clock::Clock = a.svm.get_sysvar();
    clock.slot += 200;
    a.svm.set_sysvar(&clock);
    let mut smaller = req;
    smaller.notional = 1;
    a.reduce(smaller).expect("reduce");

    let acc = a.svm.get_account(&a.position).unwrap();
    let p = demo_perps::Position::try_deserialize(&mut &acc.data[..]).unwrap();
    assert!(
        p.funding_accrued < 0,
        "a long should have paid funding, got {}",
        p.funding_accrued
    );
}

/// The mark records where it came from, on chain, so no surface has to guess.
#[test]
fn the_mark_records_its_source() {
    let market = <LiveDemoPerps as Fixture>::market();
    let a = LiveDemoPerps::new_fixture(Pubkey::default(), market, PRICE_E6);
    let acc = a.svm.get_account(&a.mark).unwrap();
    let m = demo_perps::MarkAccount::try_deserialize(&mut &acc.data[..]).unwrap();
    // Posted by the house poster in the fixture, so it must say `house` —
    // never `pyth`, which would overstate where the number came from.
    assert_eq!(m.source, MarkSourceKind::House);
    assert_eq!(m.source.name(), "house");
}
