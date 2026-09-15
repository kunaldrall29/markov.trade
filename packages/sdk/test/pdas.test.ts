/**
 * Seeds against addresses FACTS recorded from devnet: the Gate B mandate
 * (`GATE_B_MANDATE`, owner `5RPxDN9h…`, nonce 10), its vault, and the venue
 * position `gate_b_setup.rs` created. A wrong seed cannot pass this.
 */
import { describe, expect, it } from "vitest";
import { BOOK_ONE_STRATEGY_ID, GATE_B_MANDATE, GATE_B_MANDATE_NONCE, GATE_B_MANDATE_OWNER, GATE_B_MANDATE_VAULT, GATE_B_VENUE_POSITION, SOL_PERP_MARKET_ID } from "../src/facts";
import { deriveEventAuthority, deriveMandate, deriveRegistry, deriveVault, deriveVenuePosition } from "../src/pdas";

describe("pdas", () => {
  it("derives the Gate B mandate from owner, BOOK_ONE and nonce 10", async () => {
    const { address } = await deriveMandate(GATE_B_MANDATE_OWNER, BOOK_ONE_STRATEGY_ID, GATE_B_MANDATE_NONCE);
    expect(address).toBe(GATE_B_MANDATE);
  });
  it("derives the Gate B vault", async () => {
    expect((await deriveVault(GATE_B_MANDATE)).address).toBe(GATE_B_MANDATE_VAULT);
  });
  it("derives the venue position for SOL-PERP", async () => {
    expect(await deriveVenuePosition(GATE_B_MANDATE, SOL_PERP_MARKET_ID)).toBe(GATE_B_VENUE_POSITION);
  });
  it("derives stable registry and event authority addresses", async () => {
    expect(await deriveRegistry()).toMatch(/^[1-9A-HJ-NP-Za-km-z]{32,44}$/);
    expect(await deriveEventAuthority()).toMatch(/^[1-9A-HJ-NP-Za-km-z]{32,44}$/);
  });
});
