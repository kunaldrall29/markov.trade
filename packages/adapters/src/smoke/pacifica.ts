import { PACIFICA_REST } from "@markov/facts";
import { pacificaPrices, pacificaBook, buildPacificaSignable } from "../pacifica.ts";

const prices = await pacificaPrices("mainnet");
const sol = prices.find((p) => p.symbol === "SOL");
if (!sol) throw new Error("no SOL on Pacifica");
if (!(Number(sol.mark) > 0)) throw new Error("SOL mark not positive");
const age = Date.now() - Number(sol.timestamp);
if (age > 60_000) throw new Error(`pacifica SOL print older than 60s: ${age}`);
const book = await pacificaBook("SOL", "mainnet");
if (!book.bids[0] || !book.asks[0]) throw new Error("empty SOL book");
const signed = buildPacificaSignable({
  type: "create_order",
  account: "11111111111111111111111111111111",
  timestamp: 1_748_970_123_456,
  expiry_window: 5_000,
  data: { amount: "0.1", price: "100000", side: "bid", symbol: "BTC", tif: "GTC", reduce_only: false, client_order_id: "x" },
});
if (!signed.compactJson.includes('"type":"create_order"')) throw new Error("signable json");
console.log(
  JSON.stringify({
    ok: true,
    source: PACIFICA_REST,
    symbol: "SOL",
    mark: sol.mark,
    age_ms: age,
    bid: book.bids[0].price,
    ask: book.asks[0].price,
  }),
);
