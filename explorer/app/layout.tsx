import type { Metadata } from "next";
import { Inter, JetBrains_Mono } from "next/font/google";
import { ThemeProvider } from "@/providers/theme-provider";
import { QueryProvider } from "@/providers/query-provider";
import { SubscriptionsProvider } from "@/providers/subscriptions-provider";
import { Header } from "@/components/layout/header";
import { Footer } from "@/components/layout/footer";
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
  metadataBase: new URL("https://explorer.norn.network"),
  title: {
    default: "Norn Explorer — Block Explorer for Norn Protocol",
    template: "%s | Norn Explorer",
  },
  description:
    "Explore blocks, transactions, addresses, tokens, and smart contracts on the Norn network. Real-time data with WebSocket updates.",
  icons: { icon: "/icon.svg" },
  keywords: [
    "norn",
    "norn explorer",
    "block explorer",
    "norn protocol",
    "blockchain explorer",
    "norn transactions",
    "norn blocks",
    "norn tokens",
  ],
  openGraph: {
    title: "Norn Explorer",
    description:
      "Block explorer for the Norn Protocol. Browse blocks, transactions, addresses, tokens, and smart contracts in real time.",
    siteName: "Norn Explorer",
    url: "https://explorer.norn.network",
    type: "website",
    locale: "en_US",
  },
  twitter: {
    card: "summary",
    title: "Norn Explorer",
    description:
      "Block explorer for the Norn Protocol. Real-time blocks, transactions, and network stats.",
  },
  robots: {
    index: true,
    follow: true,
    googleBot: {
      index: true,
      follow: true,
    },
  },
  alternates: {
    canonical: "https://explorer.norn.network",
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
            <SubscriptionsProvider>
              <div className="relative flex min-h-screen flex-col">
                <Header />
                <main className="flex-1">{children}</main>
                <Footer />
              </div>
              <Toaster
                theme="dark"
                position="bottom-right"
                toastOptions={{
                  className: "!bg-popover !border-border !text-foreground",
                }}
                richColors
              />
            </SubscriptionsProvider>
          </QueryProvider>
        </ThemeProvider>
      </body>
    </html>
  );
}
