import type { Metadata } from "next";
import { Inter, JetBrains_Mono } from "next/font/google";
import { ThemeProvider } from "@/providers/theme-provider";
import { QueryProvider } from "@/providers/query-provider";
import { WalletProvider } from "@/providers/wallet-provider";
import { SubscriptionsProvider } from "@/providers/subscriptions-provider";
import { Toaster } from "sonner";
import "./globals.css";

const sans = Inter({
  subsets: ["latin"],
  variable: "--font-geist-sans",
  display: "swap",
});

const mono = JetBrains_Mono({
  subsets: ["latin"],
  variable: "--font-geist-mono",
  display: "swap",
});

export const metadata: Metadata = {
  metadataBase: new URL("https://wallet.norn.network"),
  title: {
    default: "Norn Wallet — Self-Custodial Web Wallet",
    template: "%s | Norn Wallet",
  },
  description:
    "Self-custodial web wallet for Norn Protocol. Secure passkey authentication, send/receive NORN, manage tokens, and interact with smart contracts.",
  icons: { icon: "/icon.svg" },
  keywords: [
    "norn",
    "norn wallet",
    "web wallet",
    "norn protocol",
    "passkey wallet",
    "self-custodial",
  ],
  openGraph: {
    title: "Norn Wallet",
    description:
      "Self-custodial web wallet for Norn Protocol with passkey authentication.",
    siteName: "Norn Wallet",
    url: "https://wallet.norn.network",
    type: "website",
    locale: "en_US",
  },
  twitter: {
    card: "summary",
    title: "Norn Wallet",
    description:
      "Self-custodial web wallet for Norn Protocol. Passkey-secured.",
  },
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html
      lang="en"
      className={`${sans.variable} ${mono.variable}`}
      suppressHydrationWarning
    >
      <body className="min-h-screen bg-background font-sans antialiased">
        <ThemeProvider
          attribute="class"
          defaultTheme="dark"
          forcedTheme="dark"
          disableTransitionOnChange
        >
          <QueryProvider>
            <WalletProvider>
              <SubscriptionsProvider>
                {children}
                <Toaster
                  theme="dark"
                  position="top-center"
                  toastOptions={{
                    className: "!bg-popover !border-border !text-foreground",
                  }}
                  richColors
                />
              </SubscriptionsProvider>
            </WalletProvider>
          </QueryProvider>
        </ThemeProvider>
      </body>
    </html>
  );
}
