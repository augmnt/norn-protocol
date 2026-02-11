import type { Metadata } from "next";
import { Inter, JetBrains_Mono } from "next/font/google";
import { ThemeProvider } from "next-themes";
import { Header } from "@/components/layout/header";
import { Footer } from "@/components/layout/footer";
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
  metadataBase: new URL("https://norn.network"),
  title: {
    default: "Norn Protocol — The chain is a courtroom, not a bank",
    template: "%s | Norn Protocol",
  },
  description:
    "Norn is a blockchain where users transact directly with cryptographic signatures. Instant finality, zero fees, private by default. The network only intervenes when there's a dispute.",
  icons: { icon: "/icon.svg" },
  keywords: [
    "norn",
    "norn protocol",
    "blockchain",
    "layer 1",
    "cryptocurrency",
    "zero fee blockchain",
    "instant finality",
    "private blockchain",
    "bilateral transactions",
    "smart contracts",
    "webassembly",
    "rust blockchain",
  ],
  authors: [{ name: "Norn Protocol" }],
  openGraph: {
    title: "Norn Protocol — The chain is a courtroom, not a bank",
    description:
      "A blockchain where users transact directly. Instant finality, zero fees, private by default.",
    siteName: "Norn Protocol",
    url: "https://norn.network",
    type: "website",
    locale: "en_US",
  },
  twitter: {
    card: "summary_large_image",
    title: "Norn Protocol",
    description:
      "A blockchain where users transact directly. Instant finality, zero fees, private by default.",
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
    canonical: "https://norn.network",
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
          <div className="relative flex min-h-screen flex-col">
            <Header />
            <main className="flex-1">{children}</main>
            <Footer />
          </div>
        </ThemeProvider>
      </body>
    </html>
  );
}
