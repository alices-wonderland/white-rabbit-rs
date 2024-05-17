import { contextBridge, ipcRenderer, type IpcRendererEvent } from "electron";

contextBridge.exposeInMainWorld("api", {
  getPort: async (): Promise<number> => {
    const port = await ipcRenderer.invoke("get-port");
    if (Number.isInteger(port)) {
      return port;
    }
    return parseInt(`${port}`);
  },
  onPortUpdated: (callback: (event: IpcRendererEvent, port?: number) => void) =>
    ipcRenderer.on("update:port", callback),
});
