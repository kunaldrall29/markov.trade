//! Create the Gate B mandate the agent runs against, and nothing else.
//!
//! `devnet_smoke.rs` proves the program works end to end and finishes by
//! withdrawing, which is correct for a smoke test and wrong for the mandate an
//! agent is about to be pointed at. This does the setup and stops: registry,
//! mandate with the Gate B template policy, funded vault, and the venue's
//! market, mark and position accounts.
//!
//! Idempotent by construction: every step checks whether the account already
//! exists, so re-running it after a partial failure is safe.
//!
//! Prints the environment the agent needs. Nothing but public keys and
//! signatures reaches stdout.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use anchor_lang::{InstructionData, ToAccountMetas};
use markov_mandate::state::{action_bits, Mandate, Policy, Registry};
use solana_client::rpc_client::RpcClient;
use solana_commitment_config::CommitmentConfig;
use solana_instruction::{AccountMeta, Instruction};
use solana_keypair::{read_keypair_file, Keypair};
use solana_message::Message;
use solana_pubkey::Pubkey;
use solana_signer::Signer;
use solana_transaction::Transaction;

const USDC_D: Pubkey = solana_pubkey::pubkey!("7ajorFYMrE9Mi3yZkwWaZp6ahzkK6RotZ75qAdtTV9Rj");
const SOL_D: Pubkey = solana_pubkey::pubkey!("73V1Vhs3A8j8NrXKCbGmRek2dR92x9MkwUk4WEdYYRfQ");
const PYTH_SOL_USD: Pubkey = solana_pubkey::pubkey!("7UVimffxr9ow1uXYxsr4LHAcV58mLzhmwaeKvJ1pjLiE");
const FEED_ID_HEX: &str = "ef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d";

/// The Gate B template, from `docs/11` §3 and `crates/book-one/src/config.rs`.
/// The agent refuses to boot against a policy that differs from this, so the
/// two must agree — and they are checked against each other by
/// `gate_b_policy_differences`.
const PER_TX_CAP: u64 = 50_000_000;
const DAILY_CAP: u64 = 200_000_000;
const SPEND_PER_CALL: u64 = 1_000_000;
const SPEND_DAILY: u64 = 5_000_000;
const MAX_SLIPPAGE_BPS: u16 = 50;
const MAX_MARK_AGE_SECS: u64 = 150;
/// Long enough that the 24-hour Gate B run cannot expire mid-way, short enough
/// that a forgotten mandate stops being live.
const EXPIRY_DAYS: i64 = 30;

fn rpc_url() -> String {
    std::env::var("RPC_HTTP_URL")
        .ok()
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| "https://api.devnet.solana.com".to_string())
}

fn key(name: &str) -> Keypair {
    read_keypair_file(format!("keys/{name}.json"))
        .unwrap_or_else(|e| panic!("keys/{name}.json: {e}"))
}

fn metas(m: impl ToAccountMetas) -> Vec<AccountMeta> {
    m.to_account_metas(None)
}

fn feed_id() -> [u8; 32] {
    let mut out = [0u8; 32];
    for (i, b) in out.iter_mut().enumerate() {
        *b = u8::from_str_radix(&FEED_ID_HEX[i * 2..i * 2 + 2], 16).unwrap();
    }
    out
}

fn send(rpc: &RpcClient, label: &str, ix: Instruction, payer: &Keypair, signers: &[&Keypair]) {
    let bh = rpc.get_latest_blockhash().expect("blockhash");
    let msg = Message::new(&[ix], Some(&payer.pubkey()));
    let mut tx = Transaction::new_unsigned(msg);
    tx.sign(signers, bh);
    let mut delay = std::time::Duration::from_millis(400);
    for attempt in 1..=5 {
        match rpc.send_and_confirm_transaction(&tx) {
            Ok(sig) => {
                println!("  {label:<28} {sig}");
                return;
            }
            Err(e) => {
                let m = e.to_string();
                let retryable = m.contains("429") || m.contains("rate") || m.contains("timed out");
                if !retryable || attempt == 5 {
                    panic!("{label} failed: {e}");
                }
                std::thread::sleep(delay);
                delay *= 2;
            }
        }
    }
}

fn main() {
    let url = rpc_url();
    let rpc = RpcClient::new_with_commitment(url.clone(), CommitmentConfig::confirmed());
    match rpc.get_genesis_hash() {
        Ok(h) if h.to_string() == "EtWTRABZaYq6iMfeYKouRu166VU2xqa1wcaWoxPkrZBG" => {}
        Ok(h) => panic!("{url} is not devnet (genesis {h})"),
        Err(e) => panic!("{url}: {e}"),
    }

    let program = markov_mandate::ID;
    let venue = demo_perps::ID;
    let deployer = key("deployer");
    let owner = key("owner-demo");
    let operator = key("operator");
    let emergency = key("emergency");
    let strategy_id = markov_mandate::BOOK_ONE;
    let nonce: u64 = std::env::var("NONCE")
        .ok()
        .and_then(|v| v.parse().ok())
        .expect("NONCE must be set explicitly, so a run cannot silently reuse a mandate");
    let fund_amount: u64 = std::env::var("FUND")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(100_000_000); // 100 USDC-d
    let owner_ata: Pubkey = std::env::var("OWNER_ATA")
        .expect("OWNER_ATA")
        .parse()
        .unwrap();

    let (registry, _) = Pubkey::find_program_address(&[Registry::SEED], &program);
    let (event_authority, _) = Pubkey::find_program_address(&[b"__event_authority"], &program);
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
    let market_id: [u8; 16] = *b"SOL-PERP\0\0\0\0\0\0\0\0";
    let (venue_market, _) = Pubkey::find_program_address(&[b"market", market_id.as_ref()], &venue);
    let (venue_mark, _) = Pubkey::find_program_address(&[b"mark", market_id.as_ref()], &venue);
    let (venue_position, _) =
        Pubkey::find_program_address(&[b"pos", mandate.as_ref(), market_id.as_ref()], &venue);

    println!("rpc       {url}");
    println!("program   {program}");
    println!("venue     {venue}");
    println!("mandate   {mandate}  (nonce {nonce})");
    println!("vault     {vault}");
    println!("signatures:");

    if rpc.get_account(&registry).is_err() {
        send(
            &rpc,
            "init_registry",
            Instruction {
                program_id: program,
                accounts: metas(markov_mandate::accounts::InitRegistry {
                    admin: deployer.pubkey(),
                    registry,
                    system_program: anchor_lang::solana_program::system_program::ID,
                }),
                data: markov_mandate::instruction::InitRegistry {}.data(),
            },
            &deployer,
            &[&deployer],
        );
    }
    send(
        &rpc,
        "set_adapters",
        Instruction {
            program_id: program,
            accounts: metas(markov_mandate::accounts::AdminOnly {
                admin: deployer.pubkey(),
                registry,
            }),
            data: markov_mandate::instruction::SetAdapters {
                adapters: vec![venue],
            }
            .data(),
        },
        &deployer,
        &[&deployer],
    );

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let mut policy = Policy {
        venues: [Pubkey::default(); 4],
        venues_len: 1,
        tokens: [Pubkey::default(); 4],
        tokens_len: 2,
        allowed_actions: action_bits::ALL,
        per_tx_cap: PER_TX_CAP,
        daily_cap: DAILY_CAP,
        spend_per_call: SPEND_PER_CALL,
        spend_daily: SPEND_DAILY,
        max_slippage_bps: MAX_SLIPPAGE_BPS,
        max_mark_age_secs: MAX_MARK_AGE_SECS,
        expiry_ts: now + EXPIRY_DAYS * 86_400,
    };
    policy.venues[0] = venue;
    policy.tokens[0] = USDC_D;
    policy.tokens[1] = SOL_D;

    if rpc.get_account(&mandate).is_err() {
        send(
            &rpc,
            "create_mandate",
            Instruction {
                program_id: program,
                accounts: metas(markov_mandate::accounts::CreateMandate {
                    owner: owner.pubkey(),
                    mint: USDC_D,
                    mandate,
                    vault,
                    token_program: anchor_spl::token::ID,
                    system_program: anchor_lang::solana_program::system_program::ID,
                    event_authority,
                    program,
                }),
                data: markov_mandate::instruction::CreateMandate {
                    args: markov_mandate::CreateMandateArgs {
                        operator: operator.pubkey(),
                        emergency: emergency.pubkey(),
                        strategy_id,
                        nonce,
                        policy,
                        mark_account: PYTH_SOL_USD,
                        feed_id: feed_id(),
                    },
                }
                .data(),
            },
            &owner,
            &[&owner],
        );
        send(
            &rpc,
            "fund",
            Instruction {
                program_id: program,
                accounts: metas(markov_mandate::accounts::Fund {
                    owner: owner.pubkey(),
                    mandate,
                    mint: USDC_D,
                    owner_ata,
                    vault,
                    token_program: anchor_spl::token::ID,
                    event_authority,
                    program,
                }),
                data: markov_mandate::instruction::Fund {
                    amount: fund_amount,
                }
                .data(),
            },
            &owner,
            &[&owner],
        );
    } else {
        println!("  mandate already exists; leaving it alone");
    }

    // The venue's own accounts. `init_market` is global; the position is per
    // mandate, so a new mandate always needs one.
    if rpc.get_account(&venue_market).is_err() {
        send(
            &rpc,
            "venue init_market",
            Instruction {
                program_id: venue,
                accounts: vec![
                    AccountMeta::new(deployer.pubkey(), true),
                    AccountMeta::new(venue_market, false),
                    AccountMeta::new(venue_mark, false),
                    AccountMeta::new_readonly(
                        anchor_lang::solana_program::system_program::ID,
                        false,
                    ),
                ],
                data: demo_perps::instruction::InitMarket {
                    args: demo_perps::InitMarketArgs {
                        market_id,
                        base_decimals: 6,
                        fee_bps: 10,
                        // Slots, not seconds: this is the venue's own gate,
                        // independent of the mandate's (ADR-003). ~165 ms a
                        // slot, so 900 is about 150 seconds.
                        max_age_slots: 900,
                        position_cap: 1_000_000_000,
                        poster: deployer.pubkey(),
                    },
                }
                .data(),
            },
            &deployer,
            &[&deployer],
        );
    }
    send(
        &rpc,
        "venue post_mark_from_pyth",
        Instruction {
            program_id: venue,
            // No signer: relaying a verified Pyth price is not a claim, so
            // the venue lets anyone do it. `post_mark` — the house fallback —
            // is the one restricted to the allowlisted poster.
            accounts: vec![
                AccountMeta::new_readonly(venue_market, false),
                AccountMeta::new(venue_mark, false),
                AccountMeta::new_readonly(PYTH_SOL_USD, false),
            ],
            data: demo_perps::instruction::PostMarkFromPyth { feed_id: feed_id() }.data(),
        },
        &deployer,
        &[&deployer],
    );
    if rpc.get_account(&venue_position).is_err() {
        send(
            &rpc,
            "venue init_position",
            Instruction {
                program_id: venue,
                accounts: vec![
                    AccountMeta::new(deployer.pubkey(), true),
                    AccountMeta::new_readonly(venue_market, false),
                    AccountMeta::new(venue_position, false),
                    AccountMeta::new_readonly(
                        anchor_lang::solana_program::system_program::ID,
                        false,
                    ),
                ],
                data: demo_perps::instruction::InitPosition { mandate, market_id }.data(),
            },
            &deployer,
            &[&deployer],
        );
    }

    // A deliberate seeding step, and labelled as one everywhere it appears.
    //
    // The Gate B core cannot open a position: every path that adds exposure is
    // behind `funding_favourable`, a stub constant `false` until a venue
    // reports real funding. So a book with nothing in it has nothing to
    // manage, and the agent would skip forever — truthfully, but proving
    // nothing about whether it can act.
    //
    // Seeding one position outside the delta band gives it something real to
    // do: the core's own rule 4 sees exposure past the band and reduces it.
    // **This transaction is not an agent decision** and must never be
    // presented as one. The agent's action is the reduce that follows.
    if let Some(seed) = std::env::var("SEED")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
    {
        let mark = rpc.get_account(&venue_mark).expect("venue mark");
        let mark: demo_perps::MarkAccount =
            anchor_lang::AccountDeserialize::try_deserialize(&mut mark.data.as_slice())
                .expect("decode mark");
        // The venue fills at mark + fee_bps for a taker, so the limit has to
        // leave room for the fee without exceeding the policy's bound.
        let mark_e6 = if mark.expo == -8 {
            (mark.price / 100) as u64
        } else {
            panic!("unexpected mark exponent {}", mark.expo)
        };
        let limit = mark_e6 + mark_e6 * u64::from(MAX_SLIPPAGE_BPS) / 2 / 10_000;
        println!("  (mark {mark_e6}, seed limit {limit})");

        let mut seed_metas = metas(markov_mandate::accounts::ExecuteVenueAction {
            operator: operator.pubkey(),
            registry,
            mandate,
            mint: USDC_D,
            vault,
            price_update: PYTH_SOL_USD,
            venue_program: venue,
            token_program: anchor_spl::token::ID,
            event_authority,
            program,
        });
        for m in [
            AccountMeta::new_readonly(mandate, false),
            AccountMeta::new_readonly(venue_market, false),
            AccountMeta::new_readonly(venue_mark, false),
            AccountMeta::new(venue_position, false),
        ] {
            seed_metas.push(m);
        }
        send(
            &rpc,
            "SEED open (not an agent decision)",
            Instruction {
                program_id: program,
                accounts: seed_metas,
                data: markov_mandate::instruction::ExecuteVenueAction {
                    intent: markov_mandate::gates::Intent {
                        intent_id: *b"gate-b-seed-position-0000000000\0",
                        action: markov_types::ActionKind::Open,
                        market: market_id,
                        notional: seed,
                        side: markov_types::Side::Long,
                        limit_price: limit,
                        max_slippage_bps: MAX_SLIPPAGE_BPS,
                        spend: 0,
                        forced: false,
                    },
                }
                .data(),
            },
            &operator,
            &[&operator],
        );
    }

    println!();
    println!("the agent's environment:");
    println!("  VENUE=devnet");
    println!("  MANDATE={mandate}");
    println!("  PROGRAM_ID={program}");
    println!("  VENUE_PROGRAM={venue}");
    println!("  MARKET_ID=SOL-PERP");
    println!("  OPERATOR_KEY_PATH=keys/operator.json");
}
