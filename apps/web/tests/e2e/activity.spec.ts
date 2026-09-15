import { expect, test } from "@playwright/test";
import { usingFixture } from "./helpers";

test("activity feed decodes receipts, filters refusals and links the explorer", async ({ page }) => {
  await page.goto("/receipts");
  await expect(page.getByTestId("connect-wallet")).toBeVisible(); // rendered after hydration; the filters need handlers
  await expect(page.getByTestId("receipt-list")).toBeVisible();
  await page.getByTestId("filter-refused").click();
  const rows = page.getByTestId("receipt-row");
  const n = await rows.count();
  for (let i = 0; i < n; i += 1) expect(await rows.nth(i).getAttribute("data-kind")).toBe("refusal");
  if (usingFixture) {
    await expect(rows).toHaveCount(2);
    await expect(page.getByTestId("receipt-verdict").first()).toHaveText("OverTxCap");
    await page.getByTestId("filter-owner").click();
    await expect(page.getByTestId("receipt-row")).toHaveCount(2);
  }
  await page.getByTestId("filter-all").click();
});

test("read api: shapes carry slot and source, and no rate field exists", async ({ request }) => {
  const stats = await request.get("/api/v1/book/stats");
  expect(stats.ok()).toBeTruthy();
  const body = await stats.text();
  const json = JSON.parse(body);
  expect(json.source).toBe("chain");
  expect(json.env).toBe("devnet");
  expect(typeof json.data_slot).toBe("string");
  expect(json.mandate.withdraw_enabled).toBe(true);
  expect(json.enforcement).toEqual({ delta: "offchain", gross: "offchain", daily_loss: "offchain" });
  expect(body).not.toMatch(/"(apy|apr|projected_[a-z_]*)"/i);
  for (const m of [json.mandate.vault, json.mandate.policy.per_tx_cap]) {
    expect(typeof m.raw).toBe("string");
    expect(typeof m.decimals).toBe("number");
    expect(typeof m.mint).toBe("string");
  }
  const receipts = await request.get("/api/v1/receipts?limit=5");
  expect(receipts.ok()).toBeTruthy();
  const r = await receipts.json();
  expect(r.source).toBe("chain");
  expect(Array.isArray(r.receipts)).toBeTruthy();
  const health = await request.get("/api/health");
  const h = await health.json();
  expect(typeof h.chainReady).toBe("boolean");
  expect(Array.isArray(h.failing)).toBeTruthy();
  const bad = await request.get("/api/v1/receipts?mandate=not-an-address");
  expect(bad.status()).toBe(400);
});
