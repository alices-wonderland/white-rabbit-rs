import { ipcMain } from "electron";
import { ChannelCredentials } from "@grpc/grpc-js";

import { Model, FindAllArgs } from "src/services/api";
import {
  Journal,
  JournalApi,
  JournalCommand,
  JournalQuery,
  JournalSort,
} from "src/services/journal";
import {
  JournalServiceClient,
  FindAllRequest,
  FindByIdRequest,
  Journal as JournalProto,
} from "../proto/gen/whiterabbit/journal/v1/journal";

export const JOURNAL_TYPE_URL = "/whiterabbit.journal.v1.Journal";

export class JournalApiImpl implements JournalApi {
  private readonly client: JournalServiceClient;

  constructor() {
    this.client = new JournalServiceClient(
      process.env.VITE_API_URL_BASE ?? "[::1]:50051",
      ChannelCredentials.createInsecure(),
    );
  }

  initialize() {
    console.log("[Electron.main] initializing Journal API");
    ipcMain.handle("journalApi.findById", async (_e, id, loadIncluded) =>
      this.findById(id, loadIncluded),
    );
    ipcMain.handle("journalApi.findAll", async (_e, args, loadIncluded) =>
      this.findAll(args, loadIncluded),
    );
    ipcMain.handle("journalApi.handleCommand", async (_e, command) => this.handleCommand(command));
  }

  uninitialize() {
    ipcMain.removeAllListeners("journalApi.findById");
    ipcMain.removeAllListeners("journalApi.findAll");
    ipcMain.removeAllListeners("journalApi.handleCommand");
  }

  async handleCommand(_command: JournalCommand): Promise<Journal[]> {
    throw new Error("Method not implemented.");
  }

  async findById(
    id: string,
    _loadIncluded?: boolean,
  ): Promise<[Journal, Map<string, Model<string>>] | undefined> {
    const result = await new Promise<Journal | undefined>((resolve, reject) =>
      this.client.findById(FindByIdRequest.fromPartial({ id }), (err, resp) => {
        if (err) {
          reject(err);
        } else if (resp.value) {
          resolve(new Journal(resp.value));
        } else {
          resolve(undefined);
        }
      }),
    );

    return result ? [result, new Map()] : undefined;
  }

  async findAll(
    args: FindAllArgs<JournalQuery, JournalSort>,
    _loadIncluded?: boolean,
  ): Promise<[Journal[], Map<string, Model<string>>]> {
    const result = await new Promise<Journal[]>((resolve, reject) =>
      this.client.findAll(FindAllRequest.fromPartial({ query: args.query }), (err, resp) => {
        if (err) {
          reject(err);
        } else {
          resolve(resp.values.map((value) => new Journal(value)));
        }
      }),
    );

    return [result, new Map()];
  }
}

export function fromProto(model: JournalProto): Journal {
  return new Journal(model);
}
