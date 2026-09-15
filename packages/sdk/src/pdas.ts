/**
 * PDA derivation, mirroring the seeds in `programs/markov-mandate/src` and
 * `programs/demo-perps/src/lib.rs`. `test/pdas.test.ts` checks these against
 * the addresses FACTS recorded from devnet, so a seed typo cannot survive.
 */
import { getAddressEncoder, getProgramDerivedAddress, getU64Encoder, getUtf8Encoder, type Address } from "@solana/kit";
import { DEMO_PERPS_PROGRAM_ID, MANDATE_PROGRAM_ID } from "./facts";
import { idToBytes } from "./format";

const utf8 = getUtf8Encoder();
const addr = getAddressEncoder();
const u64 = getU64Encoder();

export async function deriveMandate(owner: Address, strategyId: string, nonce: bigint, programId: Address = MANDATE_PROGRAM_ID) {
  const [pda, bump] = await getProgramDerivedAddress({
    programAddress: programId,
    seeds: [utf8.encode("mandate"), addr.encode(owner), idToBytes(strategyId), u64.encode(nonce)],
  });
  return { address: pda, bump };
}

export async function deriveVault(mandate: Address, programId: Address = MANDATE_PROGRAM_ID) {
  const [pda, bump] = await getProgramDerivedAddress({
    programAddress: programId,
    seeds: [utf8.encode("vault"), addr.encode(mandate)],
  });
  return { address: pda, bump };
}

export async function deriveRegistry(programId: Address = MANDATE_PROGRAM_ID) {
  const [pda] = await getProgramDerivedAddress({ programAddress: programId, seeds: [utf8.encode("registry")] });
  return pda;
}

/** Anchor's `#[event_cpi]` authority: `["__event_authority"]` of the emitting program. */
export async function deriveEventAuthority(programId: Address = MANDATE_PROGRAM_ID) {
  const [pda] = await getProgramDerivedAddress({ programAddress: programId, seeds: [utf8.encode("__event_authority")] });
  return pda;
}

export async function deriveVenueMarket(marketId: string, venue: Address = DEMO_PERPS_PROGRAM_ID) {
  const [pda] = await getProgramDerivedAddress({ programAddress: venue, seeds: [utf8.encode("market"), idToBytes(marketId)] });
  return pda;
}

export async function deriveVenueMark(marketId: string, venue: Address = DEMO_PERPS_PROGRAM_ID) {
  const [pda] = await getProgramDerivedAddress({ programAddress: venue, seeds: [utf8.encode("mark"), idToBytes(marketId)] });
  return pda;
}

export async function deriveVenuePosition(mandate: Address, marketId: string, venue: Address = DEMO_PERPS_PROGRAM_ID) {
  const [pda] = await getProgramDerivedAddress({
    programAddress: venue,
    seeds: [utf8.encode("pos"), addr.encode(mandate), idToBytes(marketId)],
  });
  return pda;
}
