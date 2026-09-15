import type { ApiError, BookStats, Health, MandateSummary, ReceiptsResponse } from "@/lib/api-types";

async function getJson<T>(path: string): Promise<T> {
  const res = await fetch(path, { headers: { accept: "application/json" } });
  const body = (await res.json().catch(() => null)) as T | ApiError | null;
  if (!res.ok) {
    const message = body && "error" in (body as ApiError) ? (body as ApiError).error.message : `${res.status} ${res.statusText}`;
    throw new Error(message);
  }
  if (body == null) throw new Error("empty response");
  return body as T;
}

export const api = {
  health: () => getJson<Health>("/api/health"),
  bookStats: () => getJson<BookStats>("/api/v1/book/stats"),
  receipts: (opts: { mandate?: string; limit?: number; before?: string } = {}) => {
    const q = new URLSearchParams();
    if (opts.mandate) q.set("mandate", opts.mandate);
    if (opts.limit) q.set("limit", String(opts.limit));
    if (opts.before) q.set("before", opts.before);
    const s = q.toString();
    return getJson<ReceiptsResponse>(`/api/v1/receipts${s ? "?" + s : ""}`);
  },
  mandate: (address: string) => getJson<{ mandate: MandateSummary }>(`/api/v1/mandates/${address}`).then((r) => r.mandate),
};
