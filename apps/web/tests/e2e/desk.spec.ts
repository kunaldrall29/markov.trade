import { expect, test } from "@playwright/test";
import { resetScenario, setScenario, usingFixture } from "./helpers";

test.beforeEach(async () => {
  if (usingFixture) await resetScenario();
});

test("desk reads the house book from the chain and shows receipts verbatim", async ({ page }) => {
  await page.goto("/book");
  const desk = page.getByTestId("desk");
  await expect(desk.getByTestId("stat-strip")).toBeVisible();
  await expect(desk.getByTestId("circuit")).toBeVisible();
  await expect(desk.getByTestId("receipt-list")).toBeVisible();
  // Every row links to the explorer on devnet.
  const links = desk.locator("[data-testid=receipt-row] a[href]");
  const n = await links.count();
  for (let i = 0; i < n; i += 1) expect(await links.nth(i).getAttribute("href")).toMatch(/explorer\.solana\.com\/tx\/.*cluster=devnet/);
  if (usingFixture) {
    await expect(desk.getByTestId("stat-strip")).toContainText("100.00");
    const verdicts = desk.getByTestId("receipt-verdict");
    await expect(verdicts.filter({ hasText: "OverTxCap" })).toHaveCount(1);
    await expect(verdicts.filter({ hasText: "SlippageExceeded" })).toHaveCount(1);
    await expect(desk.getByTestId("circuit")).toHaveText("circuit live");
  }
});

test("withdraw-enabled-in-every-state", async ({ page }) => {
  test.skip(!usingFixture, "flips chain state through the fixture rpc");
  for (const state of ["Active", "Paused", "Revoked"] as const) {
    await setScenario({ state });
    await page.goto("/book");
    await expect(page.getByTestId("desk")).toContainText(state.toLowerCase());
    const withdraw = page.getByTestId("verb-withdraw");
    await expect(withdraw).toBeVisible();
    await expect(withdraw).toBeEnabled();
    expect(await withdraw.getAttribute("disabled")).toBeNull();
    await withdraw.click();
    await expect(page.getByRole("dialog")).toContainText("Withdraw");
    await page.keyboard.press("Escape");
  }
  // And with the chain unreadable: banner up, counters greyed, withdraw still on.
  await setScenario({ down: true });
  await page.goto("/book");
  await expect(page.getByTestId("degraded-banner")).toBeVisible();
  const withdraw = page.getByTestId("verb-withdraw");
  await expect(withdraw).toBeEnabled();
  await withdraw.click();
  await expect(page.getByRole("dialog")).toContainText("Withdraw");
  await setScenario({ down: false });
});

test("paused and global halt show on the circuit chip", async ({ page }) => {
  test.skip(!usingFixture, "flips chain state through the fixture rpc");
  await setScenario({ state: "Paused" });
  await page.goto("/book");
  await expect(page.getByTestId("circuit")).toHaveText("paused");
  await setScenario({ state: "Active", halt: true });
  await page.goto("/book");
  await expect(page.getByTestId("circuit")).toHaveText("global halt");
  await setScenario({ halt: false });
});
