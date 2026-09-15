import { defineConfig, devices } from "@playwright/test";

/**
 * Two ways to run:
 *  - default: a fixture RPC (tests/fixture-rpc.mjs) and the dev server with the
 *    build-flagged test wallet in capture mode. No network needed.
 *  - E2E_RPC_URL=https://…: the same specs against a real devnet endpoint;
 *    scenario-switching tests skip themselves because they cannot flip real
 *    chain state.
 */
const fixtureRpc = "http://127.0.0.1:8899";
const rpcUrl = process.env.E2E_RPC_URL || fixtureRpc;
const usingFixture = rpcUrl === fixtureRpc;
const port = 8080;

export default defineConfig({
  testDir: "./tests/e2e",
  timeout: 60_000,
  expect: { timeout: 15_000 },
  fullyParallel: false,
  workers: 1,
  retries: 0,
  reporter: [["list"], ["html", { open: "never" }]],
  use: {
    baseURL: `http://127.0.0.1:${port}`,
    trace: "retain-on-failure",
    launchOptions: process.env.PW_CHROMIUM ? { executablePath: process.env.PW_CHROMIUM } : {},
  },
  projects: [
    { name: "desktop", use: { ...devices["Desktop Chrome"] } },
    { name: "mobile", use: { ...devices["Pixel 7"], hasTouch: true, isMobile: true } },
  ],
  webServer: [
    ...(usingFixture ? [{ command: "node tests/fixture-rpc.mjs", url: `${fixtureRpc}/__scenario`, reuseExistingServer: true, timeout: 20_000 }] : []),
    {
      command: `VITE_RPC_URL=${rpcUrl} VITE_RPC_FALLBACK=${rpcUrl} RPC_URL=${rpcUrl} RPC_HTTP_FALLBACK=${rpcUrl} VITE_TEST_WALLET=1 VITE_TEST_WALLET_MODE=${process.env.E2E_WALLET_MODE || "capture"} npx vite dev --host 127.0.0.1 --port ${port}`,
      url: `http://127.0.0.1:${port}/api/health`,
      reuseExistingServer: false,
      timeout: 120_000,
      stdout: "ignore",
    },
  ],
});
