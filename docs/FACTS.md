# FACTS — Markov Book
Last full verification: 2026-09-01   Verified by: Session 0 (all seats)

Rules (from `docs/17-FACTS-TEMPLATE.md`): every row has a value, a date and a source. `PENDING` blocks work that depends on it. Nothing below is guessed; where a value comes from someone else's ledger it says so and says whether this session re-verified it.

Two deployed `markov-mandate`-shaped programs exist on devnet. This file names **`5o8E…`** as `PROGRAM_ID` because it is the one that has emitted all eleven `BlockReason`s on chain, and records **`CT2n…`** as its dormant predecessor. Whether Gate B upgrades `5o8E…` in place or deploys a successor is decision **D3** (`docs/adr/ADR-004`), still open.

## Cluster and toolchain (this workstation, 2026-09-01)
| Key | Value | Verified | Source |
|---|---|---|---|
| CLUSTER | devnet (`https://api.devnet.solana.com`, cluster version `4.3.0-beta.3`, slot ≈ 491,626,706, epoch 1138) | 2026-09-01 | `solana config set --url devnet`; `solana cluster-version`; `solana epoch-info` |
| ANCHOR_VERSION | **installed: anchor-cli 0.31.1** (via avm 1.1.2). Both deployed programs pin `anchor_version = "0.31.1"`. Current stable on crates.io is **1.1.2** (2026-06-26); `2.0.0-rc.1` (2026-08-12). **Pin is decision D0 (`ADR-001`), open.** | 2026-09-01 | `avm install 0.31.1 && anchor --version`; `https://crates.io/api/v1/crates/anchor-lang`; `gh api repos/coral-xyz/anchor/releases`; `Anchor.toml` in both program repos |
| SOLANA_CLI_VERSION | solana-cli **4.2.2** (src e29e5d91, client Agave), solana-keygen 4.2.2, cargo-build-sbf 4.1.0, platform-tools v1.54. Installed today from `release.anza.xyz/stable`. Agave GitHub latest stable v4.2.2 (2026-08-28). | 2026-09-01 | `solana --version`; `cargo-build-sbf --version`; `gh api repos/anza-xyz/agave/releases` |
| RUST_VERSION | host rustc **1.97.1** (rustup `stable`, only toolchain installed). The `5o8E…` program declares `rust-version = "1.79"`; its authoring box used 1.85.0. `rust-toolchain.toml` for this repo: PENDING (P01). | 2026-09-01 | `rustc --version`; `rustup toolchain list`; `programs/mandate/Cargo.toml` in `kunaldrall29/markov` |
| LITESVM_VERSION | PENDING pin. Latest **0.16.0** (2026-08-24) depends on `agave-*`/`solana-*` **4.2/4.3** crates, while anchor-lang 0.31.1 depends on `solana-program ^2` and anchor-lang 1.1.2 on `solana-*` ^3. Compatibility in one workspace is **not verified** and is a P01 pre-flight build check. | 2026-09-01 | `crates.io/api/v1/crates/litesvm/0.16.0/dependencies`; anchor-lang 0.31.1 / 1.1.2 dependency lists |
| SURFPOOL_VERSION | not installed; latest v1.5.0 (2026-07-13). Not needed for a devnet-only mock-venue build unless LiteSVM proves incompatible. | 2026-09-01 | `gh api repos/txtx/surfpool/releases` |
| CODAMA_VERSION | PENDING pin. Latest: `codama` 1.10.2, `@codama/nodes-from-anchor` 1.5.5, `@codama/renderers-js` 2.4.0. | 2026-09-01 | `npm view <pkg> version` |
| KIT_VERSION | PENDING pin. Latest: `@solana/kit` 8.2.0, `@solana/kit-plugin-wallet` 0.19.0, `@solana/react` 8.2.0, `@solana/wallet-standard` 1.1.6. Legacy `@solana/web3.js` is 1.98.4 (the existing TS stack uses it; not for new code). | 2026-09-01 | `npm view <pkg> version` |
| ANCHOR_TS_CLIENT | `@coral-xyz/anchor` npm latest is **0.32.1** (last published 2025-10-10). `@anchor-lang/core` 1.1.2 exists. Not needed if the web client is Codama-generated. | 2026-09-01 | `npm view @coral-xyz/anchor dist-tags time.modified`; `npm view @anchor-lang/core version` |
| WEB_STACK_LATEST | `@tanstack/react-start` 1.168.49, `@tanstack/react-router` 1.170.32, `tailwindcss` 4.3.3, `@tailwindcss/vite` 4.3.3, `vite` 8.2.2, `react` 19.2.8, `lucide-react` 1.39.0, `zustand` 5.0.15, `@playwright/test` 1.62.1. Pins PENDING (P01). | 2026-09-01 | `npm view <pkg> version` |
| NODE / PNPM / BUN | node v24.19.0, pnpm 9.15.9, npm 11.17.0, bun 1.4.0 (`~/.bun/bin`), git 2.43.0, docker 29.7.2 | 2026-09-01 | `node --version` etc. |
| FAUCET | `solana airdrop 1` and `0.5` to a throwaway key both failed: "rate limit reached". P01 pre-flight #2 not yet satisfied from this IP. | 2026-09-01 | `solana airdrop` ×2 |

## Programs and accounts
| Key | Value | Verified | Source |
|---|---|---|---|
| PROGRAM_ID | **`5o8EAwdHyQ31Nmt6tUDm1y6PNDt5STmVvA6CX3E6WJPm`** — executable, BPF upgradeable loader, ProgramData `8313nq75PfJ1BE2Z2i5JWXjG5iJXRRRXcZJXz2kH1K85`, last deployed slot **489249058**, data length 405120. Source: `github.com/kunaldrall29/markov` `programs/mandate` (`declare_id!` matches; Anchor 0.31.1). 98 transactions 2026-08-27T14:21:37Z → 2026-08-29T20:32:10Z, zero errored. | 2026-09-01 | `solana program show 5o8E…`; `getSignaturesForAddress` full walk; `gh api repos/kunaldrall29/markov/contents/programs/mandate/src/lib.rs` |
| PROGRAM_ID_PREDECESSOR | `CT2ntFzcG68wx3XXjjcmZRCKjDE52tZtZYirCe6AadPA` — `MarkovFyi/markov-program` "Phase 0 M1" mandate. Authority `Af1HZgvuD16XVYQzzQTiKH3hToFKx12NPyohpWwBykPy`, slot 487093766, 364336 bytes. Only 4 transactions, all deploys on 2026-08-23; **no receipt ever emitted**. Its 13-name `BlockReason` set (`Unauthorized, MandatePaused, MandateRevoked, MandateExpired, VenueNotAllowlisted, TokenNotAllowlisted, PerTxCapExceeded, UtcDayCapExceeded, X402BudgetExceeded, OperatorNotRegistered, InvalidStateTransition, FailClosed, VenueExecuteNotShipped`) is therefore not bound by the append-only rule. Not the Gate B base. | 2026-09-01 | `solana program show CT2n…`; signature walk; `state.rs`, `SPEC.md`, `docs/FACTS.md` in `MarkovFyi/markov-program`; `packages/idl/src/block-reason.ts` in `MarkovFyi/markov-sdk` |
| PROGRAM_IDL_SHA | **No on-chain IDL account** for either program (Anchor IDL PDA `9GQArvHfqCMCmRPLkssq9iyZSm2CoFzc1cSPabUij1sT` for `5o8E…` and `4uoX4w8C5HirJ9J1Pk5m7NuzJC46NiHNRzwGmfgy2e27` for `CT2n…` both absent). Checked-in IDL `kunaldrall29/markov:packages/operator/idl/mandate.json` — **sha256 `55c62d4b946de7397b2359c6861bb641a10b0877fea4eb4cef9667df8b27196d`**, 32549 bytes, `metadata.version 0.1.0`, `spec 0.1.0`. Consistency with the deployed binary verified two ways: all 8 event discriminators seen on chain match the IDL, and the `BlockReason` discriminants 0–10 decoded from 20 on-chain `ActionRefused` payloads match the IDL enum order. | 2026-09-01 | `dump-idl.mjs` (Kit PDA derivation + `getAccountInfo`); `sha256sum`; `disc.py` (byte 81 of `ActionRefused`); `hist.py` event discriminator histogram |
| UPGRADE_AUTHORITY | **`2fpQvTynG9cnCXUqsrJ8CvpJZsNykehMdSv4nkJVStGg`** for `5o8E…`, `demo_swap` and `demo_yield`. Balance 6.7469 SOL; ≥1000 signatures 2026-08-27 → 08-29. Key file is `keys/deployer.json` in the working tree that ran the 27–29 Aug sessions (gitignored; **custody not confirmed by this session — D3 input**). | 2026-09-01 | `solana program show`; `solana balance`; `Anchor.toml [provider] wallet = "keys/deployer.json"` and SESSION_LOG in `kunaldrall29/markov` |
| DEMO_PERPS_ID | PENDING (not built). Existing stub venues of `5o8E…`: `demo_swap` **`3HwcGXdsbfaAov2rYhDtnyeeEbuFVXUaT5GASTCjUUSK`** (slot 489248688, 236832 B) and `demo_yield` **`GsE3vpoBb26vZWbPBbtMACwVem2qgw7whouTLwAAhyzC`** (slot 489248781, 237408 B), same authority. Pools/vaults in `data/devnet.json`: swap pool `7uN8ZGqgy3pfmzz89CPvzBheEmyCFH6BBUGDSxqYQhdV`, yield pool `97nuXo5qWJcudP6yNd3sQAxGCp2TRmvLbHduegjomaze`. | 2026-09-01 | `solana program show`; CPI targets in the 98-tx histogram (11 `Swap`, 2 `Deposit`); `data/devnet.json` |
| REGISTRY_PDA | PENDING — the deployed program has **no registry / global halt**. | 2026-09-01 | `lib.rs` read in full |
| DEMO_MANDATE_PDA | PENDING for Gate B. Existing mandate PDAs (seeds `["mandate", owner, seed_le_u64]`, **not** the pack's `[mandate, owner, strategy_id, nonce]`): `8wEAR5oSYzKrRtLki8H7E87TcHaDYbzuFT7L2cjnPnJo` (four-beat), `4YQ5Xm7qxxC6UjDHPSh8F2Md5dorb8hsKbcWpfrM21X4` (wallet demo, momentum `strategy_id`), `ERUee9ziZwBcpAVtGs8U7KcpgwmvpxDkLMgDG756vFM5` (latest receipt). 20 `MandateCreated` events total. | 2026-09-01 | `lib.rs` seeds; hosted `/v1/receipts`; `kunaldrall29/markov` FACTS |
| VAULT_ATA | PENDING | | |
| ACCOUNT_LAYOUT | `Mandate` = 8 + 673 bytes, **no reserve padding**: owner, operator, emergency, quote_mint (4×Pubkey), state u8 (0 Active / 1 Paused / 2 Revoked — **no Expired state**, expiry is a gate), created_ts, expires_ts, day_stamp, spent_today, spend_today, nonce, yield_shares, seed, bump, `Policy`, `strategy_id: Option<[u8;32]>`. `Policy` = programs[4]+program_len, tokens[4]+token_len, per_tx_cap, daily_cap, spend_per_call_cap, spend_daily_cap, max_slippage_bps u16. No `max_mark_age_slots`, no `allowed_actions`, no `expiry` in policy. | 2026-09-01 | `lib.rs`; IDL `types` |
| INSTRUCTIONS_DEPLOYED | `register_operator, create_mandate, fund, amend_policy, pause, unpause, revoke, owner_withdraw, execute_swap, execute_deposit, execute_withdraw_venue, spend` (12). No `init_vault`, `close_mandate`, `set_global_halt`, `execute_venue_action`. Refusals **already emit and return `Ok(())`**; `owner_withdraw` has **no state check**; `unpause` is `OwnerOnly`; emergency = pause/revoke only (`check_emergency_or_owner`). `amend_policy` is **not** tighten-only (any valid policy accepted). Events use `emit!` (`Program data:` logs), not `emit_cpi`. | 2026-09-01 | `lib.rs`; IDL; 98-tx instruction histogram (`ExecuteSwap` 29, `Fund` 20, `CreateMandate` 20, `Spend` 10, `Revoke` 6, `OwnerWithdraw` 4, `RegisterOperator` 3, `ExecuteDeposit` 2, `Unpause` 2, `Pause` 2) |
| GATE_ORDER_DEPLOYED | `Paused → Revoked → Expired → Unauthorized (caller ≠ operator) → ProgramNotAllowed → TokenNotAllowed → OverTxCap → OverDailyCap → SlippageExceeded` (swap path); spend path substitutes `OverSpendCap → OverSpendDailyCap` after the token gate. Differs from `docs/10-PROGRAM-SPEC.md` §3 (which adds GlobalHalt, DuplicateIntent, ActionNotAllowed, StaleOracle, VenueRejected, PostCheckFailed and orders state before halt). | 2026-09-01 | `gate_state`, `gate_swap`, `gate_move` in `lib.rs` |

## Mints
| Key | Value | Verified | Source |
|---|---|---|---|
| USDC_D_MINT | **`6eDVRpGLcBeYfazqsUN9tyeW7NEKjgE3TSgs7yz1YmvC`** — SPL Token (classic), decimals 6, supply 3,002,574.000000, mint authority `2fpQ…`. Used in 147 token-balance rows across the 98 txs. (The predecessor stack's USDC-d `J83TSeZ1RXEvGpc76bZsZMcFtuPbJHAhUrmyjYc8wdWH` is a different mint, authority `Af1H…`.) | 2026-09-01 | `getAccountInfo` jsonParsed; `mints.py` over all 98 txs; `data/devnet.json` |
| SOL_D_MINT | PENDING — no SOL-d exists. The second existing mint is **DEMO** `54XpmsacxMvrWPFXrtSjYkconsWm4Xpd7sFPUH1Z4zhd` (decimals 6, supply 10,000,000, authority `2fpQ…`). | 2026-09-01 | same |
| USDC_D_DECIMALS | 6 | 2026-09-01 | mint account |

## BlockReason enum (decoded from the deployed `5o8E…` program — APPEND ONLY)
Discriminants were read from the `reason` byte of 20 on-chain `ActionRefused` events (offset 81 in the borsh payload), cross-checked against the program log line `ActionRefused <Name>` in the same transaction, the `lib.rs` enum order, and the checked-in IDL. All four agree. **These eleven, in this order, are frozen.** New variants append at 11+.

| # | Name | First emitted (sig) | Slot / time (UTC) | Verified |
|---|---|---|---|---|
| 0 | `OverTxCap` | `3ajx6eZ67oJGGsL5TUzHvhLGrW3wBaXXng7kYxxzu7DGQmL7B3jJ1tBrDZxjGMudS6pex1yF3rD9b2rj4gtkq6cR` | 489250149 · 2026-08-28T06:53:14Z | 2026-09-01 |
| 1 | `OverDailyCap` | `2o4HvZShnyqEVvLrbxcd9kHechjcUaAvPBtEheW9BqU2SYdRJys8Lh9DK1rn87Y3D6NbCCK7HokRQ9pe4uXw1xtb` | 490057931 · 2026-08-29T20:07:53Z | 2026-09-01 |
| 2 | `OverSpendCap` | `3BWrnwFAwZJ4T21UedSqqsNzN9YMa4Lah4hcEXKUseWvkeK6u8zLrh1qBoP6QfRMVqzrBq6j8uWDrkNYEPyzXDEb` | 490057937 · 2026-08-29T20:07:54Z | 2026-09-01 |
| 3 | `OverSpendDailyCap` | `48rxsnSqyKLX5HSL6YhMSaeY8Vxb2w4do2RL5eSCQW7wDGy9zpdsHB8DJFRFAKQFWFyqnoSrvvayP61gqzr8buhB` | 490057959 · 2026-08-29T20:07:58Z | 2026-09-01 |
| 4 | `ProgramNotAllowed` | `5WYsaxf5jDFdkN1LMQ9peF8EVxofmEvcHKMC8xZfEFCQb1ehxtcoXVGu32Q3iEAnYAAxey4AnCstmPoqV9wrKtUa` | 490057893 · 2026-08-29T20:07:47Z | 2026-09-01 |
| 5 | `TokenNotAllowed` | `39t5zf1zCbUYvE5gkUE3YSSF8jhPzWHYPun6tHnv2RwThMXk7PDJSbiLAXXcT3uSxzCT3s98GbBzUX4GtEfpAa7u` | 490057915 · 2026-08-29T20:07:51Z | 2026-09-01 |
| 6 | `SlippageExceeded` | `8ctcEEoD95bS7xnTXtJepaiv3Lr3poryvwZj1CKNrnpPauD8gXKb1MnpV5F9UVDwsoiJYwjKzyvQYCxLrre334P` | 490057962 · 2026-08-29T20:07:58Z | 2026-09-01 |
| 7 | `Expired` | `LCtb5nHDARCNuHDmGM5T4NFGsMnNrLaqBjqd733bvhGpAGh3NG46uwTyNQK3zHCEXXdpdYMeULtibiCSkBmuySc` | 490057877 · 2026-08-29T20:07:44Z | 2026-09-01 |
| 8 | `Paused` | `4DUZvXKMM7TU4VLHFspxF7fwewAcXCkL5tLB7dZZ4qqrNj4Z87D6duEUyfSqYRngcmmHSgFhJ37LoSjsGFW4rDBW` | 490057745 · 2026-08-29T20:07:23Z | 2026-09-01 |
| 9 | `Revoked` | `3vZt8w1yzn799rFrBW8yMNqARnV29K6TzhX7P1FN7puAQJhF7MM2wSBg9vfmSzLTpDTZTiXaAQyqHYTd7br7mwcU` | 489250156 · 2026-08-28T06:53:16Z | 2026-09-01 |
| 10 | `Unauthorized` | `4KRnfoC51QHhfG5e5CqsKNZjMRT45Xf1nLUFEAMuL8i1gFSjFG8BWUfugKrszV7JWbinz4TxYorTcvujbsnMECjG` | 490057880 · 2026-08-29T20:07:45Z | 2026-09-01 |
| 11+ | *(to append in Gate B, names from `docs/10-PROGRAM-SPEC.md` §4: `StaleOracle`, `ActionNotAllowed`, `DuplicateIntent`, `GlobalHalt`, `VenueRejected`, `PostCheckFailed` — order to be fixed in P02, never reordered after first emit)* | — | — | — |

Counts on chain: `OverTxCap` ×6, `Revoked` ×5, every other reason ×1 (20 refusals, 21 `ActionExecuted`). **Spec reconciliation required:** `docs/10-PROGRAM-SPEC.md` §4 numbers `Paused` as 0 and names `NotOperator` / `VenueNotAllowed`; the chain says `OverTxCap` is 0 and the names are `Unauthorized` / `ProgramNotAllowed`. The deployed program wins (ADR-004).

Deployed event set (IDL discriminators, all but one seen on chain): `ActionExecuted` (21), `ActionRefused` (20), `MandateCreated` (20), `MandateFunded` (20), `Revoked` (6), `OwnerWithdrew` (4), `Paused` (2), `Unpaused` (2), `PolicyAmended` (0 — never emitted).

## Keys (public only)
| Key | Value | Verified | Source |
|---|---|---|---|
| OWNER_DEMO_PUBKEY | PENDING for Gate B (generate fresh per `docs/14` §5). Existing demo owner signer: `526oKwSqS7mGTBRbJLFqXwZxygLvGZicd9RGYRrWafTq` (50 of 98 txs; `keys/owner.json` in the prior workspace; 0.3055 SOL). | 2026-09-01 | signer histogram; `solana balance` |
| OPERATOR_PUBKEY | PENDING for Gate B. Existing house operators, all registered on chain: `markov-steady` `AFmFYWsn7hijB54y45Tvs8XQxuy1uG9MRQXDqThKXXBs` (19 sigs), `markov-momentum` `CoVy9jnkzdxzi2say1tpHgiwDbcJ4n7RTvVwp8Rt8bHZ` (10), `markov-redteam` `3ASMCGmhgjuhgVQB672tygz1FYxZk44Fq7rjpCS9jDWB` (15). | 2026-09-01 | `data/house-operators.json`; signer histogram |
| EMERGENCY_PUBKEY | PENDING for Gate B. Existing: `6rkSFPh6HWYfT9dMhgiHQ7wV8Fxs7rK1RhgMmKJUFoQz` (6 sigs). **Its private key (`EMERGENCY_KEY_JSON`) sits in the Railway `api` service environment, not the `bot` service** — the bot calls the API, which signs. That is the opposite of `docs/14` §2 (emergency key in its own service). | 2026-09-01 | `data/house-operators.json`; Railway `get-service-config` variable names for `api` and `bot` |
| MARK_POSTER_PUBKEY | PENDING | | |
| KEY_CUSTODY | Where `keys/deployer.json`, `keys/owner.json`, `keys/op_dca.json`, `keys/emergency.json` physically live is **unknown to this session** (gitignored and dockerignored; the sessions that used them ran in a `/workspace` checkout, not this box). `~/.config/solana` does not exist here. | 2026-09-01 | `.gitignore`, `.dockerignore`, `apps/api/src/chain.ts` paths; `ls ~/.config/solana` |

## Price source
| Key | Value | Verified | Source |
|---|---|---|---|
| MARK_SOURCE | PENDING (D2, `ADR-003`) | | |
| HERMES_URL | `https://hermes.pyth.network` — **`/v2/updates/price/latest`, `/v2/updates/price/stream` and v1 `/api/latest_price_feeds` return `401 unauthorized`**; `/v2/price_feeds` (metadata) returns 200. Pyth docs: "Authentication becomes required on August 26, 2026 at 16:00 UTC … Hermes now requires an API Key" (Pyth Pro key, `/price-feeds/pro/acquire-api-key`) or a node provider (Triton, P2P, extrnode, Liquify). Public limit was 10 req / 10 s / IP. `hermes-beta.pyth.network` behaves identically. | 2026-09-01 | `curl` with three user agents; `docs.pyth.network/price-feeds/core/how-pyth-works/hermes` and `…/api-instances-and-providers/hermes` |
| SOL_USD_FEED_ID | `ef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d` (`Crypto.SOL/USD`) | 2026-09-01 | `GET /v2/price_feeds?query=SOL/USD` on hermes and hermes-beta |
| PYTH_RECEIVER_PROGRAM | `rec5EKMGg6MxZYaMdyBfgwp4d5rB9T1VQH5pJv5LtFJ` — listed for **Solana Devnet** in the Pyth docs table; on devnet executable, **last upgraded slot 491006444 (≈ 2026-08-31)**, authority `upg8KLALUN7ByD…`. The address did not change with the August upgrades. | 2026-09-01 | docs page; `getAccountInfo` on program + ProgramData; `getBlockTime` |
| PYTH_PRICE_FEED_PROGRAM | `pythWSnswVUd12oZpeFP8e9CVaEqJg25g1Vtc2biRsT` — docs table Solana Devnet; on devnet executable, last upgraded slot 293898740 (2024-04-22). Push oracle `HDwcJBJXjL9FpJ7UBsYBtaDjsBUhuLCUYoz3zr8SWWaQ` upgraded 2026-08-25. | 2026-09-01 | same |
| PYTH_DEVNET_FEED_ACCOUNT | **`7UVimffxr9ow1uXYxsr4LHAcV58mLzhmwaeKvJ1pjLiE`** — SOL/USD `PriceUpdateV2`, shard 0, PDA of `pythW…` seeds `[u16 0 LE, feed_id]`, **owned by the receiver `rec5…`**, 134 bytes, `verification_level = Full`. At read: price 9999848408 × 10⁻⁸ = 99.998 USD, conf 1501627, `publish_time` 2026-09-01T20:39:19Z, `posted_slot` 491633562, **age 66 slots / 14 s**. Shards 1 and 2 exist but are stale (last posted 2026-03-02 and 2024-07-19); shard 3 absent. A free, slot-stamped on-chain mark exists on devnet; cadence over a full day not yet measured. | 2026-09-01 | `pyth-feed.mjs` (Kit PDA derivation, `getAccountInfo` base64, hand-decoded `PriceUpdateV2`), `getSlot` |
| PYTH_SDK_COMPAT | `pyth-solana-receiver-sdk` **2.0.0** (2026-06-15) requires `anchor-lang ^1.0.2` (+ `pythnet-sdk ^3`); **1.0.1** requires `^0.31.1`; 1.1.0 / 1.2.0 require `^0.32.1`; 0.6.1 `>=0.28`. So a matching SDK exists for either D0 pin. **Build proof PENDING (P04 pre-flight).** | 2026-09-01 | `crates.io/api/v1/crates/pyth-solana-receiver-sdk/<v>/dependencies` |
| MARK_MAX_AGE_SLOTS | PENDING (policy) | | |

## Hosts
| Key | Value | Verified | Source |
|---|---|---|---|
| LANDING_URL | `https://markov.trade` — domain is **registered and delegated to Cloudflare** (`tori.ns.cloudflare.com`, `simon.ns.cloudflare.com`) but has **no A/AAAA/CNAME records**; HTTPS unreachable. Same for `www.` and `api.markov.trade`. | 2026-09-01 | `dig NS/SOA/A`; `curl` |
| BOOK_URL | PENDING (D1, `ADR-002`) | | |
| RECEIPTS_API_URL | Intended `https://api.markov.trade/v1/receipts` (no DNS). **Existing:** `https://data-api-production-5ac5.up.railway.app/v1/receipts` → 200; `/v1/receipts/stats` → `{total:41, allowed:21, blocked:20}` with all 11 `by_reason` keys. CORS: responds 200 to `Origin: https://markov.trade` with no `Access-Control-Allow-Origin` header (allowlist is markovhq.com + localhost per its SECURITY.md). | 2026-09-01 | `curl` |
| HEALTH_URL | Existing `data-api /health` → `chainReady:false, rpcOk:false, lastIndexedSlot 490387517, lagSlots 1,245,076` (indexer stalled since ≈ 2026-08-29); `indexer /health` same lag; `api /health` → `chainReady:true` (it only probes RPC), slot 491632562. | 2026-09-01 | `curl` |
| RPC_PRIMARY | PENDING — no RPC-provider account or key found on this box or in the Railway variable names (indexer uses `SOLANA_RPC_URL`/`SOLANA_WS_URL` = public devnet, which 429s). | 2026-09-01 | `env`; Railway variable names |
| RPC_FALLBACK | `https://api.devnet.solana.com` (`getVersion` → solana-core 4.3.0-beta.3). Rate-limited this session on `getTransaction` bursts (HTTP 429). | 2026-09-01 | `curl` |
| EXISTING_RAILWAY | Project **`markov`** `9c4577ff-23a5-4ebf-a167-2aa0a10caad7`, workspace "kunal drall's Projects" (user `kunaldrall29`), environment `production`. Services (all Dockerfile builds from `kunaldrall29/markov`, all online): `api` `api-production-d2e8.up.railway.app` · `data-api` `data-api-production-5ac5…` · `indexer` `indexer-production-00ef…` · `agents` `agents-production-2720…` · `bot` `bot-production-e30d…` · `Postgres` (volume). Six failed deploys 28 Aug, last successful 29 Aug 20:30 UTC. No custom domains. | 2026-09-01 | Railway MCP `whoami`, `list-projects`, `list-services`, `environment-status`, `get-service-config`, `list-domains` |
| EXISTING_VERCEL | MCP-visible team `kunals-projects-35d3a237` (hobby): `markov-float` (0 deployments; git-linked to **`kunal-drall/markov`, which is the old Base product — stale link**), `float-web` `prj_tcpy7u9riTqUOAZEfoSnf5olJz9B`, `markov-docs`, `markov` (`markov-rosy.vercel.app`), plus `markov-landing-…`, `markov-litepaper-…`, `markov-site-…`. The **live** Float/docs deploys (`float-web-three.vercel.app`, `float.markovhq.com`, `markov-docs-black.vercel.app`) were made to a different team, `lemmalabs`, which this MCP connection cannot see. | 2026-09-01 | Vercel MCP `list_teams`, `list_projects`, `get_project`; `curl` (all three 200) |
| EXISTING_SITES | `https://markovhq.com` 200 (Next.js marketing site, `kunaldrall29/markov-landing`; litepaper page v0.6 names `5o8E…` in §13). `https://markovhq.grok.me` 200 — the **Markov Book design**: TanStack Start (`$_TSR`, `/assets/{index,routes,book,book-store,blotter,footer}-*.js`), Tailwind v4; `/book` exists and is a **static sample blotter** (no fetches, no program IDs, header `BOOK_ONE · demo_perps`, copy `Demo blotter — not mainnet capital`). | 2026-09-01 | `curl`; asset grep |
| EXISTING_BOT | Telegram `@markov_float_bot` (`tokenSet:true` on the Railway `bot` service; token in Railway env only). | 2026-09-01 | `bot /health` |

## Gate B proofs
| Key | Value | Verified | Source |
|---|---|---|---|
| SIG-FUND | PENDING | | |
| SIG-ACT | PENDING | | |
| SIG-AMEND | PENDING | | |
| SIG-CAP | PENDING | | |
| SIG-REV | PENDING | | |
| SIG-REV2 | PENDING | | |
| SIG-WD | PENDING | | |
| SIG-SLIP-OR-SPEND | PENDING | | |
| PAPER_START_DATE | PENDING — `paper/` does not exist yet. **Never edit once written.** | 2026-09-01 | `ls` |
| GATE_B_STATUS | **OPEN** | 2026-09-01 | |
| GATE_B_CLOSED_DATE | — | | |

Prior-art signatures on `5o8E…` that are *not* Gate B proofs (different program spec, `strategy_id` momentum/redteam, no `BOOK_ONE`): four-beat 2026-08-28 (fund `2iqbzg6s…`, OverTxCap `3ajx6eZ6…`, Revoked `3vZt8w1y…`, withdraw `342HP72T…`); wallet demo `mdt_0038` (create `3mGrXq5v…`, pause `623dpkC8…`, withdraw `Nt7YSyWm…`, revoke `5uBLfQ6T…`). Recorded so nobody re-labels them as B2–B8.

## Venue checklist (ADR-03) — no venue may be named in public copy until all five pass
| Venue | 1 programmatic open/close/cancel | 2 settlement mint | 3 position+funding readable | 4 paper/devnet parity | 5 ToS allows vault agent | Verdict |
|---|---|---|---|---|---|---|
| *(none evaluated)* | PENDING | PENDING | PENDING | PENDING | PENDING | NOT NAMED |

`scripts/copy-grep.sh` run on `landing/index.html`: **clean** (exit 0). Injecting `Earn 12% APY, guaranteed. Built on Jupiter.` produced four FAIL rules (promised-rate, apy-claim, guaranteed, named-venue) and exit 1. 2026-09-01.

## Verification log (append-only; never edit an old entry)

```
2026-09-01  S0  PACK_LAYOUT          pack arrived flat under files/; moved with git mv to docs/ prompts/ landing/ scripts/ (commit 73b0dc3); contents byte-identical to ~/projects/markov.trade/docs (diff -rq)
2026-09-01  S0  RUST_VERSION=1.97.1  cmd: `rustc --version`
2026-09-01  S0  ANCHOR/SOLANA        `anchor`, `solana`, `avm` NOT INSTALLED at session start -> installed Agave stable (solana-cli 4.2.2) via release.anza.xyz, avm 1.1.2 via `cargo install --git coral-xyz/anchor avm`, `avm install 0.31.1`. anchor-cli 0.31.1 prints. Pin still D0.
2026-09-01  S0  FAUCET               `solana airdrop 1` / `0.5` -> "rate limit reached" twice. Balance 0. P01 pre-flight #2 unsatisfied.
2026-09-01  S0  DEVNET_RPC           getVersion -> solana-core 4.3.0-beta.3; getTransaction bursts hit HTTP 429 (needs a provider)
2026-09-01  S0  PROGRAM_ID           two candidates found in kunaldrall29/markov-landing docs/FACTS.md; both executable on devnet. Resolved: 5o8E... (kunaldrall29/markov, 98 txs, 11 reasons emitted) is the base; CT2n... (MarkovFyi/markov-program, 4 deploy txs, never emitted) is the predecessor.
2026-09-01  S0  IDL                  no on-chain IDL for either program (PDA absent). Checked-in IDL sha256 55c62d4b... verified against chain via event discriminators (8/8) and ActionRefused reason bytes (11/11).
2026-09-01  S0  BLOCKREASON          0..10 = OverTxCap OverDailyCap OverSpendCap OverSpendDailyCap ProgramNotAllowed TokenNotAllowed SlippageExceeded Expired Paused Revoked Unauthorized. Contradicts docs/10 §4 numbering and two names. Deployed program wins -> ADR-004 amendment needed before P02.
2026-09-01  S0  UPGRADE_AUTHORITY    2fpQ... holds 5o8E, demo_swap, demo_yield. Key custody not confirmed. -> D3 input, ADR-004.
2026-09-01  S0  HERMES               price endpoints 401 since 2026-08-26 (Pyth docs). Metadata endpoint 200. -> D2 changes shape: the *off-chain* path needs a key or provider too. ADR-003.
2026-09-01  S0  PYTH_DEVNET          receiver rec5... upgraded 2026-08-31 (slot 491006444), address unchanged; price-feed program pythW... unchanged since 2024.
2026-09-01  S0  PYTH_SDK             2.0.0 -> anchor ^1.0.2; 1.0.1 -> anchor ^0.31.1. Pack's "compatible up to 0.31.1" is stale. Build proof still pending (P04).
2026-09-01  S0  VERSIONS             pack says Anchor 1.0.0/1.0.1; crates.io says 1.1.2 stable, 2.0.0-rc.1. Recorded, not pinned.
2026-09-01  S0  HOSTING              existing Railway project `markov` (6 services, online) and Vercel deploys for the 5o8E stack found; indexer stalled (lag 1.2M slots, chainReady false). Not in the pack. -> D4, ADR-005.
2026-09-01  S0  EMERGENCY_KEY        EMERGENCY_KEY_JSON is a variable of the Railway `api` service, not `bot`. Violates docs/14 §2 separation for the existing stack. Gate B services must not inherit this.
2026-09-01  S0  DOMAIN               markov.trade delegated to Cloudflare, no records. D1 must be decided before any DNS is set.
2026-09-01  S0  COPY_GREP            landing/index.html clean; injected APY+venue -> 4 FAILs, exit 1. B15 tooling works as shipped.
2026-09-01  S0  PYTH_DEVNET_FEED     SOL/USD PriceUpdateV2 shard 0 = 7UVimffx... live on devnet, owner rec5..., Full, age 66 slots / 14 s at 20:39:33Z. Shards 1-2 stale, 3 absent. -> ADR-003 option A is viable; P08 can start with MARK_SOURCE=onchain. Day-long cadence unmeasured.
```
