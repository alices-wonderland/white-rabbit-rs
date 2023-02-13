// eslint-disable-next-line @typescript-eslint/no-var-requires
const path = require("path");

/** @type {import('tailwindcss').Config} */
module.exports = {
  corePlugins: {
    preflight: false,
  },
  content: [
    path.resolve(__dirname, "packages/*/index.html"),
    path.resolve(__dirname, "packages/*/src/**/*.{vue,js,ts,jsx,tsx}"),
  ],
  theme: {
    extend: {},
  },
};
