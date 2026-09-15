import { describe, expect, it } from "vitest";
import { formatUnits, parseUnits, bytesToId, idToBytes, formatPriceE6 } from "../src/format";
import { policyFromSpec, wideningFields, ACTION_BITS } from "../src/verbs";
import { GATE_B_POLICY, USDC_D_MINT, DEMO_PERPS_PROGRAM_ID } from "../src/facts";

describe("format", () => {
  it("formats base units without floats", () => {
    expect(formatUnits(12_345_678n, 6)).toBe("12.345678");
    expect(formatUnits(100_000_000n, 6)).toBe("100.00");
    expect(formatUnits(-25_000n, 6)).toBe("-0.025");
    expect(formatUnits(1_234_567_000_000n, 6)).toBe("1,234,567.00");
    expect(formatPriceE6(104_724_049n)).toBe("104.724");
  });
  it("parses typed amounts and refuses junk", () => {
    expect(parseUnits("12.5", 6)).toBe(12_500_000n);
    expect(parseUnits("0.000001", 6)).toBe(1n);
    expect(parseUnits("1.2345678", 6)).toBeNull();
    expect(parseUnits("1e3", 6)).toBeNull();
    expect(parseUnits("", 6)).toBeNull();
  });
  it("round-trips fixed-width ids", () => {
    expect(bytesToId(idToBytes("SOL-PERP"))).toBe("SOL-PERP");
    expect(idToBytes("BOOK_ONE").length).toBe(16);
  });
  it("mirrors Policy::assert_tightens", () => {
    const base = policyFromSpec({
      venues: [DEMO_PERPS_PROGRAM_ID], tokens: [USDC_D_MINT], allowedActions: ACTION_BITS.all,
      perTxCap: GATE_B_POLICY.perTxCap, dailyCap: GATE_B_POLICY.dailyCap, spendPerCall: GATE_B_POLICY.spendPerCall,
      spendDaily: GATE_B_POLICY.spendDaily, maxSlippageBps: 50, maxMarkAgeSecs: 150n, expiryTs: 1_760_000_000n,
    });
    const prev = { ...base, venues: base.venues, tokens: base.tokens } as never;
    expect(wideningFields(prev, base)).toEqual([]);
    expect(wideningFields(prev, { ...base, perTxCap: BigInt(base.perTxCap) + 1n })).toEqual(["per-trade cap"]);
    expect(wideningFields(prev, { ...base, allowedActions: ACTION_BITS.reduce })).toEqual([]);
    expect(wideningFields(prev, { ...base, expiryTs: 1_760_000_001n, maxSlippageBps: 51 })).toEqual(["max slippage", "expiry"]);
  });
});
