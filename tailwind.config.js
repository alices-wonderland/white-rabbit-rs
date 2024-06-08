import path from "path";

export default {
  content: [
    path.resolve(__dirname, "./packages/*/index.html"),
    path.resolve(__dirname, "./packages/*/src/**/*.{vue,js,ts,jsx,tsx}"),
  ],
  theme: {
    extend: {},
  },
  plugins: [],
};
