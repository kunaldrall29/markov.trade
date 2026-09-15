import { expect, test } from "@playwright/test";
import { captured, connectTestWallet, reownFixtureTo, resetScenario, setScenario, usingFixture } from "./helpers";

// Anchor discriminators from the checked-in IDL (docs/idl/mandate-25CdYaZe.json).
const DISC = {
  fund: [218, 188, 111, 221, 152, 113, 174, 7],
  owner_withdraw: null as number[] | null,
};
const PROGRAM = "25CdYaZeB18QvUR7cTyZPgTZPNREb7t6xL8zmk1eXAU6";
const ATA_PROGRAM = "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL";
const MANDATE = "2ivTE7hwgW9nzCQzRXm2ED1htArg37BgZXhLyAh2TMTo";
const VAULT = "Am8xa8dQxVUKp7CYwKnXkFRYn6G3NeAVWmKaSF9mR6Yu";

function disc(ix: { data: number[] }) {
  return ix.data.slice(0, 8);
}

test.describe("account", () => {
  test.beforeEach(async () => {
    if (usingFixture) await resetScenario();
  });

  test("connect, see your mandates, build every owner verb against the real program", async ({ page }) => {
    test.skip(!usingFixture, "needs the fixture rpc to re-own the mandate to the test wallet");
    await page.goto("/account");
    const address = await connectTestWallet(page);
    // Re-own the fixture mandate to the test wallet so getProgramAccounts finds it.
    await reownFixtureTo(address);
    await page.reload();
    await expect(page.getByTestId("wallet-menu")).toBeVisible();
    await expect(page.getByTestId("mandate-list")).toBeVisible();
    const card = page.getByTestId("mandate-card").first();
    await expect(card).toHaveAttribute("data-mandate", MANDATE);
    await expect(card.getByTestId("mandate-state")).toContainText("active");
    await expect(card).toContainText("Your rules");
    await expect(card).toContainText("50.00 USDC-d"); // per-trade cap from the Gate B template
    await expect(page.getByTestId("owner-usdc")).toHaveText("250.00");

    // Fund: amount, submit, the wallet captures the transaction and the UI shows the wallet's words verbatim.
    await card.getByTestId("verb-fund").click();
    await page.getByTestId("verb-amount").fill("7");
    await page.getByTestId("verb-submit").click();
    await expect(page.getByRole("dialog")).toContainText("capture mode");
    let txs = await captured(page);
    expect(txs).toHaveLength(1);
    const fund = txs[0]!.instructions.find((ix) => ix.programAddress === PROGRAM)!;
    expect(disc(fund)).toEqual(DISC.fund);
    expect(fund.accounts).toContain(MANDATE);
    expect(fund.accounts).toContain(VAULT);
    expect(txs[0]!.feePayer).toBe(address);
    await page.keyboard.press("Escape");

    // Withdraw: max button fills the vault balance; the tx carries an idempotent ATA create then owner_withdraw.
    await card.getByTestId("verb-withdraw").click();
    await page.getByTestId("amount-max").click();
    await expect(page.getByTestId("verb-amount")).toHaveValue("100");
    await page.getByTestId("verb-submit").click();
    await expect(page.getByRole("dialog")).toContainText("capture mode");
    txs = await captured(page);
    expect(txs).toHaveLength(2);
    const programs = txs[1]!.instructions.map((ix) => ix.programAddress);
    expect(programs).toContain(ATA_PROGRAM);
    expect(programs[programs.length - 1]).toBe(PROGRAM);
    await page.keyboard.press("Escape");

    // Pause and revoke name their consequence before anything is signed.
    await card.getByTestId("verb-pause").click();
    await expect(page.getByRole("dialog")).toContainText("Only you can unpause");
    await page.getByTestId("verb-submit").click();
    await expect(page.getByRole("dialog")).toContainText("capture mode");
    await page.keyboard.press("Escape");
    await card.getByTestId("verb-revoke").click();
    await expect(page.getByRole("dialog")).toContainText("There is no un-revoke");
    await page.getByTestId("verb-submit").click();
    await expect(page.getByRole("dialog")).toContainText("capture mode");
    await page.keyboard.press("Escape");
    txs = await captured(page);
    expect(txs).toHaveLength(4);
    for (const t of txs) expect(t.instructions.some((ix) => ix.programAddress === PROGRAM)).toBeTruthy();

    // Amend refuses to widen and builds when tightening.
    await card.getByTestId("verb-amend").click();
    await page.getByTestId("am-pertx").fill("60");
    await expect(page.getByTestId("amend-rules")).toContainText("would widen: per-trade cap");
    await expect(page.getByTestId("am-submit")).toBeDisabled();
    await page.getByTestId("am-pertx").fill("40");
    await expect(page.getByTestId("am-submit")).toBeEnabled();
    await page.getByTestId("am-submit").click();
    await expect(page.getByTestId("amend-rules")).toContainText("capture mode");
    txs = await captured(page);
    expect(txs).toHaveLength(5);

    // New mandate: the create instruction derives the next nonce.
    await page.getByTestId("new-mandate").click();
    await expect(page.getByTestId("create-mandate")).toContainText("nonce 11");
    await page.getByTestId("cm-submit").click();
    await expect(page.getByTestId("create-mandate")).toContainText("capture mode");
    txs = await captured(page);
    expect(txs).toHaveLength(6);
    expect(txs[5]!.instructions.some((ix) => ix.programAddress === PROGRAM)).toBeTruthy();

    // Disconnect forgets the wallet; the page goes back to the connect prompt.
    await page.getByTestId("wallet-menu").click();
    await page.getByTestId("disconnect-wallet").click();
    await expect(page.getByTestId("connect-wallet")).toBeVisible();
    await setScenario({ owner: null, ownerAta: null });
  });

  test("a wallet with no mandates gets an honest empty state", async ({ page }) => {
    test.skip(!usingFixture, "fixture-only: a fresh key has no mandates there by construction");
    await page.goto("/account");
    await connectTestWallet(page);
    await expect(page.getByTestId("no-mandates")).toBeVisible();
    await expect(page.getByTestId("no-mandates")).toContainText("no mandates for this wallet on devnet");
  });

  test("the desk's verbs refuse to act for a wallet that is not the owner", async ({ page }) => {
    test.skip(!usingFixture, "fixture-only");
    await page.goto("/book");
    await connectTestWallet(page);
    await page.getByTestId("verb-withdraw").click();
    await expect(page.getByRole("dialog")).toContainText("does not own this mandate");
  });
});
