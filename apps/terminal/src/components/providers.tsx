"use client";

import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { useState } from "react";
import { WalletRoot } from "@/components/wallet-root";
import { Shell } from "@/components/shell";

export function Providers({ children }: { children: React.ReactNode }) {
  const [client] = useState(() => new QueryClient({ defaultOptions: { queries: { refetchOnWindowFocus: false } } }));
  return (
    <QueryClientProvider client={client}>
      <WalletRoot>
        <Shell>{children}</Shell>
      </WalletRoot>
    </QueryClientProvider>
  );
}
