import { createFileRoute } from "@tanstack/react-router";
import { receiptsPage } from "@/lib/server/chain.server";
import { apiError, intParam, json, withErrors } from "@/lib/server/respond";

const ADDRESS = /^[1-9A-HJ-NP-Za-km-z]{32,44}$/;
const SIGNATURE = /^[1-9A-HJ-NP-Za-km-z]{64,90}$/;

export const Route = createFileRoute("/api/v1/receipts")({
  server: {
    handlers: {
      GET: async ({ request }) =>
        withErrors(async () => {
          const url = new URL(request.url);
          const limit = intParam(url, "limit", 50, 100);
          const address = url.searchParams.get("mandate") ?? undefined;
          const before = url.searchParams.get("before") ?? undefined;
          if (address && !ADDRESS.test(address)) return apiError("BAD_REQUEST", "mandate must be a base58 address", 400);
          if (before && !SIGNATURE.test(before)) return apiError("BAD_REQUEST", "before must be a base58 signature", 400);
          return json(await receiptsPage({ address, limit, before }), { maxAge: 5 });
        }),
    },
  },
});
