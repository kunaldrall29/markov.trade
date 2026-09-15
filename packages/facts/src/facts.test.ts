import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import assert from "node:assert/strict";
import * as facts from "./index.ts";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "../../..");
const md = readFileSync(resolve(root, "docs/FACTS.md"), "utf8");

test("every exported pubkey/url appears in FACTS.md", () => {
  const must = [
    facts.PROGRAM_ID,
    facts.DRIFT_V2_PROGRAM,
    facts.PHOENIX_PROD,
    facts.SUBSCRIPTIONS_PROGRAM,
    facts.PACIFICA_REST,
    facts.PHOENIX_REST,
    facts.USDC_MAINNET,
    facts.TSLAX,
    facts.NVDAX,
    facts.SPYX,
    facts.AAPLX,
    facts.JUPITER_PERPS,
  ];
  for (const v of must) {
    assert.ok(md.includes(v), `missing from FACTS.md: ${v}`);
  }
});

test("Jupiter Perps is listed only as excluded", () => {
  assert.match(md, /not integrated/i);
});
