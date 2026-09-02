//! P01 harness proof: the built .so loads into LiteSVM and `ping` lands.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use anchor_lang::{solana_program::instruction::Instruction, InstructionData, ToAccountMetas};
use litesvm::LiteSVM;
use solana_keypair::Keypair;
use solana_message::{Message, VersionedMessage};
use solana_signer::Signer;
use solana_transaction::versioned::VersionedTransaction;

#[test]
fn ping_lands_on_litesvm() {
    let program_id = markov_mandate::id();
    let payer = Keypair::new();
    let mut svm = LiteSVM::new();
    let bytes = include_bytes!(concat!(
        env!("CARGO_TARGET_TMPDIR"),
        "/../deploy/markov_mandate.so"
    ));
    svm.add_program(program_id, bytes).unwrap();
    svm.airdrop(&payer.pubkey(), 1_000_000_000).unwrap();

    let ix = Instruction::new_with_bytes(
        program_id,
        &markov_mandate::instruction::Ping {}.data(),
        markov_mandate::accounts::Ping {}.to_account_metas(None),
    );
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[ix], Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&payer]).unwrap();

    let res = svm.send_transaction(tx);
    assert!(res.is_ok(), "{res:?}");
    let logs = res.unwrap().logs;
    assert!(logs.iter().any(|l| l.contains("scaffold ping")), "{logs:?}");
}
