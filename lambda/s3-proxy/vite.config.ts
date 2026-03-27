import { defineConfig } from "vite";

const nodeBuiltins = [
  /^node:/,
  "http",
  "https",
  "url",
  "crypto",
  "path",
  "fs",
  "stream",
  "buffer",
  "util",
  "os",
  "events",
  "tls",
  "net",
  "zlib",
  "assert",
  "querystring",
  "string_decoder",
  "timers",
  "child_process",
  "dns",
  "domain",
  "punycode",
  "readline",
  "repl",
  "vm",
  "worker_threads",
];

export default defineConfig({
  build: {
    lib: {
      entry: "src/index.ts",
      formats: ["cjs"],
      fileName: () => "index.js",
    },
    rollupOptions: {
      external: nodeBuiltins,
    },
    outDir: "dist/build",
    target: "node22",
    sourcemap: false,
    minify: false,
  },
});
