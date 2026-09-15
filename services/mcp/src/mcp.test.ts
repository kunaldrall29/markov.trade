import { test } from "node:test";
import assert from "node:assert/strict";
import { allowedTools, absentTools } from "./index.ts";

test("forbidden tools are absent", () => {
  for (const t of absentTools()) {
    assert.equal(allowedTools().includes(t as never), false);
  }
  assert.ok(!allowedTools().some((t) => t.includes("withdraw") || t.endsWith(".execute")));
});
