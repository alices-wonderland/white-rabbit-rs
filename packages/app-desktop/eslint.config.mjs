import configs from "../../eslint.config.base.mjs";

export default [
  ...configs,
  {
    ignores: ["./src-tauri/gen/schemas"],
  },
];
