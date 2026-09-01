import { createFileRoute } from "@tanstack/react-router";
import { LandingPage } from "@/components/landing/page";
import { SiteShell } from "@/components/markov/site-shell";

export const Route = createFileRoute("/")({ component: Home });

function Home() {
  return (
    <SiteShell>
      <LandingPage />
    </SiteShell>
  );
}
