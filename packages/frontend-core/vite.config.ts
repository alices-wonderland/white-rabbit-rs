import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import { fileURLToPath } from "url";
import { visualizer } from "rollup-plugin-visualizer";
import vuetify from "vite-plugin-vuetify";
import path from "path";

// https://vitejs.dev/config/
export default defineConfig(() => ({
  plugins: [vue(), visualizer(), vuetify()],
  resolve: {
    alias: {
      "@core": path.resolve(path.dirname(fileURLToPath(import.meta.url)), "src"),
    },
  },
  test: {
    environment: "jsdom",
    coverage: {
      reporter: ["lcov", "html"],
    },
  },
}));
