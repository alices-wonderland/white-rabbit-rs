import { defineStore } from "pinia";
import nodeProcess from "node:process";

type Props = {
  readonly apiUrlBase: string;
};

export const useConfigStore = defineStore("config", {
  state: (): Props => ({
    apiUrlBase:
      (process.env.SERVER ? nodeProcess.env.VITE_API_URL_BASE : process.env.VITE_API_URL_BASE) ??
      "",
  }),
});
