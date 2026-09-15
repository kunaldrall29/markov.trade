/** JSON-RPC helpers. No SDK. Fail closed. */

export async function getAccountExecutable(
  rpc: string,
  address: string,
): Promise<{ deployed: boolean; slot: number | null }> {
  try {
    const res = await fetch(rpc, {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({
        jsonrpc: "2.0",
        id: 1,
        method: "getAccountInfo",
        params: [address, { encoding: "base64" }],
      }),
      signal: AbortSignal.timeout(8_000),
    });
    const j = (await res.json()) as {
      result?: { context?: { slot: number }; value: { executable?: boolean } | null };
    };
    return {
      deployed: Boolean(j.result?.value?.executable),
      slot: j.result?.context?.slot ?? null,
    };
  } catch {
    return { deployed: false, slot: null };
  }
}
