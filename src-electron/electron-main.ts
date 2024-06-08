import { app, BrowserWindow, ipcMain } from "electron";
import path from "path";
import os from "os";
import { fileURLToPath } from "url";
import { ChannelCredentials, Metadata } from "@grpc/grpc-js";

import { AccountServiceClient } from "./proto/gen/whiterabbit/account/v1/account";
import { JournalQuery, JournalServiceClient } from "./proto/gen/whiterabbit/journal/v1/journal";

// needed in case process is undefined under Linux
const platform = process.platform || os.platform();

const currentDir = fileURLToPath(new URL(".", import.meta.url));

let mainWindow: BrowserWindow | undefined;
let journalClient: JournalServiceClient | undefined;
let accountClient: AccountServiceClient | undefined;

function createWindow() {
  /**
   * Initial window options
   */
  mainWindow = new BrowserWindow({
    icon: path.resolve(currentDir, "icons/icon.png"), // tray icon
    width: 1000,
    height: 600,
    useContentSize: true,
    webPreferences: {
      contextIsolation: true,
      // More info: https://v2.quasar.dev/quasar-cli-vite/developing-electron-apps/electron-preload-script
      preload: path.resolve(
        currentDir,
        path.join(
          process.env.QUASAR_ELECTRON_PRELOAD_FOLDER,
          "electron-preload" + process.env.QUASAR_ELECTRON_PRELOAD_EXTENSION,
        ),
      ),
    },
  });

  if (process.env.DEV) {
    mainWindow.loadURL(process.env.APP_URL);
  } else {
    mainWindow.loadFile("index.html");
  }

  if (process.env.DEBUGGING) {
    // if on DEV or Production with debug enabled
    mainWindow.webContents.openDevTools();
  } else {
    // we're on production; no access to devtools pls
    mainWindow.webContents.on("devtools-opened", () => {
      mainWindow?.webContents.closeDevTools();
    });
  }

  mainWindow.on("closed", () => {
    mainWindow = undefined;
  });
}

app.whenReady().then(() => {
  journalClient = new JournalServiceClient(
    process.env.VITE_API_URL_BASE ?? "[::1]:50051",
    ChannelCredentials.createInsecure(),
  );
  accountClient = new AccountServiceClient(
    process.env.VITE_API_URL_BASE ?? "[::1]:50051",
    ChannelCredentials.createInsecure(),
  );
  createWindow();
});

app.on("window-all-closed", () => {
  if (platform !== "darwin") {
    app.quit();
  }
});

app.on("activate", () => {
  if (mainWindow === undefined) {
    createWindow();
  }
});

ipcMain.handle("journalFindById", async (_e, args) => {
  console.log("journalFindById: ", args);
  if (journalClient && args.id) {
    const metadata = new Metadata();
    metadata.set("authorization", "Bearer some-secret-token");

    return new Promise((resolve, reject) => {
      journalClient!.findById({ id: `${args.id}` }, metadata, (err, data) => {
        if (err) {
          reject(err);
        } else {
          resolve(data.value);
        }
      });
    });
  }
});

ipcMain.handle("journalFindAll", async (_e, args) => {
  console.log("journalFindAll: ", args);
  if (journalClient && args.query) {
    const metadata = new Metadata();
    metadata.set("authorization", "Bearer some-secret-token");

    return new Promise((resolve, reject) => {
      journalClient!.findAll(
        { query: JournalQuery.fromPartial(args.query) },
        metadata,
        (err, data) => {
          if (err) {
            reject(err);
          } else {
            resolve([data.values, new Map()]);
          }
        },
      );
    });
  }
});

ipcMain.handle("accountFindById", async (_e, args) => {
  if (accountClient && args.id) {
    const metadata = new Metadata();
    metadata.set("authorization", "Bearer some-secret-token");

    return new Promise((resolve, reject) => {
      accountClient!.findById({ id: `${args.id}` }, metadata, (err, data) => {
        if (err) {
          reject(err);
        } else {
          resolve(data.value);
        }
      });
    });
  }
});

ipcMain.handle("accountFindAll", async (_e, args) => {
  if (accountClient && args.query) {
    const metadata = new Metadata();
    metadata.set("authorization", "Bearer some-secret-token");

    return new Promise((resolve, reject) => {
      accountClient!.findAll({ query: args.query }, metadata, (err, data) => {
        if (err) {
          reject(err);
        } else {
          resolve([data.values, new Map()]);
        }
      });
    });
  }
});
