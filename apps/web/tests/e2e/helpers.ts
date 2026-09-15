import { expect, type Page } from "@playwright/test";
import type { Address } from "@solana/kit";
import { findAssociatedTokenPda, TOKEN_PROGRAM_ADDRESS } from "@solana-program/token";
import { USDC_D_MINT } from "@markov/sdk";

export const FIXTURE_RPC = "http://127.0.0.1:8899";
export const usingFixture = !process.env.E2E_RPC_URL || process.env.E2E_RPC_URL === FIXTURE_RPC;

export async function setScenario(patch: Record<string, unknown>) {
  const res = await fetch(`${FIXTURE_RPC}/__scenario`, { method: "POST", body: JSON.stringify(patch) });
  expect(res.ok).toBeTruthy();
}

export async function resetScenario() {
  await setScenario({ state: "Active", down: false, owner: null, ownerAta: null, halt: false });
}

/** Re-own the fixture mandate (and its owner's USDC-d account) to `address`. */
export async function reownFixtureTo(address: string) {
  const [ata] = await findAssociatedTokenPda({ owner: address as Address, mint: USDC_D_MINT, tokenProgram: TOKEN_PROGRAM_ADDRESS });
  await setScenario({ owner: address, ownerAta: ata });
}

/** Connect the build-flagged test wallet through the same UI a person uses. */
export async function connectTestWallet(page: Page): Promise<string> {
  await page.getByTestId("connect-wallet").click();
  await page.getByRole("dialog").getByTestId("wallet-Markov Test Wallet").click();
  await expect(page.getByTestId("wallet-menu")).toBeVisible();
  const address = await page.evaluate(() => window.__markovTestWallet?.address ?? "");
  expect(address).toMatch(/^[1-9A-HJ-NP-Za-km-z]{32,44}$/);
  return address;
}

export async function captured(page: Page) {
  return page.evaluate(() => window.__markovTestWallet?.captured ?? []);
}

export async function noHorizontalOverflow(page: Page) {
  const { scrollWidth, clientWidth } = await page.evaluate(() => ({ scrollWidth: document.documentElement.scrollWidth, clientWidth: document.documentElement.clientWidth }));
  expect(scrollWidth, "page must not scroll horizontally").toBeLessThanOrEqual(clientWidth + 1);
}

declare global {
  interface Window {
    __markovTestWallet?: { address: string; mode: "capture" | "send"; captured: { instructions: { programAddress: string; data: number[]; accounts: string[] }[]; feePayer: string; wire: string }[] };
  }
}
