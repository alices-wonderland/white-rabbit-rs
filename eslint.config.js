import js from "@eslint/js";
import pluginVue from "eslint-plugin-vue";
import tseslint from "typescript-eslint";
import vueEslintParser from "vue-eslint-parser";
import globals from "globals";
import eslintConfigPrettier from "eslint-config-prettier";

/**
 * @type {import('@types/eslint').FlatConfig[]}
 */
export default [
  js.configs.recommended,
  ...tseslint.configs.recommended,
  ...pluginVue.configs["flat/recommended"],
  eslintConfigPrettier,
  {
    ignores: [
      "dist/",
      "src-capacitor/",
      "src-cordova/",
      ".quasar/",
      "node_modules/",
      "quasar.config.*.temporary.compiled*",
      "coverage/",
      "target/",
    ],
  },
  {
    languageOptions: {
      parser: tseslint.parser,
      globals: {
        ...globals.browser,
        ...globals.es2021,
        ...globals.node,
        ga: "readonly", // Google Analytics
        cordova: "readonly",
        __statics: "readonly",
        __QUASAR_SSR__: "readonly",
        __QUASAR_SSR_SERVER__: "readonly",
        __QUASAR_SSR_CLIENT__: "readonly",
        __QUASAR_SSR_PWA__: "readonly",
        process: "readonly",
        Capacitor: "readonly",
        chrome: "readonly",
      },
    },
  },
  {
    files: ["**/*.vue"],
    languageOptions: {
      parser: vueEslintParser,
      parserOptions: {
        parser: tseslint.parser,
      },
    },
  },
  {
    rules: {
      "@typescript-eslint/no-unused-vars": [
        "warn",
        {
          argsIgnorePattern: "^_",
          varsIgnorePattern: "^_",
          caughtErrorsIgnorePattern: "^_",
        },
      ],
    },
  },
];
