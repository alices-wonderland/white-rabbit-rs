import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import path from "path";
import { fileURLToPath } from "url";
import { visualizer } from "rollup-plugin-visualizer";
import vuetify from "vite-plugin-vuetify";
import vueI18n from "@intlify/unplugin-vue-i18n/vite";

// https://vitejs.dev/config/
export default defineConfig(() => {
  return {
    plugins: [
      vue(),
      vueI18n({
        include: [
          path.resolve(
            path.dirname(fileURLToPath(import.meta.url)),
            "../frontend-core/src/locales/**",
          ),
        ],
      }),
      vuetify(),
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
