import type { ApiError } from "@/lib/api-types";

const CORS = { "access-control-allow-origin": "*", "access-control-allow-methods": "GET, OPTIONS" };

export function json(data: unknown, init: { status?: number; maxAge?: number } = {}): Response {
  return new Response(JSON.stringify(data), {
    status: init.status ?? 200,
    headers: {
      "content-type": "application/json; charset=utf-8",
      "cache-control": init.maxAge != null ? `public, max-age=${init.maxAge}, stale-while-revalidate=${init.maxAge}` : "no-store",
      ...CORS,
    },
  });
}

export function apiError(code: string, message: string, status = 502): Response {
  const body: ApiError = { error: { code, message } };
  return json(body, { status });
}

export function withErrors(fn: () => Promise<Response>): Promise<Response> {
  return fn().catch((e: unknown) => apiError("CHAIN_READ_FAILED", e instanceof Error ? e.message : String(e)));
}

export function intParam(url: URL, name: string, fallback: number, max: number): number {
  const raw = url.searchParams.get(name);
  if (!raw) return fallback;
  const n = Number.parseInt(raw, 10);
  if (!Number.isFinite(n) || n < 1) return fallback;
  return Math.min(n, max);
}
