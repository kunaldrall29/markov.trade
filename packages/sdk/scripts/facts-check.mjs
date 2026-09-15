#!/usr/bin/env node
// Every literal in src/facts.ts must appear verbatim in docs/FACTS.md.
// A constant that FACTS does not carry is a constant typed from memory.
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const facts = readFileSync(join(here, "..", "..", "..", "docs", "FACTS.md"), "utf8");
const src = readFileSync(join(here, "..", "src", "facts.ts"), "utf8");

const literals = [...src.matchAll(/address\("([1-9A-HJ-NP-Za-km-z]{32,44})"\)/g)].map((m) => m[1]);
literals.push(...[...src.matchAll(/"(https?:\/\/[^"]+|wss:\/\/[^"]+)"/g)].map((m) => m[1]));
literals.push(...[...src.matchAll(/"([0-9a-f]{64})"/g)].map((m) => m[1]));

const missing = literals.filter((v) => !facts.includes(v));
if (missing.length) {
  console.error("facts-check: not found in docs/FACTS.md:\n  " + missing.join("\n  "));
  process.exit(1);
}
console.log(`facts-check: ${literals.length} literals present in docs/FACTS.md`);
