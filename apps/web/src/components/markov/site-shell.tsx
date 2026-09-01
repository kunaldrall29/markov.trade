import type { ReactNode } from "react";
import { Footer } from "./footer";
import { Grain } from "./grain";
import { Header } from "./header";

export function SiteShell({ children }: { children: ReactNode }) {
  return (
    <div className="min-h-svh bg-bg text-fg">
      <a
        href="#main"
        className="sr-only focus:not-sr-only focus:absolute focus:left-4 focus:top-4 focus:z-50 focus:rounded-sm focus:bg-accent focus:px-3 focus:py-2 focus:text-accent-fg"
      >
        Skip to content
      </a>
      <Grain />
      <Header />
      <main id="main">{children}</main>
      <Footer />
    </div>
  );
}
