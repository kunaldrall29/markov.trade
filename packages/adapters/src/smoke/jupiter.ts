import { NVDAX, USDC_MAINNET } from "@markov/facts";
import { jupiterQuote, jupiterToken } from "../jupiter.ts";

const tok = await jupiterToken("NVDAx");
if (!tok || tok.id !== NVDAX) throw new Error(`NVDAx mint drift: ${tok?.id}`);
if (!tok.tokenProgram.includes("Tokenz")) throw new Error("NVDAx is not Token-2022");
const q = await jupiterQuote({ outputMint: NVDAX, amount: 5_000_000 });
if (q.inputMint !== USDC_MAINNET) throw new Error("quote input");
if (!(Number(q.outAmount) > 0)) throw new Error("zero outAmount");
console.log(
  JSON.stringify({
    ok: true,
    inputMint: q.inputMint,
    outputMint: q.outputMint,
    inAmount: q.inAmount,
    outAmount: q.outAmount,
    priceImpactPct: q.priceImpactPct,
    contextSlot: q.contextSlot,
    costBps: q.costBps,
  }),
);
