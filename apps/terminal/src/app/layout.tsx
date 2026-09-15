import type { Metadata, Viewport } from "next";
import { Bricolage_Grotesque, Instrument_Sans, JetBrains_Mono } from "next/font/google";
import "./globals.css";
import { WalletRoot } from "@/components/wallet-root";
import { Shell } from "@/components/shell";
import { stageLabel } from "@/lib/env";

const display = Bricolage_Grotesque({
  subsets: ["latin"],
  weight: ["700", "800"],
  variable: "--font-display",
  display: "swap",
});
const body = Instrument_Sans({
  subsets: ["latin"],
  weight: ["400", "500", "600"],
  variable: "--font-body",
  display: "swap",
});
const mono = JetBrains_Mono({
  subsets: ["latin"],
  weight: ["400", "500", "600"],
  variable: "--font-mono",
  display: "swap",
});

export const metadata: Metadata = {
  title: `Markov Terminal · ${stageLabel}`,
  description: "Programmable account for perps and tokenized stocks on Solana. Owner-signed. Real data only.",
  icons: { icon: "/brand/markov-mark.svg" },
};

export const viewport: Viewport = {
  width: "device-width",
  initialScale: 1,
  themeColor: "#ECEBE6",
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en" className={`${display.variable} ${body.variable} ${mono.variable}`}>
      <body>
        <WalletRoot>
          <Shell>{children}</Shell>
        </WalletRoot>
      </body>
    </html>
  );
}
