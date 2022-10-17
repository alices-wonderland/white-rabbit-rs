import { defineConfig, Plugin } from "vite";
import vue from "@vitejs/plugin-vue";
import path from "path";
import { fileURLToPath } from "url";
import { visualizer } from "rollup-plugin-visualizer";
import vuetify from "vite-plugin-vuetify";
import vueI18n from "@intlify/vite-plugin-vue-i18n";

// https://vitejs.dev/config/
export default defineConfig(() => ({
  plugins: [
    vue(),
    visualizer() as Plugin,
    vuetify({ autoImport: true }),
    vueI18n({
      include: path.resolve(__dirname, "./src/locales/**"),
    }),
  ],
  resolve: {
    alias: {
      "@shared": path.resolve(
        path.dirname(fileURLToPath(import.meta.url)),
        "src"
      ),
    },
  },
  test: {
    environment: "jsdom",
    coverage: {
      reporter: ["lcov", "html"],
    },
    setupFiles: ["../../vitest.setup.ts"],
    deps: {
      inline: ["vuetify", "element-plus"],
    },
  },
}));
