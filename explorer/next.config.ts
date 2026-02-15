import type { NextConfig } from "next";
import { resolve } from "path";

const nextConfig: NextConfig = {
  // Pin the workspace root to the monorepo so Next.js doesn't pick up a
  // stray lockfile/node_modules higher in the filesystem tree.
  outputFileTracingRoot: resolve(__dirname, ".."),
};

export default nextConfig;
