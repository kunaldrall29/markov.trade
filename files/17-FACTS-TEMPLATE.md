# 17 — `docs/FACTS.md` template
Markov Book · 31 August 2026 · v0.1

FACTS is the single source of truth for versions, IDs, and decoded shapes. Rules:

1. Every row has a **value, a date, and a source** (a command that was run, a URL that was fetched, or an explorer link).
2. Code reads IDs from env populated from FACTS. Nothing hardcodes a program ID in a source file.
3. A row with `PENDING` blocks any work that depends on it. If a verification fails, stop and report — do not substitute a guess.
4. Re-verify on every deploy that changes a program, and on every dependency bump.

---

```markdown
# FACTS — Markov Book
Last full verification: <date>   Verified by: <seat>

## Cluster and toolchain
| Key | Value | Verified | Source |
| CLUSTER | devnet | | |
| ANCHOR_VERSION | PENDING | | `anchor --version` |
| SOLANA_CLI_VERSION | PENDING | | `solana --version` |
| RUST_VERSION | PENDING | | `rustc --version` |
| LITESVM_VERSION | PENDING | | Cargo.lock |
| CODAMA_VERSION | PENDING | | package.json |
| KIT_VERSION | PENDING | | package.json (@solana/kit) |

## Programs and accounts
| PROGRAM_ID | PENDING | | `solana program show` |
| PROGRAM_IDL_SHA | PENDING | | `sha256sum target/idl/markov_mandate.json` |
| UPGRADE_AUTHORITY | PENDING | | `solana program show` |
| DEMO_PERPS_ID | PENDING | | deploy output |
| REGISTRY_PDA | PENDING | | derive script |
| DEMO_MANDATE_PDA | PENDING | | derive script |
| VAULT_ATA | PENDING | | explorer |

## Mints
| USDC_D_MINT | PENDING | | explorer |
| SOL_D_MINT | PENDING | | explorer |
| USDC_D_DECIMALS | PENDING | | mint account |

## BlockReason enum (decoded from deployed IDL — APPEND ONLY)
| # | Name | First emitted (sig) | Verified |
| 0 | PENDING | | |
| ... one row per variant ... |

## Keys (public only)
| OWNER_DEMO_PUBKEY | PENDING | | |
| OPERATOR_PUBKEY | PENDING | | |
| EMERGENCY_PUBKEY | PENDING | | |
| MARK_POSTER_PUBKEY | PENDING | | |

## Price source
| MARK_SOURCE | PENDING (hermes \| house) | | |
| HERMES_URL | PENDING | | fetched |
| SOL_USD_FEED_ID | PENDING | | fetched |
| PYTH_RECEIVER_PROGRAM | PENDING | | docs, post-18-Aug-2026 upgrade — confirm devnet address |
| MARK_MAX_AGE_SLOTS | PENDING | | policy |

## Hosts
| BOOK_URL | PENDING | | opened logged-out |
| LANDING_URL | PENDING | | |
| RECEIPTS_API_URL | PENDING | | |
| HEALTH_URL | PENDING | | |
| RPC_PRIMARY | PENDING | | |
| RPC_FALLBACK | PENDING | | |

## Gate B proofs
| SIG-FUND | PENDING | | explorer |
| SIG-ACT | PENDING | | |
| SIG-AMEND | PENDING | | |
| SIG-CAP | PENDING | | |
| SIG-REV | PENDING | | |
| SIG-REV2 | PENDING | | |
| SIG-WD | PENDING | | |
| SIG-SLIP-OR-SPEND | PENDING | | |
| PAPER_START_DATE | PENDING | | first file in paper/ |
| GATE_B_STATUS | OPEN | | |
| GATE_B_CLOSED_DATE | — | | |

## Venue checklist (ADR-03) — no venue may be named in public copy until all five pass
| Venue | 1 programmatic open/close/cancel | 2 settlement mint | 3 position+funding readable | 4 paper/devnet parity | 5 ToS allows vault agent | Verdict |
| <name> | PENDING | PENDING | PENDING | PENDING | PENDING | NOT NAMED |
```

## Verification log format

Append below the tables; never edit an old entry.

```
2026-09-02  Agents  ANCHOR_VERSION=<x>  cmd: `anchor --version`  note: pinned in Anchor.toml + CI image
2026-09-02  Agents  pyth-solana-receiver-sdk build against pinned Anchor: FAILED
                    -> STOP. Reported. Decision recorded in ADR-013: house MarkAccount fallback,
                       labelled on the page as a house-posted devnet mark.
```

That second entry is the shape a failed pre-flight is supposed to take: it stops, it says what failed, it names the decision that followed, and it does not improvise a workaround inside the code.
