/**
 * Registers the Playwright test wallet when — and only when — the build was
 * made with `VITE_TEST_WALLET=1`. In every other build this component is a
 * constant `null` and the test wallet module is never bundled.
 */
import { useEffect } from "react";
import { testWalletEnabled } from "@/lib/config";

export function TestWalletBoot() {
  useEffect(() => {
    if (!testWalletEnabled) return;
    void import("./wallet").then((m) => m.registerTestWallet());
  }, []);
  return null;
}
