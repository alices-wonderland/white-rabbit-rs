import { defineConfig, Plugin } from "vite";
import vue from "@vitejs/plugin-vue";
import path from "path";
import { fileURLToPath } from "url";
import { visualizer } from "rollup-plugin-visualizer";
import vuetify from "vite-plugin-vuetify";

// https://vitejs.dev/config/
export default defineConfig(() => {
  return {
    plugins: [
      vue(),
      visualizer() as Plugin,
      vuetify({
        autoImport: true,
      }),
    ],
    resolve: {
      alias: {
        "@": path.resolve(path.dirname(fileURLToPath(import.meta.url)), "src"),
        "@shared": path.resolve("../frontend-shared/src"),
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
      rollupOptions: {
        output: {
          manualChunks: (id): string | null => {
            if (id.includes("@ag-grid-community/core")) {
              return "ag-grid-community-core";
            } else if (id.includes("@ag-grid")) {
              return "ag-grid";
            }
            return null;
          },
        },
      },
    },
  };
});
