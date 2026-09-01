# ADR-003 — Mark path (decision D2)

Status: **Proposed — needs Kunal** · Date: 2026-09-01 · Seat: Protocol + Agents · Blocks: P04, P06, P07, **P08 (paper runner cannot start without a price source)**

## Context (verified 2026-09-01)

1. **Hermes now requires an API key.** `https://hermes.pyth.network/v2/updates/price/latest`, `/stream` and the v1 endpoint return `401 unauthorized`; Pyth's docs state authentication became mandatory on 2026-08-26 16:00 UTC. Only `/v2/price_feeds` (metadata) is still open. Keys come from Pyth Pro, or Hermes is served by node providers (Triton, P2P, extrnode, Liquify). `hermes-beta` behaves the same. The pack's assumption that the *off-chain* Hermes pull is free and unauthenticated is no longer true.
2. **Pyth on-chain addresses are unchanged** after the 18 Aug / 25 Aug / 31 Aug upgrades: receiver `rec5EKMGg6MxZYaMdyBfgwp4d5rB9T1VQH5pJv5LtFJ` (upgraded slot 491006444 ≈ 31 Aug), price-feed program `pythWSnswVUd12oZpeFP8e9CVaEqJg25g1Vtc2biRsT`. Both listed for Solana Devnet in the docs and executable on devnet.
3. **SDK compatibility is not the blocker the pack feared:** `pyth-solana-receiver-sdk` 2.0.0 → anchor ^1.0.2; 1.0.1 → anchor ^0.31.1. A build has still not been run.
4. `SOL_USD_FEED_ID` = `ef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d`.
5. **A sponsored SOL/USD `PriceUpdateV2` account is live on devnet:** `7UVimffxr9ow1uXYxsr4LHAcV58mLzhmwaeKvJ1pjLiE` (shard 0), owned by the receiver, `verification_level = Full`, 66 slots / 14 s old when read on 2026-09-01T20:39Z. Readable for free over RPC with `posted_slot` for the freshness gate. Shards 1–2 are stale; shard 3 does not exist. **Cadence (5-minute sample): one update every ~50 s; age in slots median 195, p90 317, max 324.** The pack's example `max_mark_age_slots = 150` would veto most ticks against this feed; a policy value around 400 slots (~2.5 minutes, i.e. two or three pusher intervals) fits. Full-day cadence is P08 day-1 output (`docs/FACTS.md` row `PYTH_DEVNET_FEED_ACCOUNT`).

## Options

**A. Pyth on-chain price-feed account as the mark, both off-chain and on-chain.** The agent, the paper runner and `demo_perps` all read the same devnet `PriceUpdateV2` account (price, exponent, publish_time, posted_slot). No Hermes, no key, freshness enforced on chain by `slot - posted_slot`. Depends on the devnet pusher actually updating that account at a useful cadence (verify: see FACTS). Requires the SDK (or a hand-written borsh reader for one account layout) to build against the D0 pin.

**B. House `MarkAccount` (the pack's fallback).** A `mark-poster` job pulls SOL/USD from an authenticated Hermes (Pyth Pro key or a provider) and writes `price, expo, slot, source="house"`. Page label: **house-posted devnet mark**. Freshness gate stays real. Cost: one key/provider account must be opened now, and the page says "house" not "oracle".

**C. B, but the poster reads the Pyth on-chain devnet account instead of Hermes.** No key needed; `source` can honestly say `pyth` because the number came from Pyth's own account; the house key only relays it. Falls back to `source="house"` from another feed only if the devnet account goes stale.

## Recommendation

**A**, with `MARK_MAX_AGE_SLOTS ≈ 400` (not the pack's 150) because the devnet pusher runs about once a minute; **C** if a 24-hour cadence measurement (P08's first day, logged per tick) shows gaps longer than that. In both cases P08 starts immediately with `MARK_SOURCE=onchain` reading `7UVimffx…` over RPC, because the paper runner needs a slot-stamped price today and Hermes cannot provide one without a key. B is the last resort and requires Kunal to open a Pyth Pro or provider account first.

Whatever is chosen, the on-chain `source` field and the page label must match the actual origin of the number (`docs/08` §3, `docs/14` §4 accepted risk 4).

## Consequences

`MARK_SOURCE`, `MARK_MAX_AGE_SLOTS` and `MARK_POSTER_PUBKEY` in FACTS are filled by P04 after a signature exists. `HERMES_URL` stays recorded as "requires key" so nobody writes code against the open endpoint again.
