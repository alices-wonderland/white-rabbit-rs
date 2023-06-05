import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import path from "path";
import { fileURLToPath } from "url";
import { visualizer } from "rollup-plugin-visualizer";
import { quasar, transformAssetUrls } from "@quasar/vite-plugin";

// https://vitejs.dev/config/
export default defineConfig(() => {
  return {
    plugins: [
      vue({
        template: { transformAssetUrls },
      }),
      visualizer(),
      quasar(),
    ],
    clearScreen: false,
    server: {
      port: 1420,
      strictPort: true,
    },
    envPrefix: ["VITE_", "TAURI_"],
    resolve: {
      alias: {
        "@desktop": path.resolve(path.dirname(fileURLToPath(import.meta.url)), "src"),
        "@core": path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../frontend-core/src"),
      },
    },
    test: {
      environment: "jsdom",
      coverage: {
        reporter: ["lcov", "html"],
      },
      setupFiles: ["../../vitest.setup.ts", "vitest.setup.ts"],
      deps: {
        inline: ["vuetify"],
      },
    },
    build: {
      target: ["esnext"],
      // don't minify for debug builds
      minify: !process.env.TAURI_DEBUG ? "esbuild" : false,
      // produce sourcemaps for debug builds
      sourcemap: !!process.env.TAURI_DEBUG,
      rollupOptions: {
        output: {
          manualChunks: (id): string | null => {
            if (id.includes("ag-charts")) {
              return "ag-charts";
            } else if (id.includes("@ag-grid-community/core")) {
              return "ag-grid-community-core";
            }
            return null;
          },
        },
      },
    },
  };
});
