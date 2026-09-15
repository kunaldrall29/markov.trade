import type { NextConfig } from "next";

const api = process.env.API_INTERNAL_URL || "http://127.0.0.1:4000";

const nextConfig: NextConfig = {
  output: "standalone",
  transpilePackages: ["@markov/facts"],
  images: { unoptimized: true },
  async rewrites() {
    return [{ source: "/v1/:path*", destination: `${api}/:path*` }];
  },
  webpack: (config, { isServer }) => {
    config.resolve.fallback = {
      ...(config.resolve.fallback || {}),
      fs: false,
      net: false,
      tls: false,
      crypto: false,
    };
    if (!isServer && Array.isArray(config.externals)) {
      config.externals.push("pino-pretty", "lokijs", "encoding");
    }
    return config;
  },
};

export default nextConfig;
