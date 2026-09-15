import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import * as facts from "./index.ts";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "../../..");
const md = readFileSync(resolve(root, "docs/FACTS.md"), "utf8");
const must = [
  facts.PROGRAM_ID,
  facts.DRIFT_V2_PROGRAM,
  facts.PHOENIX_PROD,
  facts.SUBSCRIPTIONS_PROGRAM,
  facts.PACIFICA_REST,
  facts.PACIFICA_TESTNET,
  facts.PHOENIX_REST,
  facts.USDC_MAINNET,
  facts.TSLAX,
  facts.NVDAX,
  facts.JUPITER_SWAP_QUOTE,
];
const missing = must.filter((v) => !md.includes(v));
if (missing.length) {
  console.error("facts drift — constants not in docs/FACTS.md:\n", missing.join("\n"));
  process.exit(1);
}
console.log("facts check ok", must.length, "constants");
