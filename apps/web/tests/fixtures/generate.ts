/**
 * Builds `tests/fixtures/rpc.json`: the account and transaction set the
 * fixture RPC serves to Playwright. Bytes are produced by the SDK's codecs,
 * which `packages/sdk` tests against bytes the program crate itself wrote,
 * and are placed at the real devnet addresses FACTS records (the Gate B
 * mandate, its vault, its venue position). Values are labelled fixtures;
 * they are not a recording of a slot.
 */
import { getAddressEncoder, getBase58Decoder, getBase64Decoder, type Address, type ReadonlyUint8Array } from "@solana/kit";
import { getTokenEncoder, TOKEN_PROGRAM_ADDRESS, findAssociatedTokenPda } from "@solana-program/token";
import {
  BOOK_ONE_EMERGENCY,
  BOOK_ONE_OPERATOR,
  DEMO_PERPS_PROGRAM_ID,
  DEVNET_ADMIN,
  EVENT_IX_TAG_LE,
  GATE_B_MANDATE,
  GATE_B_MANDATE_OWNER,
  GATE_B_MANDATE_VAULT,
  GATE_B_POLICY,
  GATE_B_VENUE_POSITION,
  MANDATE_PROGRAM_ID,
  PYTH_RECEIVER_PROGRAM_ID,
  PYTH_SOL_USD_PRICE_UPDATE,
  SOL_USD_FEED_ID_HEX,
  USDC_D_MINT,
  deriveRegistry,
  deriveVenueMark,
  deriveVenueMarket,
  hexToBytes,
  idToBytes,
  policyFromSpec,
  ACTION_BITS,
} from "@markov/sdk";
import {
  MANDATE_DISCRIMINATOR,
  getMandateEncoder,
  getRegistryEncoder,
  getActionReceiptEventEncoder,
  getOwnerActionEventEncoder,
  getRefusalReceiptEventEncoder,
  getPriceUpdateV2Encoder,
  BlockReason,
  MandateState,
  OwnerActionKind,
} from "@markov/sdk/generated/mandate";
import { getMarkAccountEncoder, getMarketEncoder, getPositionEncoder, MarkSourceKind, Side } from "@markov/sdk/generated/demo-perps";

const b64 = getBase64Decoder();
const b58 = getBase58Decoder();
const addr = getAddressEncoder();

function concat(...parts: (Uint8Array | ArrayLike<number>)[]): Uint8Array {
  const len = parts.reduce((n, p) => n + p.length, 0);
  const out = new Uint8Array(len);
  let o = 0;
  for (const p of parts) {
    out.set(p as Uint8Array, o);
    o += p.length;
  }
  return out;
}

export type FixtureAccount = { owner: string; data: string; lamports?: number };
export type FixtureTx = { signature: string; slot: number; blockTime: number; err: null | object; programIds: string[]; innerData: string[]; accountKeys: string[] };
export type Fixtures = {
  note: string;
  slot: number;
  accounts: Record<string, FixtureAccount>;
  transactions: FixtureTx[];
  offsets: { mandateState: number; mandateOwner: number; pythPublishTime: number; pythPostedSlot: number };
  addresses: Record<string, string>;
};

const NOW = Math.floor(Date.now() / 1000);
const SLOT = 493_000_000;

function fakeSignature(n: number): string {
  // 64 bytes, deterministic, base58. A fixture signature, never a real one.
  const bytes = new Uint8Array(64);
  for (let i = 0; i < 64; i += 1) bytes[i] = (n * 131 + i * 17) & 0xff;
  return b58.decode(bytes);
}

export async function buildFixtures(): Promise<Fixtures> {
  const policy = policyFromSpec({
    venues: [DEMO_PERPS_PROGRAM_ID],
    tokens: [USDC_D_MINT],
    allowedActions: ACTION_BITS.all,
    perTxCap: GATE_B_POLICY.perTxCap,
    dailyCap: GATE_B_POLICY.dailyCap,
    spendPerCall: GATE_B_POLICY.spendPerCall,
    spendDaily: GATE_B_POLICY.spendDaily,
    maxSlippageBps: GATE_B_POLICY.maxSlippageBps,
    maxMarkAgeSecs: GATE_B_POLICY.maxMarkAgeSecs,
    expiryTs: BigInt(NOW + 25 * 86_400),
  });
  const mandateBody = getMandateEncoder().encode({
    owner: GATE_B_MANDATE_OWNER,
    operator: BOOK_ONE_OPERATOR,
    emergency: BOOK_ONE_EMERGENCY,
    strategyId: idToBytes("BOOK_ONE"),
    state: MandateState.Active,
    policy,
    vault: GATE_B_MANDATE_VAULT,
    mint: USDC_D_MINT,
    markAccount: PYTH_SOL_USD_PRICE_UPDATE,
    feedId: hexToBytes(SOL_USD_FEED_ID_HEX),
    dayEpoch: BigInt(Math.floor(NOW / 86_400)),
    dayNotionalUsed: 25_000_000n,
    daySpendUsed: 1_000_000n,
    actionSeq: 3n,
    recentIntents: Array.from({ length: 8 }, () => new Uint8Array(32)),
    recentIntentsLen: 0,
    recentIntentsNext: 0,
    createdAt: BigInt(NOW - 3 * 86_400),
    nonce: 10n,
    bump: 254,
    vaultBump: 253,
    reserve: new Uint8Array(128),
  });
  // Generated account encoders emit the discriminator themselves.
  const mandate = mandateBody as Uint8Array;

  const registry = getRegistryEncoder().encode({
      admin: DEVNET_ADMIN,
      globalHalt: false,
      adapters: [DEMO_PERPS_PROGRAM_ID, ...Array.from({ length: 7 }, () => "11111111111111111111111111111111" as Address)],
      adaptersLen: 1,
      bump: 255,
    }) as Uint8Array;

  const vault = getTokenEncoder().encode({
    mint: USDC_D_MINT,
    owner: GATE_B_MANDATE,
    amount: 100_000_000n,
    delegate: null,
    state: 1,
    isNative: null,
    delegatedAmount: 0n,
    closeAuthority: null,
  });

  const marketAddr = await deriveVenueMarket("SOL-PERP");
  const markAddr = await deriveVenueMark("SOL-PERP");
  const market = getMarketEncoder().encode({ authority: DEVNET_ADMIN, marketId: idToBytes("SOL-PERP"), baseDecimals: 9, mark: markAddr, feeBps: 10, maxAgeSlots: 900n, positionCap: 1_000_000_000n, paused: false, bump: 250 }) as Uint8Array;
  const mark = getMarkAccountEncoder().encode({ marketId: idToBytes("SOL-PERP"), price: 10_461_943_000n, expo: -8, publishTime: BigInt(NOW - 20), slot: BigInt(SLOT - 100), source: MarkSourceKind.Pyth, poster: DEVNET_ADMIN, bump: 249 }) as Uint8Array;
  const position = getPositionEncoder().encode({ mandate: GATE_B_MANDATE, marketId: idToBytes("SOL-PERP"), side: Side.Long, notional: 15_000_000n, entryPrice: 104_724_049n, fundingAccrued: -120n, updatedSlot: BigInt(SLOT - 500), bump: 248 }) as Uint8Array;

  const PRICE_UPDATE_V2_DISCRIMINATOR = new Uint8Array([34, 241, 35, 99, 157, 126, 244, 205]);
  const pythBody = getPriceUpdateV2Encoder().encode({
    writeAuthority: DEVNET_ADMIN,
    verificationLevel: { __kind: "Full" },
    priceMessage: { feedId: hexToBytes(SOL_USD_FEED_ID_HEX), price: 10_461_943_000n, conf: 1_500_000n, exponent: -8, publishTime: BigInt(NOW - 15), prevPublishTime: BigInt(NOW - 45), emaPrice: 10_460_000_000n, emaConf: 1_400_000n },
    postedSlot: BigInt(SLOT - 60),
  });
  const pyth = concat(PRICE_UPDATE_V2_DISCRIMINATOR, pythBody);
  // 8 disc + 32 write_authority + 1 (Full) + 32 feed_id + 8 price + 8 conf + 4 exponent = 93
  const pythPublishTime = 8 + 32 + 1 + 32 + 8 + 8 + 4;
  const pythPostedSlot = pyth.length - 8;

  const [ownerAta] = await findAssociatedTokenPda({ owner: GATE_B_MANDATE_OWNER, mint: USDC_D_MINT, tokenProgram: TOKEN_PROGRAM_ADDRESS });
  const ownerToken = getTokenEncoder().encode({ mint: USDC_D_MINT, owner: GATE_B_MANDATE_OWNER, amount: 250_000_000n, delegate: null, state: 1, isNative: null, delegatedAmount: 0n, closeAuthority: null });

  const strategyId = idToBytes("BOOK_ONE");
  const marketId = idToBytes("SOL-PERP");
  const base = (seq: bigint) => ({ seq, mandate: GATE_B_MANDATE, operator: BOOK_ONE_OPERATOR, strategyId, venue: DEMO_PERPS_PROGRAM_ID });
  const ev = (bytes: Uint8Array | ReadonlyUint8Array) => concat(EVENT_IX_TAG_LE, bytes);
  const events: { data: Uint8Array; secondsAgo: number; keys: string[] }[] = [
    { secondsAgo: 60, keys: [BOOK_ONE_OPERATOR, GATE_B_MANDATE], data: ev(getRefusalReceiptEventEncoder().encode({ ...base(5n), intentId: new Uint8Array(32).fill(0xbb), action: 2, notional: 51_000_000n, reason: BlockReason.OverTxCap, gateIndex: 8, forced: true, ts: BigInt(NOW - 60), slot: BigInt(SLOT - 360) })) },
    { secondsAgo: 600, keys: [BOOK_ONE_OPERATOR, GATE_B_MANDATE], data: ev(getActionReceiptEventEncoder().encode({ ...base(4n), intentId: new Uint8Array(32).fill(0xaa), owner: GATE_B_MANDATE_OWNER, market: marketId, action: 3, side: 0, notional: 25_000_000n, fillPrice: 104_724_049n, fee: 25_000n, markPrice: 104_619_430n, markPublishTime: BigInt(NOW - 610), spend: 1_000_000n, forced: false, ts: BigInt(NOW - 600), slot: BigInt(SLOT - 3600), netDeltaUsdE6: 0n, grossUsdE6: 0n })) },
    { secondsAgo: 1200, keys: [BOOK_ONE_OPERATOR, GATE_B_MANDATE], data: ev(getRefusalReceiptEventEncoder().encode({ ...base(3n), intentId: new Uint8Array(32).fill(0xcc), action: 1, notional: 10_000_000n, reason: BlockReason.SlippageExceeded, gateIndex: 11, forced: true, ts: BigInt(NOW - 1200), slot: BigInt(SLOT - 7200) })) },
    { secondsAgo: 3600, keys: [GATE_B_MANDATE_OWNER, GATE_B_MANDATE], data: ev(getOwnerActionEventEncoder().encode({ kind: OwnerActionKind.Fund, mandate: GATE_B_MANDATE, owner: GATE_B_MANDATE_OWNER, actor: GATE_B_MANDATE_OWNER, strategyId, mint: USDC_D_MINT, amount: 100_000_000n, ts: BigInt(NOW - 3600), slot: BigInt(SLOT - 21_600) })) },
    { secondsAgo: 4000, keys: [GATE_B_MANDATE_OWNER, GATE_B_MANDATE], data: ev(getOwnerActionEventEncoder().encode({ kind: OwnerActionKind.Create, mandate: GATE_B_MANDATE, owner: GATE_B_MANDATE_OWNER, actor: GATE_B_MANDATE_OWNER, strategyId, mint: USDC_D_MINT, amount: 0n, ts: BigInt(NOW - 4000), slot: BigInt(SLOT - 24_000) })) },
  ];
  const transactions: FixtureTx[] = events.map((e, i) => ({
    signature: fakeSignature(i + 1),
    slot: SLOT - Math.round(e.secondsAgo * 6),
    blockTime: NOW - e.secondsAgo,
    err: null,
    programIds: [MANDATE_PROGRAM_ID],
    innerData: [b58.decode(e.data)],
    accountKeys: [...e.keys, MANDATE_PROGRAM_ID],
  }));

  const registryAddr = await deriveRegistry();
  const enc = (bytes: Uint8Array | ReadonlyUint8Array) => b64.decode(bytes);
  return {
    note: "Fixture set for Playwright. Layouts from the SDK codecs (tested against program-crate bytes); values synthetic; addresses real (FACTS).",
    slot: SLOT,
    offsets: { mandateState: 8 + 32 * 3 + 16, mandateOwner: 8, pythPublishTime, pythPostedSlot },
    addresses: { mandate: GATE_B_MANDATE, vault: GATE_B_MANDATE_VAULT, owner: GATE_B_MANDATE_OWNER, ownerAta, registry: registryAddr, market: marketAddr, mark: markAddr, position: GATE_B_VENUE_POSITION, pyth: PYTH_SOL_USD_PRICE_UPDATE },
    accounts: {
      [GATE_B_MANDATE]: { owner: MANDATE_PROGRAM_ID, data: enc(mandate) },
      [registryAddr]: { owner: MANDATE_PROGRAM_ID, data: enc(registry) },
      [GATE_B_MANDATE_VAULT]: { owner: TOKEN_PROGRAM_ADDRESS, data: enc(vault as Uint8Array) },
      [ownerAta]: { owner: TOKEN_PROGRAM_ADDRESS, data: enc(ownerToken as Uint8Array) },
      [marketAddr]: { owner: DEMO_PERPS_PROGRAM_ID, data: enc(market) },
      [markAddr]: { owner: DEMO_PERPS_PROGRAM_ID, data: enc(mark) },
      [GATE_B_VENUE_POSITION]: { owner: DEMO_PERPS_PROGRAM_ID, data: enc(position) },
      [PYTH_SOL_USD_PRICE_UPDATE]: { owner: PYTH_RECEIVER_PROGRAM_ID, data: enc(pyth) },
    },
    transactions,
  };
}

// The mandate state byte sits after discriminator, owner, operator, emergency and strategy_id.
void addr;
void MANDATE_DISCRIMINATOR;
