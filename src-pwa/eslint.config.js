import globals from "globals";
import { resolve } from "node:path";

/**
 * @type {import('@types/eslint').FlatConfig[]}
 */
export default [
  {
    languageOptions: {
      parserOptions: {
        project: resolve(__dirname, "./tsconfig.json"),
      },
    },
  },
  {
    files: ["custom-service-worker.ts"],
    languageOptions: {
      globals: {
        ...globals.serviceworker,
      },
    },
  },
];
