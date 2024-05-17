import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import path from "path";
import { fileURLToPath } from "url";
import { visualizer } from "rollup-plugin-visualizer";
import vueI18n from "@intlify/unplugin-vue-i18n/vite";
import { quasar, transformAssetUrls } from "@quasar/vite-plugin";

// https://vitejs.dev/config/
export default defineConfig(() => {
  return {
    plugins: [
      vue({
        template: { transformAssetUrls },
      }),
      vueI18n({
        include: [
          path.resolve(
            path.dirname(fileURLToPath(import.meta.url)),
            "../frontend-core/src/locales/**",
          ),
        ],
      }),
      quasar(),
      visualizer(),
    ],
    clearScreen: false,
    server: {
      strictPort: true,
    },
    envPrefix: ["VITE_", "TAURI_"],
    resolve: {
      alias: {
        "@": path.resolve(path.dirname(fileURLToPath(import.meta.url)), "src"),
        "@core": path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../frontend-core/src"),
      },
    },
    test: {
      environment: "jsdom",
      coverage: {
        reporter: ["lcov", "html"],
      },
      setupFiles: ["vitest.setup.ts"],
    },
    build: {
      target: ["esnext"],
      minify: !process.env.TAURI_DEBUG ? "esbuild" : false,
      sourcemap: !!process.env.TAURI_DEBUG,
      rollupOptions: {
        output: {
          manualChunks: (id): string | undefined => {
            if (id.includes("@ag-grid-community/core")) {
              return "ag-grid-community-core";
            } else if (id.includes("ag-charts-community")) {
              return "ag-charts-community";
            } else if (id.includes("ag-grid")) {
              return "ag-grid";
            }
          },
        },
      },
    },
  };
});
