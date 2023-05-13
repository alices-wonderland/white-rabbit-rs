import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import { fileURLToPath } from "url";
import { visualizer } from "rollup-plugin-visualizer";
import { quasar, transformAssetUrls } from "@quasar/vite-plugin";
import path from "path";

// https://vitejs.dev/config/
export default defineConfig(() => ({
  plugins: [
    vue({
      template: { transformAssetUrls },
    }),
    visualizer(),
    quasar(),
  ],
  resolve: {
    alias: {
      "@": path.resolve(path.dirname(fileURLToPath(import.meta.url)), "src"),
    },
  },
  build: {
    lib: {
      name: "TestRustAppCore",
      entry: fileURLToPath(new URL("./src/index.ts", import.meta.url)),
      formats: ["es", "cjs", "iife"],
      fileName: (format) => {
        switch (format) {
          case "es":
            return "index.mjs";
          case "cjs":
            return "index.cjs";
          case "iife":
            return "index.js";
          default:
            return "index.js";
        }
      },
    },
    minify: false,
    rollupOptions: {
      external: ["vue"],
      output: {
        exports: "named",
        globals: {
          vue: "Vue",
        },
      },
    },
  },
  test: {
    environment: "jsdom",
    coverage: {
      reporter: ["lcov", "html"],
    },
  },
}));
