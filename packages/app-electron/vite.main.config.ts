import type { ConfigEnv } from "vite";
import { defineConfig, mergeConfig } from "vite";
import { getBuildConfig, getBuildDefine, external, pluginHotRestart } from "./vite.base.config";
import path from "path";
import { fileURLToPath } from "url";

// https://vitejs.dev/config
export default defineConfig((env) => {
  const forgeEnv = env as ConfigEnv<"build">;
  const { forgeConfigSelf } = forgeEnv;
  const define = getBuildDefine(forgeEnv);

  const config = {
    build: {
      lib: {
        entry: forgeConfigSelf.entry!,
        fileName: () => "[name].js",
        formats: ["cjs"],
      },
      rollupOptions: {
        external,
      },
    },
    plugins: [pluginHotRestart("restart")],
    define,
    resolve: {
      // Load the Node.js entry.
      mainFields: ["module", "jsnext:main", "jsnext"],
      alias: {
        "@": path.resolve(path.dirname(fileURLToPath(import.meta.url)), "src"),
        "@core": path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../frontend-core/src"),
      },
    },
  };

  return mergeConfig(getBuildConfig(forgeEnv), config);
});
