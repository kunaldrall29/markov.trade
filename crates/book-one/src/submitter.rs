//! The submitter: the only thing in the agent that signs.
//!
//! It builds exactly one instruction — `execute_venue_action` — and it holds
//! exactly one key, the operator's. That is not a convention: the owner verbs
//! (`unpause`, `amend_policy`, `owner_withdraw`, `set_global_halt`) are absent
//! from this binary, and `cannot_build_owner_instructions` fails the build if
//! any of them appears anywhere in the crate. A stolen operator key can
//! propose; it cannot widen a cap, unpause a book, or move a token.
//!
//! Three rules from `docs/11` §5 that are easy to state and easy to violate:
//!
//! - **`intent_id` is the idempotency key.** A retry re-sends the *same signed
//!   bytes*. The program's replay ring then treats a second landing as
//!   `DuplicateIntent`, which is success-already-happened, not a failure.
//! - **A refusal is never retried.** It is a result. Retrying it would just
//!   produce the same receipt twice and make the tape a lie about how often
//!   the book tried.
//! - **No parameter is ever widened to make a transaction land.** If the cap
//!   refuses it, the answer *is* the refusal.

use anchor_lang::{InstructionData, ToAccountMetas};
use markov_chain::{Chain, Landed};
use markov_guard::Intent as GuardIntent;
use markov_types::BlockReason;
use solana_instruction::{AccountMeta, Instruction};
use solana_keypair::{read_keypair_file, Keypair};
use solana_message::Message;
use solana_pubkey::Pubkey;
use solana_signer::Signer;
use solana_transaction::Transaction;

/// Every mandate instruction this binary is able to construct.
///
/// One entry, and the test below proves the list is not a comment: it scans
/// the crate's own source for the owner verbs.
pub const BUILDABLE_INSTRUCTIONS: &[&str] = &["execute_venue_action"];

/// Instructions the agent must never be able to build, whatever happens to its
/// key. `docs/14` §2 and the P07 hard constraint.
pub const FORBIDDEN_INSTRUCTIONS: &[&str] = &[
    "unpause",
    "amend_policy",
    "owner_withdraw",
    "set_global_halt",
    "revoke",
    "close_mandate",
];

/// The addresses one mandate's actions need. Read from the environment at
/// boot; the agent never discovers a venue or a mint at run time.
#[derive(Clone, Debug)]
pub struct Wiring {
    pub program: Pubkey,
    pub registry: Pubkey,
    pub mandate: Pubkey,
    pub mint: Pubkey,
    pub vault: Pubkey,
    pub price_update: Pubkey,
    pub venue_program: Pubkey,
    pub token_program: Pubkey,
    pub event_authority: Pubkey,
    /// The venue's own accounts, forwarded to the CPI. **Never the vault**:
    /// the program refuses that at gate 15 (ADR-009), and it is refused here
    /// too so a misconfiguration is a boot failure rather than a daily
    /// refusal on the tape.
    pub venue_accounts: Vec<AccountMeta>,
    pub market_id: [u8; 16],
}

impl Wiring {
    /// Refuse to run with a wiring that would be refused on chain.
    pub fn validate(&self) -> Result<(), String> {
        if let Some(bad) = self
            .venue_accounts
            .iter()
            .find(|m| m.pubkey == self.vault || m.pubkey == self.mint)
        {
            return Err(format!(
                "venue accounts must not include an account the mandate controls: {}",
                bad.pubkey
            ));
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum Submitted {
    /// The transaction landed. Whether the program allowed or refused it is in
    /// the receipt, and both are results.
    Landed(Landed),
    /// Never sent: the agent is halted, or out of its hourly budget.
    Withheld(&'static str),
    /// Could not be confirmed after the retries. Not a refusal, and not
    /// recorded as one.
    Failed(String),
}

pub struct Submitter {
    operator: Keypair,
    wiring: Wiring,
}

impl Submitter {
    /// Load the operator key once, at boot. A missing key is fatal: an agent
    /// that cannot sign must not run pretending it can, and a silent
    /// downgrade to read-only would make the tape claim a quiet market when
    /// what it had was a missing credential.
    pub fn new(operator_key_path: &str, wiring: Wiring) -> anyhow::Result<Submitter> {
        wiring.validate().map_err(|e| anyhow::anyhow!(e))?;
        let operator = read_keypair_file(operator_key_path)
            .map_err(|e| anyhow::anyhow!("operator key {operator_key_path}: {e}"))?;
        Ok(Submitter { operator, wiring })
    }

    pub fn operator(&self) -> Pubkey {
        self.operator.pubkey()
    }

    pub fn wiring(&self) -> &Wiring {
        &self.wiring
    }

    /// Build the one instruction this binary knows how to build.
    ///
    /// `price_update_override` exists for the red team's `StaleOracle` probe,
    /// which supplies a mark account that is not this mandate's. It is not a
    /// way to change the price the program uses: the program binds the mark to
    /// the mandate itself and refuses anything else.
    pub fn instruction(
        &self,
        intent: &markov_mandate::gates::Intent,
        price_update_override: Option<Pubkey>,
    ) -> Instruction {
        let w = &self.wiring;
        let mut metas = markov_mandate::accounts::ExecuteVenueAction {
            operator: self.operator.pubkey(),
            registry: w.registry,
            mandate: w.mandate,
            mint: w.mint,
            vault: w.vault,
            price_update: price_update_override.unwrap_or(w.price_update),
            venue_program: w.venue_program,
            token_program: w.token_program,
            event_authority: w.event_authority,
            program: w.program,
        }
        .to_account_metas(None);
        metas.extend(w.venue_accounts.iter().cloned());
        Instruction {
            program_id: w.program,
            accounts: metas,
            data: markov_mandate::instruction::ExecuteVenueAction { intent: *intent }.data(),
        }
    }

    /// Sign once, then send those exact bytes up to `attempts` times.
    pub fn submit(
        &self,
        chain: &Chain,
        intent: &markov_mandate::gates::Intent,
        price_update_override: Option<Pubkey>,
        attempts: u32,
    ) -> Submitted {
        let ix = self.instruction(intent, price_update_override);
        let blockhash = match chain.blockhash() {
            Ok(h) => h,
            Err(e) => return Submitted::Failed(format!("blockhash: {e}")),
        };
        let message = Message::new(&[ix], Some(&self.operator.pubkey()));
        let tx = Transaction::new(&[&self.operator], message, blockhash);
        match chain.send_confirm(&tx, attempts) {
            Ok(landed) => Submitted::Landed(landed),
            Err(e) => Submitted::Failed(e.to_string()),
        }
    }
}

/// Build the program's intent from the guard's, with the idempotency key.
///
/// `intent_id` is `blake3(mandate, slot_bucket, action, notional, nonce)` per
/// `docs/10`; it is derived from what the intent *is*, so the same decision in
/// the same slot bucket produces the same id and the program's replay ring
/// catches a double submission.
pub fn program_intent(
    mandate: &Pubkey,
    slot: u64,
    market_id: [u8; 16],
    intent: &GuardIntent,
    max_slippage_bps: u16,
    forced: bool,
    nonce: u64,
) -> markov_mandate::gates::Intent {
    markov_mandate::gates::Intent {
        intent_id: intent_id(mandate, slot, intent, nonce),
        action: intent.action,
        market: market_id,
        notional: intent.notional,
        side: intent.side,
        limit_price: intent.limit_price_e6,
        max_slippage_bps,
        spend: intent.spend,
        forced,
    }
}

/// The slot bucket is 150 slots — about 25 seconds at devnet's pacing, and
/// comfortably inside one 60-second tick. Two ticks therefore cannot collide
/// on an id, and a retry inside one tick cannot miss.
pub const SLOT_BUCKET: u64 = 150;

pub fn intent_id(mandate: &Pubkey, slot: u64, intent: &GuardIntent, nonce: u64) -> [u8; 32] {
    let mut h = blake3::Hasher::new();
    h.update(mandate.as_ref());
    h.update(&(slot / SLOT_BUCKET).to_le_bytes());
    h.update(&[intent.action as u8, intent.side as u8]);
    h.update(&intent.notional.to_le_bytes());
    h.update(&intent.limit_price_e6.to_le_bytes());
    h.update(&intent.spend.to_le_bytes());
    h.update(&nonce.to_le_bytes());
    *h.finalize().as_bytes()
}

/// Anchor's marker for an `emit_cpi!` event: the first eight bytes of the
/// self-CPI's instruction data, before the event's own discriminator.
const EVENT_CPI_MARKER_LEN: usize = 8;

/// Decode the receipts a landed transaction emitted.
///
/// The program writes **no log lines at all** — a receipt is an `emit_cpi!`
/// event, which reaches the chain as the instruction data of a self-CPI. So
/// this reads inner instruction data, exactly as the indexer will, rather than
/// grepping logs for a name that is never printed. Reading the same bytes is
/// what keeps the agent and the indexer from disagreeing about what happened.
pub fn decode_event<
    T: anchor_lang::Event + anchor_lang::Discriminator + anchor_lang::AnchorDeserialize,
>(
    inner_data: &[Vec<u8>],
) -> Option<T> {
    for data in inner_data {
        if data.len() < EVENT_CPI_MARKER_LEN + 8 {
            continue;
        }
        let (marker, rest) = data.split_at(EVENT_CPI_MARKER_LEN);
        // The marker is Anchor's, not ours; what identifies the event is the
        // discriminator immediately after it.
        let _ = marker;
        let (disc, payload) = rest.split_at(8);
        if disc == T::DISCRIMINATOR {
            if let Ok(event) = T::deserialize(&mut &payload[..]) {
                return Some(event);
            }
        }
    }
    None
}

/// The refusal a transaction recorded, if it recorded one.
pub fn refusal_reason(inner_data: &[Vec<u8>]) -> Option<BlockReason> {
    decode_event::<markov_mandate::receipts::RefusalReceipt>(inner_data).map(|r| r.reason)
}

/// Did this transaction record an executed action?
pub fn action_receipt(inner_data: &[Vec<u8>]) -> Option<markov_mandate::receipts::ActionReceipt> {
    decode_event::<markov_mandate::receipts::ActionReceipt>(inner_data)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The agent cannot build an owner instruction, and this is checked
    /// against the source rather than against a list someone maintains.
    ///
    /// `docs/14` §2: a stolen operator key must not be able to unpause a book,
    /// widen a cap, or move a token. The strongest cheap proof is that the
    /// symbols are not in the binary's crate at all.
    #[test]
    fn cannot_build_owner_instructions() {
        assert_eq!(
            BUILDABLE_INSTRUCTIONS,
            &["execute_venue_action"],
            "the agent builds exactly one instruction"
        );

        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/src");
        let mut scanned = 0usize;
        let mut offences = Vec::new();
        for entry in std::fs::read_dir(dir).expect("src/") {
            let path = entry.expect("entry").path();
            if path.extension().and_then(|e| e.to_str()) != Some("rs") {
                continue;
            }
            let text = std::fs::read_to_string(&path).expect("read");
            scanned += 1;
            for forbidden in FORBIDDEN_INSTRUCTIONS {
                // Anchor's generated type for `owner_withdraw` is
                // `OwnerWithdraw`; check both spellings so a camel-case call
                // site cannot slip through a snake-case grep.
                let camel: String = forbidden
                    .split('_')
                    .map(|w| {
                        let mut c = w.chars();
                        match c.next() {
                            Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                            None => String::new(),
                        }
                    })
                    .collect();
                for needle in [
                    format!("instruction::{camel}"),
                    format!("accounts::{camel}"),
                ] {
                    if text.contains(&needle) {
                        offences.push(format!("{}: {needle}", path.display()));
                    }
                }
            }
        }
        assert!(
            scanned >= 5,
            "scanned only {scanned} files — is the path right?"
        );
        assert!(
            offences.is_empty(),
            "the agent can build an owner instruction: {offences:?}"
        );
    }

    /// The same decision in the same slot bucket is the same intent, so a
    /// retry cannot become a second trade. A different decision is a different
    /// id, so two real actions are never confused for a replay.
    #[test]
    fn intent_id_is_the_idempotency_key() {
        let mandate = Pubkey::new_unique();
        let i = GuardIntent {
            action: markov_types::ActionKind::Open,
            side: markov_types::Side::Long,
            notional: 5_000_000,
            limit_price_e6: 100_000_000,
            spend: 0,
        };
        let a = intent_id(&mandate, 1_000, &i, 7);
        assert_eq!(a, intent_id(&mandate, 1_000, &i, 7), "not deterministic");
        assert_eq!(
            a,
            intent_id(
                &mandate,
                1_000 + SLOT_BUCKET - 1 - (1_000 % SLOT_BUCKET),
                &i,
                7
            ),
            "the same bucket must give the same id"
        );
        assert_ne!(
            a,
            intent_id(&mandate, 1_000 + SLOT_BUCKET, &i, 7),
            "the next bucket is a new decision"
        );
        for changed in [
            GuardIntent {
                notional: 5_000_001,
                ..i
            },
            GuardIntent {
                limit_price_e6: 100_000_001,
                ..i
            },
            GuardIntent { spend: 1, ..i },
            GuardIntent {
                side: markov_types::Side::Short,
                ..i
            },
            GuardIntent {
                action: markov_types::ActionKind::Reduce,
                ..i
            },
        ] {
            assert_ne!(a, intent_id(&mandate, 1_000, &changed, 7), "{changed:?}");
        }
        assert_ne!(a, intent_id(&Pubkey::new_unique(), 1_000, &i, 7), "mandate");
        assert_ne!(a, intent_id(&mandate, 1_000, &i, 8), "nonce");
    }

    /// A wiring that forwards the vault is refused at boot, not once a day on
    /// the tape.
    #[test]
    fn a_wiring_that_forwards_the_vault_is_refused() {
        let vault = Pubkey::new_unique();
        let w = Wiring {
            program: Pubkey::new_unique(),
            registry: Pubkey::new_unique(),
            mandate: Pubkey::new_unique(),
            mint: Pubkey::new_unique(),
            vault,
            price_update: Pubkey::new_unique(),
            venue_program: Pubkey::new_unique(),
            token_program: Pubkey::new_unique(),
            event_authority: Pubkey::new_unique(),
            venue_accounts: vec![AccountMeta::new(vault, false)],
            market_id: *b"SOL-PERP\0\0\0\0\0\0\0\0",
        };
        assert!(
            w.validate().is_err(),
            "gate 15 would refuse this every tick"
        );
        let ok = Wiring {
            venue_accounts: vec![AccountMeta::new_readonly(Pubkey::new_unique(), false)],
            ..w
        };
        assert!(ok.validate().is_ok());
    }

    /// The decoder reads the same bytes the chain carries: an event-CPI
    /// marker, the event's discriminator, then borsh. Built here from the real
    /// receipt type, so a change to the receipt breaks this test rather than
    /// silently producing `None` on every tick.
    #[test]
    fn refusal_reason_decodes_the_receipt_not_a_log_line() {
        use anchor_lang::{AnchorSerialize, Discriminator};
        let receipt = markov_mandate::receipts::RefusalReceipt {
            seq: 1,
            intent_id: [3; 32],
            mandate: Pubkey::new_unique(),
            operator: Pubkey::new_unique(),
            strategy_id: [4; 16],
            venue: Pubkey::new_unique(),
            action: 1,
            notional: 51_000_000,
            reason: BlockReason::OverTxCap,
            gate_index: 8,
            forced: true,
            ts: 1_788_000_000,
            slot: 492_600_000,
        };
        let mut data = vec![0u8; EVENT_CPI_MARKER_LEN];
        data.extend_from_slice(markov_mandate::receipts::RefusalReceipt::DISCRIMINATOR);
        receipt.serialize(&mut data).expect("serialize");

        let inner = vec![vec![1, 2, 3], data];
        assert_eq!(refusal_reason(&inner), Some(BlockReason::OverTxCap));
        let decoded = decode_event::<markov_mandate::receipts::RefusalReceipt>(&inner)
            .expect("the receipt decodes");
        assert_eq!(decoded.gate_index, 8);
        assert!(decoded.forced, "a forced probe must say so on the receipt");

        // Nothing to find is None, not a guess.
        assert_eq!(refusal_reason(&[vec![9; 40]]), None);
        assert_eq!(refusal_reason(&[]), None);
        // An ActionReceipt is not a refusal.
        assert!(action_receipt(&inner).is_none());
    }
}
