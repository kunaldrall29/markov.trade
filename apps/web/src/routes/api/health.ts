import { createFileRoute } from "@tanstack/react-router";
import { health } from "@/lib/server/chain.server";
import { json } from "@/lib/server/respond";

export const Route = createFileRoute("/api/health")({
  server: {
    handlers: {
      GET: async () => {
        const h = await health();
        return json(h, { status: h.ok ? 200 : 503 });
      },
    },
  },
});
