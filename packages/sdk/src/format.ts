/**
 * Integer-only formatting. Money on this surface is `{raw, decimals, mint}`
 * (docs/12 §3); there is no float on the path from chain to pixel.
 */
export type Money = { raw: bigint; decimals: number };

/** `12345678n` at 6 decimals → `"12.345678"`; `trim` drops trailing zeros to `min` places. */
export function formatUnits(raw: bigint, decimals: number, opts: { min?: number; max?: number } = {}): string {
  const min = opts.min ?? 2;
  const max = opts.max ?? decimals;
  const negative = raw < 0n;
  const abs = negative ? -raw : raw;
  const base = 10n ** BigInt(decimals);
  const whole = abs / base;
  let frac = (abs % base).toString().padStart(decimals, "0");
  frac = frac.slice(0, Math.max(min, Math.min(max, decimals)));
  while (frac.length > min && frac.endsWith("0")) frac = frac.slice(0, -1);
  const wholeStr = whole.toLocaleString("en-US");
  return `${negative ? "-" : ""}${wholeStr}${frac.length ? "." + frac : ""}`;
}

/** Parse a user-typed decimal string into base units; refuses anything that is not a plain number. */
export function parseUnits(text: string, decimals: number): bigint | null {
  const m = /^\s*(\d+)(?:\.(\d*))?\s*$/.exec(text);
  if (!m) return null;
  const whole = m[1] ?? "0";
  const frac = (m[2] ?? "").slice(0, decimals).padEnd(decimals, "0");
  if ((m[2] ?? "").length > decimals) return null;
  return BigInt(whole) * 10n ** BigInt(decimals) + BigInt(frac || "0");
}

/** A price scaled 1e6 per unit (the program's `*_e6` fields). */
export function formatPriceE6(price: bigint): string {
  return formatUnits(price, 6, { min: 2, max: 4 });
}

/** Fixed-width on-chain ids (`[u8; 16]`) are NUL-padded ASCII. */
export function bytesToId(bytes: Uint8Array | ArrayLike<number>): string {
  let out = "";
  for (let i = 0; i < bytes.length; i += 1) {
    const b = bytes[i] ?? 0;
    if (b === 0) break;
    out += String.fromCharCode(b);
  }
  return out;
}

export function idToBytes(id: string, length = 16): Uint8Array {
  const out = new Uint8Array(length);
  for (let i = 0; i < id.length && i < length; i += 1) out[i] = id.charCodeAt(i) & 0xff;
  return out;
}

export function hexToBytes(hex: string): Uint8Array {
  const clean = hex.startsWith("0x") ? hex.slice(2) : hex;
  const out = new Uint8Array(clean.length / 2);
  for (let i = 0; i < out.length; i += 1) out[i] = parseInt(clean.slice(i * 2, i * 2 + 2), 16);
  return out;
}

export function bytesToHex(bytes: ArrayLike<number>): string {
  let s = "";
  for (let i = 0; i < bytes.length; i += 1) s += (bytes[i] ?? 0).toString(16).padStart(2, "0");
  return s;
}

/** `12ab…89xy` for addresses and signatures. */
export function short(s: string, head = 4, tail = 4): string {
  return s.length <= head + tail + 1 ? s : `${s.slice(0, head)}…${s.slice(-tail)}`;
}

export function slotAgeSeconds(nowSlot: bigint, thenSlot: bigint, slotMs: number): number {
  const d = nowSlot > thenSlot ? nowSlot - thenSlot : 0n;
  return Math.round((Number(d) * slotMs) / 1000);
}
