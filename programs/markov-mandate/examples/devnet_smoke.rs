//! P02 devnet evidence: create → fund → refuse → withdraw, against the real
//! program on devnet. Prints one signature per step so FACTS can cite them.
//!
//! Not a Gate B proof run: the strategy id is `BOOK_ONE` but the agent is not
//! running here, so these are `P02-*` signatures, not `SIG-ACT`/`SIG-FUND`.
//! Keys come from `keys/`, which is gitignored; nothing is printed but public
//! keys and signatures.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use anchor_lang::{AccountDeserialize, InstructionData, ToAccountMetas};
use markov_mandate::state::{action_bits, Mandate, Policy, Registry};
use solana_client::rpc_client::RpcClient;
use solana_commitment_config::CommitmentConfig;
use solana_instruction::{AccountMeta, Instruction};
use solana_keypair::{read_keypair_file, Keypair};
use solana_message::Message;
use solana_pubkey::Pubkey;
use solana_signer::Signer;
use solana_transaction::Transaction;

/// Read from `RPC_HTTP_URL` so the run can use whichever endpoint FACTS
/// currently names; the public default is the fallback of last resort.
fn rpc_url() -> String {
    std::env::var("RPC_HTTP_URL")
        .ok()
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| "https://api.devnet.solana.com".to_string())
}
const USDC_D: Pubkey = solana_pubkey::pubkey!("7ajorFYMrE9Mi3yZkwWaZp6ahzkK6RotZ75qAdtTV9Rj");
const SOL_D: Pubkey = solana_pubkey::pubkey!("73V1Vhs3A8j8NrXKCbGmRek2dR92x9MkwUk4WEdYYRfQ");
const PYTH_SOL_USD: Pubkey = solana_pubkey::pubkey!("7UVimffxr9ow1uXYxsr4LHAcV58mLzhmwaeKvJ1pjLiE");
const FEED_ID_HEX: &str = "ef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d";

fn key(name: &str) -> Keypair {
    read_keypair_file(format!("keys/{name}.json"))
        .unwrap_or_else(|e| panic!("keys/{name}.json: {e}"))
}

fn metas(m: impl ToAccountMetas) -> Vec<AccountMeta> {
    m.to_account_metas(None)
        .into_iter()
        .map(|a| AccountMeta {
            pubkey: a.pubkey,
            is_signer: a.is_signer,
            is_writable: a.is_writable,
        })
        .collect()
}

fn send(
    rpc: &RpcClient,
    label: &str,
    ix: Instruction,
    payer: &Keypair,
    signers: &[&Keypair],
) -> Option<String> {
    let bh = rpc.get_latest_blockhash().expect("blockhash");
    let msg = Message::new(&[ix], Some(&payer.pubkey()));
    let mut tx = Transaction::new_unsigned(msg);
    tx.sign(signers, bh);
    // Public endpoints rate-limit; a refusal to land is worth retrying, a
    // refusal by the program is not (it already landed as a receipt).
    let mut delay = std::time::Duration::from_millis(400);
    for attempt in 1..=5 {
        match rpc.send_and_confirm_transaction(&tx) {
            Ok(sig) => {
                println!("  {label:<28} {sig}");
                return Some(sig.to_string());
            }
            Err(e) => {
                let msg = e.to_string();
                let retryable =
                    msg.contains("429") || msg.contains("rate") || msg.contains("timed out");
                if !retryable || attempt == 5 {
                    println!("  {label:<28} FAILED: {e}");
                    return None;
                }
                std::thread::sleep(delay);
                delay *= 2;
            }
        }
    }
    None
}

fn feed_id() -> [u8; 32] {
    let mut out = [0u8; 32];
    for (i, b) in out.iter_mut().enumerate() {
        *b = u8::from_str_radix(&FEED_ID_HEX[i * 2..i * 2 + 2], 16).unwrap();
    }
    out
}

fn main() {
    let url = rpc_url();
    let rpc = RpcClient::new_with_commitment(url.clone(), CommitmentConfig::confirmed());
    // Refuse to run against anything but devnet. A mainnet URL here would make
    // every read succeed and every number wrong.
    match rpc.get_genesis_hash() {
        Ok(h) if h.to_string() == "EtWTRABZaYq6iMfeYKouRu166VU2xqa1wcaWoxPkrZBG" => {}
        Ok(h) => panic!("{url} is not devnet (genesis {h})"),
        Err(e) => panic!("{url}: {e}"),
    }
    println!("rpc       {url}");
    let program = markov_mandate::ID;
    let venue = demo_perps::ID;
    let deployer = key("deployer");
    let owner = key("owner-demo");
    let operator = key("operator");
    let emergency = key("emergency");

    let (registry, _) = Pubkey::find_program_address(&[Registry::SEED], &program);
    let (event_authority, _) = Pubkey::find_program_address(&[b"__event_authority"], &program);
    let strategy_id = markov_mandate::BOOK_ONE;
    // A fresh nonce each run, so re-running never collides with an existing PDA.
    let nonce: u64 = std::env::var("NONCE")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(1);
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
    let owner_ata: Pubkey = std::env::var("OWNER_ATA")
        .expect("OWNER_ATA")
        .parse()
        .unwrap();

    println!("program   {program}");
    println!("venue     {venue}");
    println!("owner     {}", owner.pubkey());
    println!("operator  {}", operator.pubkey());
    println!("mandate   {mandate}  (nonce {nonce})");
    println!("vault     {vault}");
    println!("signatures:");

    // Registry: create once, then keep the adapter list current.
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
        per_tx_cap: 50_000_000, // 50 USDC-d
        daily_cap: 200_000_000, // 200 USDC-d
        spend_per_call: 1_000_000,
        spend_daily: 5_000_000,
        max_slippage_bps: 50,
        max_mark_age_secs: 150,
        expiry_ts: now + 14 * 86_400, // the Gate B template: 14 days
    };
    policy.venues[0] = venue;
    policy.tokens[0] = USDC_D;
    policy.tokens[1] = SOL_D;

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
                amount: 100_000_000,
            }
            .data(),
        },
        &owner,
        &[&owner],
    );

    // The ladder, on chain: an intent above the per-tx cap must come back as a
    // committed OverTxCap receipt, not a failed transaction.
    let mut exec_metas = metas(markov_mandate::accounts::ExecuteVenueAction {
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
    exec_metas.push(AccountMeta::new_readonly(mandate, false));
    let over_cap = markov_mandate::gates::Intent {
        intent_id: [nonce as u8; 32],
        action: markov_types::ActionKind::Open,
        market: *b"SOL-PERP\0\0\0\0\0\0\0\0",
        notional: 51_000_000, // per_tx_cap is 50
        side: markov_types::Side::Long,
        limit_price: 100_000_000,
        max_slippage_bps: 50,
        spend: 100_000,
        forced: true, // a deliberate probe, recorded as such
    };
    send(
        &rpc,
        "execute (OverTxCap)",
        Instruction {
            program_id: program,
            accounts: exec_metas,
            data: markov_mandate::instruction::ExecuteVenueAction { intent: over_cap }.data(),
        },
        &operator,
        &[&operator],
    );

    // ── P04: the venue is real, so set it up and take a real fill ────────
    let market_id: [u8; 16] = *b"SOL-PERP\0\0\0\0\0\0\0\0";
    let (venue_market, _) = Pubkey::find_program_address(&[b"market", market_id.as_ref()], &venue);
    let (venue_mark, _) = Pubkey::find_program_address(&[b"mark", market_id.as_ref()], &venue);
    let (venue_position, _) =
        Pubkey::find_program_address(&[b"pos", mandate.as_ref(), market_id.as_ref()], &venue);
    println!("venue market   {venue_market}");
    println!("venue mark     {venue_mark}");
    println!("venue position {venue_position}");

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
                        base_decimals: 9,
                        fee_bps: 10,
                        max_age_slots: 300,
                        position_cap: 1_000_000_000,
                        poster: key("mark-poster").pubkey(),
                    },
                }
                .data(),
            },
            &deployer,
            &[&deployer],
        );
    }

    // Relay the Pyth price onto the venue's mark. `source` records `pyth`,
    // which is what the page is allowed to say.
    send(
        &rpc,
        "venue post_mark_from_pyth",
        Instruction {
            program_id: venue,
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

    // Read the mark the venue just recorded, so the limit is derived from the
    // real price rather than guessed.
    let mark_e6: u64 = match rpc.get_account(&venue_mark) {
        Ok(acc) => {
            let m = demo_perps::MarkAccount::try_deserialize(&mut &acc.data[..]).expect("mark");
            println!(
                "mark           {} expo {} source {:?}",
                m.price, m.expo, m.source
            );
            let shift = m.expo + 6;
            if shift >= 0 {
                (m.price as u128 * 10u128.pow(shift as u32)) as u64
            } else {
                (m.price as u128 / 10u128.pow((-shift) as u32)) as u64
            }
        }
        Err(e) => panic!("mark unreadable: {e}"),
    };
    // The venue fills a long open at mark + 10 bps; ask for a limit 20 bps
    // above the mark so the fill lands inside the bound and is visibly not
    // the limit.
    let limit = mark_e6 + mark_e6 * 20 / 10_000;

    let mut open_metas = metas(markov_mandate::accounts::ExecuteVenueAction {
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
        open_metas.push(m);
    }
    let open = markov_mandate::gates::Intent {
        intent_id: [(nonce as u8).wrapping_add(100); 32],
        action: markov_types::ActionKind::Open,
        market: market_id,
        notional: 5_000_000, // 5 USDC-d, well inside the 50 per-tx cap
        side: markov_types::Side::Long,
        limit_price: limit,
        max_slippage_bps: 50,
        spend: 100_000,
        forced: false,
    };
    send(
        &rpc,
        "execute (real fill)",
        Instruction {
            program_id: program,
            accounts: open_metas,
            data: markov_mandate::instruction::ExecuteVenueAction { intent: open }.data(),
        },
        &operator,
        &[&operator],
    );

    send(
        &rpc,
        "owner_withdraw",
        Instruction {
            program_id: program,
            accounts: metas(markov_mandate::accounts::OwnerWithdraw {
                owner: owner.pubkey(),
                mandate,
                vault,
                destination: owner_ata,
                token_program: anchor_spl::token::ID,
                event_authority,
                program,
            }),
            data: markov_mandate::instruction::OwnerWithdraw { amount: 40_000_000 }.data(),
        },
        &owner,
        &[&owner],
    );
}
