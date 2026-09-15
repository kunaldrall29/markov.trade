import { createFileRoute } from "@tanstack/react-router";
import { bookStats } from "@/lib/server/chain.server";
import { json, withErrors } from "@/lib/server/respond";

export const Route = createFileRoute("/api/v1/book/stats")({
  server: {
    handlers: {
      GET: async () => withErrors(async () => json(await bookStats(), { maxAge: 5 })),
    },
  },
});
