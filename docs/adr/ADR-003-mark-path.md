# ADR-003 — Mark path (decision D2)

Status: **Accepted 2026-09-02** (Kunal: "for others decide on me"; decided per the Recommendation, see Decision at the end) · Date: 2026-09-01 · Seat: Protocol + Agents · Blocks: P04, P06, P07, **P08 (paper runner cannot start without a price source)**

## Context (verified 2026-09-01)

1. **Hermes now requires an API key.** `https://hermes.pyth.network/v2/updates/price/latest`, `/stream` and the v1 endpoint return `401 unauthorized`; Pyth's docs state authentication became mandatory on 2026-08-26 16:00 UTC. Only `/v2/price_feeds` (metadata) is still open. Keys come from Pyth Pro, or Hermes is served by node providers (Triton, P2P, extrnode, Liquify). `hermes-beta` behaves the same. The pack's assumption that the *off-chain* Hermes pull is free and unauthenticated is no longer true.
2. **Pyth on-chain addresses are unchanged** after the August upgrades observed on chain (receiver `rec5EKMGg6MxZYaMdyBfgwp4d5rB9T1VQH5pJv5LtFJ` upgraded at slot 491006444 ≈ 31 Aug; Wormhole bridge `HDwc…` 25 Aug; push-oracle / price-feed program `pythWSnswVUd12oZpeFP8e9CVaEqJg25g1Vtc2biRsT` unchanged since 2024). Both Pyth programs are listed for Solana Devnet in the docs and executable on devnet. The pack's "18 Aug" date is not something this session evidenced.
3. **SDK compatibility is not the blocker the pack feared:** `pyth-solana-receiver-sdk` 2.0.0 → anchor ^1.0.2; 1.0.1 → anchor ^0.31.1. A build has still not been run.
4. `SOL_USD_FEED_ID` = `ef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d`.
5. **A sponsored SOL/USD `PriceUpdateV2` account is live on devnet:** `7UVimffxr9ow1uXYxsr4LHAcV58mLzhmwaeKvJ1pjLiE` (shard 0), owned by the receiver, `verification_level = Full`, 66 slots / 14 s old when read on 2026-09-01T20:39Z. Readable for free over RPC with `posted_slot` for the freshness gate. Shards 1–2 are stale; shard 3 does not exist. **Cadence (5-minute sample): one update every ~50 s; age in slots median 195, p90 317, max 324.** Devnet runs at **≈ 165 ms per slot** today (FACTS `DEVNET_SLOT_MS`), so the pack's `max_mark_age_slots = 150` is ≈ 25 s and would veto most ticks, and 400 slots is only ≈ 66 s — about one pusher interval, with ~13 s of margin against the observed maximum. A window of ~2.5 minutes is ≈ 900 slots at this pacing, or the gate compares `Clock.unix_timestamp − publish_time` in seconds, which is robust to pacing changes. Full-day cadence is P08 day-1 output (`docs/FACTS.md` row `PYTH_DEVNET_FEED_ACCOUNT`).

Engineering pack v0.2 (`CORRECTIONS.md`, `docs/08` §3, `docs/14` §4) says the house `MarkAccount` is "the expected Gate B path" because "`pyth-solana-receiver-sdk` compat stops at Anchor 0.31.1". That premise is stale on crates.io (2.0.0 requires anchor-lang ^1.0.2, published 2026-06-15) and the on-chain devnet feed above removes the need for Hermes entirely. The fallback remains available; it is no longer the default.

## Options

**A. Pyth on-chain price-feed account as the mark, both off-chain and on-chain.** The agent, the paper runner and `demo_perps` all read the same devnet `PriceUpdateV2` account (price, exponent, publish_time, posted_slot). No Hermes, no key, freshness enforced on chain by `slot - posted_slot`. Depends on the devnet pusher actually updating that account at a useful cadence (verify: see FACTS). Requires the SDK (or a hand-written borsh reader for one account layout) to build against the D0 pin.

**B. House `MarkAccount` (the pack's fallback).** A `mark-poster` job pulls SOL/USD from an authenticated Hermes (Pyth Pro key or a provider) and writes `price, expo, slot, source="house"`. Page label: **house-posted devnet mark**. Freshness gate stays real. Cost: one key/provider account must be opened now, and the page says "house" not "oracle".

**C. B, but the poster reads the Pyth on-chain devnet account instead of Hermes.** No key needed; `source` can honestly say `pyth` because the number came from Pyth's own account; the house key only relays it. Falls back to `source="house"` from another feed only if the devnet account goes stale.

## Recommendation

**A**, with the freshness gate sized for ~2.5 minutes (≈ 900 slots at today's pacing, or a seconds-based comparison on `publish_time`), because the devnet pusher runs about once a minute; **C** if a 24-hour cadence measurement (P08's first day, logged per tick) shows gaps longer than that.

## Verification pass (2026-09-01)

- **A is coupled to D0.** With Anchor 0.31.1 the "compatible" SDK 1.0.1 does not compile out of the box (E0277 via `pythnet-sdk 2.3.1`); with Anchor 1.1.2 the SDK 2.0.0 compiles cleanly (`cargo check` by the protocol judge). Context item 3 above overstated 1.0.1.
- **The program must bind the account, not trust the operator:** check `owner == rec5…`, `feed_id == SOL/USD`, `verification_level == Full`; otherwise the operator can pass any 134-byte account (shards 1 and 2 of the same feed are stale) and set a house mark wearing a `pyth` label. These are P02/P04 invariant tests (`wrong_owner_mark_refused`, `wrong_feed_mark_refused`, `partial_verification_mark_refused`). The stale shards are a free `StaleOracle` redteam fixture.
- **C's poster must exist before the pusher dies, not after.** Shards 1 and 2 already went stale; if shard 0 stops, every tick vetoes `StaleOracle` and B4 goes red. P04 builds the house poster (relaying the Pyth account, `source` honest) regardless of which source is primary.
- **The page must show the threshold.** Against this pusher the honest chip reads ~35 s median / ~1 min p90, which a stranger reads as broken unless the threshold is next to it; `/v1/book/stats` needs `mark.max_age_slots` (or seconds) derived from policy, and `circuit` must flip to `stale_mark` at exactly that value. In both cases P08 starts immediately with `MARK_SOURCE=onchain` reading `7UVimffx…` over RPC, because the paper runner needs a slot-stamped price today and Hermes cannot provide one without a key. B is the last resort and requires Kunal to open a Pyth Pro or provider account first.

Whatever is chosen, the on-chain `source` field and the page label must match the actual origin of the number (`docs/08` §3, `docs/14` §4 accepted risk 4).

## Consequences

`MARK_SOURCE`, `MARK_MAX_AGE_SLOTS` and `MARK_POSTER_PUBKEY` in FACTS are filled by P04 after a signature exists. `HERMES_URL` stays recorded as "requires key" so nobody writes code against the open endpoint again.

## Decision (2026-09-02)

**Option A with C built as the fallback.** The mark is the devnet Pyth SOL/USD `PriceUpdateV2` account `7UVimffxr9ow1uXYxsr4LHAcV58mLzhmwaeKvJ1pjLiE`, read on chain by the program through `pyth-solana-receiver-sdk 2.0.0` and off chain by the agent and the paper runner through the same account over RPC. The program binds the account: `owner == rec5…`, `feed_id == ef0d8b6f…`, `verification_level == Full`, each with a named refusal test. **The freshness gate is in seconds, not slots** (`Clock::unix_timestamp − publish_time <= max_mark_age_secs`, policy default **150 s** ≈ three pusher intervals), because devnet pacing (≈165 ms/slot today) makes a slot count unstable; slots are still reported for display. `docs/10` gate 12 and the `Policy` field are amended accordingly in P02 (`max_mark_age_secs`). P04 also ships the house poster that relays the same Pyth account into a `MarkAccount` with `source = house`, so the book survives the sponsored pusher stopping; the page labels the live source honestly either way and shows the threshold next to the age. `MARK_SOURCE=onchain` for P08 from today.
