import { ChannelCredentials, Metadata } from "@grpc/grpc-js";
import { app, BrowserWindow, ipcMain } from "electron";
import { FindAllRequest, AccountServiceClient } from "../proto/gen/whiterabbit/account/v1/account";

export const initializeAccountApi = () => {
  const accountClient = new AccountServiceClient(
    process.env.VITE_API_URL_BASE ?? "[::1]:50051",
    ChannelCredentials.createInsecure(),
  );

  ipcMain.handle("accountApi.findById", async (_e, args) => {
    if (accountClient && args.id) {
      const metadata = new Metadata();
      metadata.set("authorization", "Bearer some-secret-token");

      return new Promise((resolve, reject) => {
        accountClient!.findById({ id: `${args.id}` }, metadata, (err, data) => {
          if (err) {
            reject(err);
          } else {
            resolve(data);
          }
        });
      });
    }
  });

  ipcMain.handle("accountApi.findAll", async (_e, args) => {
    if (accountClient && args.args) {
      const metadata = new Metadata();
      metadata.set("authorization", "Bearer some-secret-token");

      return new Promise((resolve, reject) => {
        accountClient!.findAll(FindAllRequest.fromPartial(args), metadata, (err, data) => {
          if (err) {
            reject(err);
          } else {
            resolve(data);
          }
        });
      });
    }
  });

  ipcMain.handle("accountApi.handleCommand", async (_e, args) => {
    return [];
  });
};
