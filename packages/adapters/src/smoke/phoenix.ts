import { PHOENIX_REST } from "@markov/facts";
import { phoenixMarkets, phoenixBook } from "../phoenix.ts";

const markets = await phoenixMarkets();
if (markets.length < 10) throw new Error(`phoenix market count ${markets.length}`);
const sol = markets.find((m) => m.symbol === "SOL");
if (!sol) throw new Error("phoenix missing SOL");
if (sol.takerFee !== 0.00035) throw new Error(`unexpected SOL takerFee ${sol.takerFee}`);
const book = await phoenixBook("SOL");
if (!book.bids[0] || !book.asks[0]) throw new Error("empty phoenix SOL book");
console.log(
  JSON.stringify({
    ok: true,
    source: PHOENIX_REST,
    markets: markets.length,
    symbol: "SOL",
    slot: book.slot,
    bid: book.bids[0].price,
    ask: book.asks[0].price,
    taker_fee_bps: sol.takerFee * 10_000,
    executable: false,
  }),
);
