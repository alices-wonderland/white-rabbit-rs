import { ChannelCredentials, Metadata } from "@grpc/grpc-js";
import { app, BrowserWindow, ipcMain } from "electron";
import { FindAllRequest, JournalServiceClient } from "../proto/gen/whiterabbit/journal/v1/journal";

export const initializeJournalApi = () => {
  const journalClient = new JournalServiceClient(
    process.env.VITE_API_URL_BASE ?? "[::1]:50051",
    ChannelCredentials.createInsecure(),
  );

  ipcMain.handle("journalApi.findById", async (_e, args) => {
    if (journalClient && args.id) {
      const metadata = new Metadata();
      metadata.set("authorization", "Bearer some-secret-token");

      return new Promise((resolve, reject) => {
        journalClient!.findById({ id: `${args.id}` }, metadata, (err, data) => {
          if (err) {
            reject(err);
          } else {
            resolve(data);
          }
        });
      });
    }
  });

  ipcMain.handle("journalApi.findAll", async (_e, args) => {
    if (journalClient && args.args) {
      const metadata = new Metadata();
      metadata.set("authorization", "Bearer some-secret-token");

      return new Promise((resolve, reject) => {
        journalClient!.findAll(FindAllRequest.fromPartial(args), metadata, (err, data) => {
          if (err) {
            reject(err);
          } else {
            resolve(data);
          }
        });
      });
    }
  });

  ipcMain.handle("journalApi.handleCommand", async (_e, args) => {
    return [];
  });
};
