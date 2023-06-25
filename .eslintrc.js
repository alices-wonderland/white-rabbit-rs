/**
 * @type {import('@types/eslint').Linter.Config}
 */
module.exports = {
  root: true,
  extends: ["eslint:recommended"],
  parserOptions: {
    ecmaVersion: "latest",
    sourceType: "module",
  },
  env: {
    node: true,
  },
  overrides: [
    {
      files: ["**/*.ts", "**/*.vue"],
      parser: "vue-eslint-parser",
      parserOptions: {
        parser: "@typescript-eslint/parser",
      },
      env: {
        browser: true,
      },
      extends: [
        "plugin:@typescript-eslint/recommended",
        "plugin:sonarjs/recommended",
        "plugin:vue/vue3-recommended",
        "prettier",
      ],
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
  ],
};
