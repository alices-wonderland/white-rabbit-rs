import { defineConfig } from "vite";
import type { ConfigEnv } from "vite";
import vue from "@vitejs/plugin-vue";
import path from "path";
import { fileURLToPath } from "url";
import { visualizer } from "rollup-plugin-visualizer";
import vueI18n from "@intlify/unplugin-vue-i18n/vite";
import { quasar, transformAssetUrls } from "@quasar/vite-plugin";
import { pluginExposeRenderer } from "./vite.base.config";

// https://vitejs.dev/config/
export default defineConfig((env) => {
  const forgeEnv = env as ConfigEnv<"renderer">;
  const { root, mode, forgeConfigSelf } = forgeEnv;
  const name = forgeConfigSelf?.name ?? "";

  return {
    root,
    mode,
    base: "./",
    clearScreen: false,
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
      pluginExposeRenderer(name),
    ],
    resolve: {
      preserveSymlinks: true,
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
    },
    build: {
      outDir: `.vite/renderer/${name}`,
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
