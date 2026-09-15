import { writeFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { getBase64Encoder } from "@solana/kit";
import { getMandateDecoder, MandateState } from "@markov/sdk/generated/mandate";
import { buildFixtures } from "./generate";

describe("rpc fixtures", () => {
  it("builds a decodable fixture set and writes tests/fixtures/rpc.json", async () => {
    const fx = await buildFixtures();
    const m = fx.accounts[fx.addresses.mandate!]!;
    const bytes = getBase64Encoder().encode(m.data) as Uint8Array;
    const decoded = getMandateDecoder().decode(bytes);
    expect(decoded.owner).toBe(fx.addresses.owner);
    expect(decoded.state).toBe(MandateState.Active);
    expect(bytes[fx.offsets.mandateState]).toBe(MandateState.Active);
    // Flip the state byte the way the fixture server does and check it decodes as Paused.
    bytes[fx.offsets.mandateState] = MandateState.Paused;
    expect(getMandateDecoder().decode(bytes).state).toBe(MandateState.Paused);
    expect(fx.transactions.length).toBe(5);
    writeFileSync(join(__dirname, "rpc.json"), JSON.stringify(fx, null, 2));
  });
});
