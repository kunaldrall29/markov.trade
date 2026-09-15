# FACTS — Markov control plane

Last full verification: **2026-09-15**. Verified by this session against live HTTP/RPC. A row without a date and a source is not a fact. A constant used in code must appear here; `pnpm check:facts` fails on drift.

Retired: the Gate B “Markov Book” programs (`25CdYaZe…`, `3Zcd8Xs…`, `5o8E…`) and the Grok PWA at `apps/web`. They are not this product.

## 1. Cluster and toolchain

| Key | Value | Verified | Source |
| --- | --- | --- | --- |
| DEVNET_GENESIS | `EtWTRABZaYq6iMfeYKouRu166VU2xqa1wcaWoxPkrZBG` | 2026-09-15 | `getGenesisHash` `https://api.devnet.solana.com` |
| MAINNET_GENESIS | `5eykt4UsFv8P8NJdTREpY1vzqKqZKvdpKuc147dw2N9d` | 2026-09-15 | `getGenesisHash` `https://api.mainnet-beta.solana.com` |
| DEVNET_SLOT | 498679550 (sample) | 2026-09-15 | `getSlot` public devnet |
| MAINNET_SLOT | 447204782 (sample) | 2026-09-15 | `getSlot` public mainnet-beta |
| ANCHOR_VERSION | 1.2.0 | 2026-09-15 | `Anchor.toml` + `programs/markov` (host `cargo test -p markov --lib` green) |
| SOLANA_CLI_PIN | Agave 4.2.2 | 2026-09-15 | prior ADR-001; this box did not have the CLI installed at session start |
| RPC_DEVNET | `https://api.devnet.solana.com` (fallback). Preferred primary when available: `https://rpc.magicblock.app/devnet` (2026-09-02 survey). | 2026-09-15 / 2026-09-02 | this session + prior FACTS survey |
| RPC_MAINNET | `https://api.mainnet-beta.solana.com` (public; rate-limited). Production requires a paid provider. | 2026-09-15 | `getGenesisHash` |

## 2. Markov program

| Key | Value | Verified | Source |
| --- | --- | --- | --- |
| PROGRAM_ID | `37rW4ETzh8o7ebWFnrJRKt7iCUvWdxsv37vPENjAYz1J` | 2026-09-15 | keypair `keys/markov-program.json` (gitignored). **Not deployed** until `solana program show` returns it. |
| PROGRAM_ID_STATUS | generated, undeployed | 2026-09-15 | no `getAccountInfo` executable at this address yet |
| UPGRADE_AUTHORITY_DEVNET | deployer key (single) until Squads is wired | 2026-09-15 | Conventions §5; mainnet must be 2-of-3 Squads |
| ACCOUNT_SEEDS | `account`+owner · `mandate`+account+version_le_u32 · `invest`+account+version_le_u32 · `perm`+account+actor · `receipt`+account+request_id_le_u128 · `day`+account+day_epoch_le_u32 · `config` | 2026-09-15 | `programs/markov/src/state.rs` |

## 3. Venue programs and HTTP

| Key | Value | Verified | Source |
| --- | --- | --- | --- |
| DRIFT_V2_PROGRAM | `dRiftyHA39MWEi3m9aunc5MzRF1JYuBsbn6VPcn33UH` — executable on **mainnet and devnet** | 2026-09-15 | `getAccountInfo` both clusters, owner BPF loader |
| PHOENIX_PROD | `EtrnLzgbS7nMMy5fbD42kXiUzGg8XQzJ972Xtk1cjWih` — executable **mainnet only**; **absent on devnet** | 2026-09-15 | `getAccountInfo` |
| PHOENIX_BETA | `phDEVv4w6BcfkLrLNeXr8HhhgQxnxziVGXpGPcaadMf` | 2026-09-10 | prior Perps K8; not re-probed this session |
| HAWKEYE | `RiSeVw3ZjNfsaXPRb4mgaqYaEEt41pNNJoDvVh7pgQj` | 2026-09-10 | prior Perps K8 |
| EMBER | `EMBERpYNE6ehWmXymZZS2skiFmCa9V5dp14e1iduM5qy` | 2026-09-10 | Conventions |
| FLIGHT | `F1ightu9cujFYo34k9CabifLrJT8qzfDVM2Q7BqhJn2W` | 2026-09-10 | Conventions |
| PHOENIX_REST | `https://perp-api.phoenix.trade` | 2026-09-15 | `GET /v1/view/exchange/markets` → 82 markets; `GET /v1/view/exchange/market/SOL` 200; `GET /v1/view/orderbook/SOL` 200 (slot 447204976, bids/asks `[price, size]`) |
| PHOENIX_WS | `wss://perp-api.phoenix.trade/v1/ws` | 2026-09-15 | docs.phoenix.trade (not handshake-probed this session) |
| PHOENIX_TAKER_FEE_SOL | 3.5 bps (`takerFee` 0.00035) / maker 0.5 bps | 2026-09-15 | `/v1/view/exchange/market/SOL` |
| PHOENIX_SYMBOLS | venue symbols are `SOL`, `BTC`, `ETH`, `DOGE`, `FARTCOIN`, `PUMP`, `GOLD` — **not** `SOL-PERP` | 2026-09-15 | markets list; `/v1/view/orderbook/SOL-PERP` → 404 |
| PACIFICA_REST | `https://api.pacifica.fi/api/v1` | 2026-09-15 | `GET /info` → **77** perps; `GET /info/prices` 200; `GET /book?symbol=SOL&agg_level=1` 200 |
| PACIFICA_TESTNET | `https://test-api.pacifica.fi/api/v1` | 2026-09-15 | `GET /info` → **88** markets |
| PACIFICA_SIGNING | Ed25519 over compact canonical JSON `{data, expiry_window, timestamp, type}` | 2026-09-15 | docs.pacifica.fi signing implementation |
| PACIFICA_FEES_L0 | maker 1.5 bps / taker 4.0 bps at level 0 | 2026-09-15 | `GET /info/fees` |
| PACIFICA_KLINE | `GET /kline?symbol&interval&start_time&end_time` — SOL 1h bars with o/h/l/c/v | 2026-09-15 | test-api 200 this session |
| PACIFICA_ACCOUNT | `GET /account?account=` — 404 `Account not found` if unregistered | 2026-09-15 | test-api |
| PACIFICA_POSITIONS | `GET /positions?account=` — 200 `[]` if none | 2026-09-15 | test-api |
| PACIFICA_ORDERS_CREATE | `POST /orders/create` signed compact JSON (`create_order`) | 2026-09-15 | docs.pacifica.fi create-limit-order |
| PACIFICA_ORDERS_CANCEL | `POST /orders/cancel` signed compact JSON (`cancel_order`); `symbol` plus `order_id` or `client_order_id` | 2026-09-15 | docs.pacifica.fi cancel-order |
| PACIFICA_MIN_ORDER | `min_order_size` is `"10"` on SOL/BTC/ETH/DOGE testnet. Same number across lot sizes; treated as **USD notional**, not base size, until Pacifica docs say otherwise. | 2026-09-15 | `/info` |
| PACIFICA_SOL_MARK | 100.324 USD, ts 1789461043710, funding −0.00000358 | 2026-09-15 | `/info/prices` |
| PACIFICA_OVERLAP | SOL, BTC, ETH, DOGE, FARTCOIN, PUMP present on Pacifica mainnet | 2026-09-15 | `/info` symbol set |
| JUPITER_PERPS | `PERPHjGBqRHArX4DySjwM6UJHiR3sWAatqfdBS2qQJu` — **not integrated** | 2026-09-10 | Conventions; listed so nobody routes to it |
| JUPITER_TOKENS_V2 | `https://lite-api.jup.ag/tokens/v2/search` | 2026-09-15 | TSLAx / NVDAx 200 |
| JUPITER_SWAP_V1 | `https://lite-api.jup.ag/swap/v1/quote` (+ `swap` / `swap-instructions`) | 2026-09-15 | $5 USDC→NVDAx quote 200, `priceImpactPct` 0, `outAmount` 2359104, `contextSlot` present |
| SUBSCRIPTIONS_PROGRAM | `De1egAFMkMWZSN5rYXRj9CAdheBamobVNubTsi9avR44` — executable mainnet **and** devnet | 2026-09-15 | `getAccountInfo` |
| TOKEN_2022 | `TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb` | 2026-09-15 | TSLAx mint owner |

## 4. Mints

| Key | Value | Verified | Source |
| --- | --- | --- | --- |
| USDC_MAINNET | `EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v` SPL Token, decimals 6 | 2026-09-15 | `getAccountInfo` mainnet; **absent as USDC on devnet** (system-owned empty) |
| TSLAx | `XsDoVfqeBukxuZHWhdvWHBhgEHjGNst4MLodqsJHzoB` Token-2022, 8 decimals | 2026-09-15 | Jupiter tokens v2 + mint owner Token-2022 mainnet; **absent on devnet** |
| NVDAx | `Xsc9qvGR1efVDFGLrVsmkzv3qi45LTBjeUKSPmx9qEh` Token-2022, 8 decimals | 2026-09-15 | Jupiter tokens v2 |
| SPYx | `XsoCS1TfEyfFhfvj8EtZ528L3CaKBDBRqRapnBbDF2W` | 2026-09-11 | Invest FACTS pass #2 (not re-fetched this session) |
| AAPLx | `XsbEhLAtcf6HdfpFZ5xEMdqW8nfAvcsP5bdudRLJzJp` | 2026-09-11 | Invest FACTS pass #2 |
| XSTOCKS_RESTRICTIONS | not for US persons; not currently UK; India unconfirmed; issuer pause + permanent delegate | 2026-09-11 | issuer docs (Invest pass #2) |

## 5. Caps (code, Conventions §7)

| Key | Value |
| --- | --- |
| PER_ACCOUNT_GROSS_NOTIONAL_USD | 2000 |
| PER_POSITION_LEVERAGE_MAJORS | 2.0 |
| INVEST_MONTHLY_BUDGET_USD | 500 |
| INVEST_EXECUTION_USD | 5–100 |
| INVEST_KEEPER_GLOBAL_DAILY_USD | 2000 |
| FRESHNESS_PACIFICA_MS | 3000 |
| FRESHNESS_DRIFT_SLOTS | 2 |
| FRESHNESS_PHOENIX_MS | 3000 |
| FRESHNESS_JUPITER_QUOTE_MS | 10000 |
| FRESHNESS_OFFICIAL_PRICE_MS | 15000 |
| INVEST_MAX_COST_BPS_DEFAULT | 40 |

## 6. Canonical markets (this build)

Venue-neutral id is 16 ASCII bytes, NUL-padded (`SOL-PERP`). Venue symbols differ.

| Canonical | Pacifica | Phoenix | Drift index (SDK constants; live decode still required) |
| --- | --- | --- | --- |
| SOL-PERP | SOL | SOL | 0 |
| BTC-PERP | BTC | BTC | 1 |
| ETH-PERP | ETH | ETH | 2 |
| DOGE-PERP | DOGE | DOGE | 7 (confirm on chain before routing) |
| FARTCOIN-PERP | FARTCOIN | FARTCOIN | confirm |
| PUMP-PERP | PUMP | PUMP | confirm |

Drift hosted HTTP (`dlob.drift.trade`, `data.api.drift.trade`, `mainnet-beta.api.drift.trade`) **did not resolve** from this network on 2026-09-15. Drift market state in this build is read via Solana RPC + `@drift-labs/sdk` when the RPC is reachable; otherwise the adapter fails closed (`STALE_MARKET_DATA`). Do not substitute Pacifica/Phoenix marks for Drift.

## 7. Stage

| Key | Value |
| --- | --- |
| STAGE | `TEST STAGE` |
| EXECUTABLE_VENUES | Pacifica **testnet** (writes after owner `signMessage`); Drift **devnet** (owner-signed tx) once RPC+SDK subscribe; Phoenix **not executable** (mainnet read-only) |
| INVEST_EXECUTION | Jupiter **mainnet** only (no xStocks on devnet). Devnet keeper path uses `record_skip` plus a named `devnet_swap_stub` program id in config — never described as a fill. |

## 8. Reason codes (u16, frozen)

1 MARKET_NOT_ALLOWED · 2 MAX_LEVERAGE_EXCEEDED · 3 MAX_NOTIONAL_EXCEEDED · 4 MIN_SAFETY_BUFFER · 5 DAILY_LOSS_BUDGET_EXCEEDED · 6 STALE_MARKET_DATA · 7 SLIPPAGE_LIMIT · 8 ACTOR_SCOPE_DENIED · 9 ACTOR_CAP_EXCEEDED · 10 VENUE_NOT_ALLOWED · 11 ACCOUNT_PAUSED · 12 GLOBAL_PAUSED · 20 ASSET_NOT_ALLOWLISTED · 21 BUDGET_EXHAUSTED · 22 MONTHLY_CEILING_EXCEEDED · 23 RESERVE_FLOOR · 24 COST_LIMIT · 25 MARKET_CLOSED · 26 OFF_HOURS_DEVIATION · 27 ASSET_PAUSED · 28 NO_ROUTE · 29 STALE_QUOTE · 30 DELEGATION_INSUFFICIENT · 31 SINGLE_ASSET_CAP · 100 EXECUTION_CONFIRMED · 101 EXECUTION_FAILED

## 9. Claims that are false here

- Phoenix on devnet/testnet
- Jupiter Perps in the route set
- “Best execution”, guaranteed fills, “never get liquidated”
- Tokenized stocks as shares; 24/7 DEX price as the exchange print
- Any illustrative/demo balances in the Terminal
