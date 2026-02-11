import type { MetadataRoute } from "next";

export default function sitemap(): MetadataRoute.Sitemap {
  const base = "https://norn.network";

  const docs = [
    "quickstart",
    "architecture",
    "wallet",
    "wallet-extension",
    "names",
    "tokens",
    "looms",
    "explorer",
    "contributing",
    "sdk/typescript",
    "sdk/contracts",
  ];

  return [
    {
      url: base,
      lastModified: new Date(),
      changeFrequency: "weekly",
      priority: 1,
    },
    {
      url: `${base}/docs`,
      lastModified: new Date(),
      changeFrequency: "weekly",
      priority: 0.9,
    },
    ...docs.map((slug) => ({
      url: `${base}/docs/${slug}`,
      lastModified: new Date(),
      changeFrequency: "monthly" as const,
      priority: 0.7,
    })),
  ];
}
