import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { createRootRoute, HeadContent, Outlet, Scripts } from "@tanstack/react-router";
import { useState } from "react";
import { WalletProvider } from "@/lib/wallet/provider";
import { SignerProvider } from "@/lib/wallet/signer";
import { TestWalletBoot } from "@/test-wallet/boot";
import appCss from "../styles.css?url";

const APP_NAME = "Markov";
const THEME_BOOT = `(function(){try{var t=localStorage.getItem("markov-theme");var d=t==="dark"||(t!=="light"&&window.matchMedia("(prefers-color-scheme: dark)").matches);if(d)document.documentElement.classList.add("dark")}catch(e){}})();`;

export const Route = createRootRoute({
  head: () => ({
    meta: [
      { charSet: "utf-8" },
      { name: "viewport", content: "width=device-width, initial-scale=1, viewport-fit=cover" },
      { title: APP_NAME },
      {
        name: "description",
        content:
          "Markov is a financial control plane on Solana. Your rules live in an on-chain account; every allow and every refusal is a receipt. Test stage on devnet.",
      },
      { name: "theme-color", content: "#111110" },
      { property: "og:title", content: "Markov" },
      { property: "og:description", content: "Capital may only do what the owner allowed. Test stage, Solana devnet." },
      { property: "og:image", content: "/og.jpg" },
      { name: "twitter:card", content: "summary_large_image" },
    ],
    links: [
      { rel: "icon", type: "image/svg+xml", href: "/favicon.svg" },
      { rel: "manifest", href: "/manifest.webmanifest" },
      { rel: "apple-touch-icon", href: "/icon-180.png" },
      { rel: "stylesheet", href: appCss },
      { rel: "preconnect", href: "https://fonts.googleapis.com" },
      { rel: "preconnect", href: "https://fonts.gstatic.com", crossOrigin: "anonymous" },
      {
        rel: "stylesheet",
        href: "https://fonts.googleapis.com/css2?family=JetBrains+Mono:wght@400;500&family=Outfit:wght@400;500;600;700&display=swap",
      },
    ],
  }),
  component: RootDocument,
});

function RootDocument() {
  const [queryClient] = useState(() => new QueryClient({ defaultOptions: { queries: { refetchOnWindowFocus: true } } }));
  return (
    <html lang="en" suppressHydrationWarning>
      <head>
        <script dangerouslySetInnerHTML={{ __html: THEME_BOOT }} />
        <HeadContent />
      </head>
      <body className="antialiased">
        <QueryClientProvider client={queryClient}>
          <TestWalletBoot />
          <WalletProvider>
            <SignerProvider>
              <Outlet />
            </SignerProvider>
          </WalletProvider>
        </QueryClientProvider>
        <Scripts />
      </body>
    </html>
  );
}
