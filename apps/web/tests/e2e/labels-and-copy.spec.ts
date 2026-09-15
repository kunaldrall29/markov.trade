import { expect, test } from "@playwright/test";
import { noHorizontalOverflow } from "./helpers";

// B1/B15: every page carries the stage line above the fold, no page carries
// a sample tape, and the layout holds at phone width.
for (const path of ["/", "/book", "/receipts", "/account", "/paper"]) {
  test(`labels-and-copy ${path}`, async ({ page }) => {
    await page.goto(path);
    await expect(page.getByTestId("stage-line").first()).toContainText("devnet · marked PnL, not a promised rate");
    const text = (await page.locator("body").innerText()).toLowerCase();
    expect(text).not.toContain("sample tape");
    expect(text).not.toContain("not a live feed");
    if ((page.viewportSize()?.width ?? 1000) >= 768) await expect(page.getByRole("navigation", { name: "Primary" }).getByRole("link", { name: "Activity" })).toBeVisible();
    await noHorizontalOverflow(page);
  });
}

test("mobile menu opens and lists every section", async ({ page, isMobile }) => {
  test.skip(!isMobile, "mobile project only");
  await page.goto("/");
  await expect(page.getByTestId("connect-wallet")).toBeVisible(); // rendered after hydration
  await page.getByRole("button", { name: "Open menu" }).click();
  for (const label of ["Home", "Desk", "Activity", "Account", "Paper"]) {
    await expect(page.getByRole("navigation", { name: "Mobile" }).getByRole("link", { name: label })).toBeVisible();
  }
});
