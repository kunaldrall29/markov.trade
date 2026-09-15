import { createFileRoute } from "@tanstack/react-router";
import { mandateView } from "@/lib/server/chain.server";
import { apiError, json, withErrors } from "@/lib/server/respond";

const ADDRESS = /^[1-9A-HJ-NP-Za-km-z]{32,44}$/;

export const Route = createFileRoute("/api/v1/mandates/$address")({
  server: {
    handlers: {
      GET: async ({ params }) =>
        withErrors(async () => {
          if (!ADDRESS.test(params.address)) return apiError("BAD_REQUEST", "address must be base58", 400);
          const m = await mandateView(params.address);
          if (!m) return apiError("NOT_FOUND", `no mandate at ${params.address}`, 404);
          return json({ env: "devnet", source: "chain", fetched_at: Date.now(), mandate: m }, { maxAge: 5 });
        }),
    },
  },
});
