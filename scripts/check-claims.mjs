#!/usr/bin/env node
import { readFileSync, readdirSync, statSync } from "node:fs";
import { join } from "node:path";

const FORBIDDEN = [
  "best execution",
  "never get liquidated",
  "ai-powered trading",
  "exchange price 24/7",
  "nasdaq price 24/7",
  "jupiter perps",
  "illustrative",
  "demo mode",
  "mock mode",
];

const ROOTS = ["apps", "services", "packages"];
const OK = [".ts", ".tsx", ".js", ".mjs", ".md", ".css", ".html"];

function walk(dir, out = []) {
  for (const name of readdirSync(dir)) {
    if (name === "node_modules" || name === ".next" || name === "dist") continue;
    const p = join(dir, name);
    const st = statSync(p);
    if (st.isDirectory()) walk(p, out);
    else if (OK.some((e) => name.endsWith(e))) out.push(p);
  }
  return out;
}

let fails = 0;
for (const root of ROOTS) {
  let files = [];
  try {
    files = walk(root);
  } catch {
    continue;
  }
  for (const f of files) {
    if (f.includes("FACTS.md") || f.includes("check-claims") || f.includes("facts/src")) continue;
    const text = readFileSync(f, "utf8").toLowerCase();
    for (const phrase of FORBIDDEN) {
      if (text.includes(phrase)) {
        console.error(`FAIL ${f}: "${phrase}"`);
        fails++;
      }
    }
    if (/\bphoenix\b.{0,40}\b(devnet|testnet)\b/i.test(text) && !f.includes("FACTS") && !f.includes("PROGRAM.md") && !f.includes("STATUS")) {
      // allow the explicit negation
      if (!text.includes("not on devnet") && !text.includes("absent on devnet") && !text.includes("phoenix is not on devnet")) {
        console.error(`FAIL ${f}: Phoenix adjacent to devnet/testnet`);
        fails++;
      }
    }
  }
}
if (fails) {
  process.exit(1);
}
console.log("claims check ok");
