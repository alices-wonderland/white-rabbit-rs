/// <reference types="vite/client" />

import { IpcRendererEvent } from "electron";

declare module "*.vue" {
  import type { DefineComponent } from "vue";
  // eslint-disable-next-line @typescript-eslint/ban-types, @typescript-eslint/no-explicit-any
  const component: DefineComponent<{}, {}, any>;
  export default component;
}

declare global {
  interface Window {
    api: {
      getPort(): Promise<number>;
      onPortUpdated(callback: (event: IpcRendererEvent, port?: number) => void);
    };
  }
}
