import { app, BrowserWindow, ipcMain } from "electron";
import path from "node:path";
import { execFile, ChildProcess, type ExecFileOptions } from "node:child_process";
import squirrelCheck from "@/squirrel-check";
import net from "net";

// Handle creating/removing shortcuts on Windows when installing/uninstalling.
if (squirrelCheck()) {
  app.quit();
}

let javaProcess: ChildProcess;

async function getFreePort() {
  return new Promise<number | undefined>((res) => {
    const srv: net.Server = net.createServer();
    srv.listen(0, () => {
      let port: number | undefined = undefined;
      const address = srv.address();
      if (address && typeof address === "object") {
        port = address.port;
      }
      srv.close(() => res(port));
    });
  });
}

const createWindow = async () => {
  const mainWindow = new BrowserWindow({
    width: 1920,
    height: 1080,
    webPreferences: {
      preload: path.join(__dirname, "preload.js"),
    },
  });

  let port: number | undefined = 8080;

  if (MAIN_WINDOW_VITE_DEV_SERVER_URL) {
    void mainWindow.loadURL(MAIN_WINDOW_VITE_DEV_SERVER_URL);
  } else {
    port = await getFreePort();
    void mainWindow.loadFile(
      path.join(__dirname, `../renderer/${MAIN_WINDOW_VITE_NAME}/index.html`),
    );

    const exePath = path.resolve(app.getAppPath(), "../endpoint-grpc");

    javaProcess = execFile(exePath, {
      env: {
        WHITE_RABBIT_PORT: port,
        WHITE_RABBIT_PASSWORD: "password password",
      },
    } as ExecFileOptions);
  }

  mainWindow.webContents.openDevTools();

  ipcMain.handle("get-port", () => {
    return port;
  });
};

app.on("ready", async () => {
  await createWindow();
});

app.on("window-all-closed", () => {
  javaProcess?.kill();

  if (process.platform !== "darwin") {
    app.quit();
  }
});

app.on("activate", async () => {
  if (BrowserWindow.getAllWindows().length === 0) {
    await createWindow();
  }
});
