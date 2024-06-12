import { ipcMain } from "electron";
import { ChannelCredentials } from "@grpc/grpc-js";

import { Model, FindAllArgs } from "src/services/api";
import {
  Account,
  AccountApi,
  AccountCommand,
  AccountQuery,
  AccountSort,
  AccountType,
} from "src/services/account";

import {
  AccountServiceClient,
  FindAllRequest,
  FindByIdRequest,
  AccountType as AccountTypeProto,
  Account as AccountProto,
  FindByIdResponse,
  FindAllResponse,
} from "../proto/gen/whiterabbit/account/v1/account";
import { fromIncluded } from ".";

export const ACCOUNT_TYPE_URL = "/whiterabbit.account.v1.Account";

export class AccountApiImpl implements AccountApi {
  private readonly client: AccountServiceClient;

  constructor() {
    this.client = new AccountServiceClient(
      process.env.VITE_API_URL_BASE ?? "[::1]:50051",
      ChannelCredentials.createInsecure(),
    );
  }

  initialize() {
    console.log("[Electron.main] initializing Account API");

    ipcMain.handle("accountApi.findById", async (_e, id, loadIncluded) =>
      this.findById(id, loadIncluded),
    );
    ipcMain.handle("accountApi.findAll", async (_e, args, loadIncluded) =>
      this.findAll(args, loadIncluded),
    );
    ipcMain.handle("accountApi.handleCommand", async (_e, command) => this.handleCommand(command));
  }

  uninitialize() {
    ipcMain.removeAllListeners("accountApi.findById");
    ipcMain.removeAllListeners("accountApi.findAll");
    ipcMain.removeAllListeners("accountApi.handleCommand");
  }

  async handleCommand(_command: AccountCommand): Promise<Account[]> {
    throw new Error("Method not implemented.");
  }

  async findById(
    id: string,
    _loadIncluded?: boolean,
  ): Promise<[Account, Map<string, Model<string>>] | undefined> {
    const resp = await new Promise<FindByIdResponse>((resolve, reject) =>
      this.client.findById(FindByIdRequest.fromPartial({ id }), (err, resp) => {
        if (err) {
          reject(err);
        } else {
          resolve(resp);
        }
      }),
    );

    const parsed = resp.value ? fromProto(resp.value) : undefined;
    if (parsed) {
      return [parsed, fromIncluded(resp.included)];
    } else {
      return undefined;
    }
  }

  async findAll(
    args: FindAllArgs<AccountQuery, AccountSort>,
    _loadIncluded?: boolean,
  ): Promise<[Account[], Map<string, Model<string>>]> {
    const resp = await new Promise<FindAllResponse | undefined>((resolve, reject) => {
      const req = toProtoFindAllArgs(args);
      if (req) {
        this.client.findAll(req, (err, resp) => {
          if (err) {
            reject(err);
          } else {
            resolve(resp);
          }
        });
      } else {
        resolve(undefined);
      }
    });

    if (!resp) {
      return [[], new Map()];
    }

    const parsed = resp.values
      .map((value) => fromProto(value))
      .filter((value): value is Account => !!value);
    return [parsed, fromIncluded(resp.included)];
  }
}

function toProtoFindAllArgs(
  args: FindAllArgs<AccountQuery, AccountSort>,
): FindAllRequest | undefined {
  let type: AccountTypeProto | undefined;
  switch (args.query?.type) {
    case "Asset":
      type = AccountTypeProto.ASSET;
      break;
    case "Expense":
      type = AccountTypeProto.EXPENSE;
      break;
    case "Equity":
      type = AccountTypeProto.EQUITY;
      break;
    case "Income":
      type = AccountTypeProto.INCOME;
      break;
    case "Liability":
      type = AccountTypeProto.LIABILITY;
      break;
  }

  if (args.query?.type && type) {
    return FindAllRequest.fromPartial({ query: { ...args.query, type } });
  } else if (!args.query) {
    return FindAllRequest.fromPartial({});
  }

  return undefined;
}

export function fromProto(model: AccountProto): Account | undefined {
  let type: AccountType | undefined;
  switch (model.type) {
    case AccountTypeProto.ASSET:
      type = "Asset";
      break;
    case AccountTypeProto.EXPENSE:
      type = "Expense";
      break;
    case AccountTypeProto.EQUITY:
      type = "Equity";
      break;
    case AccountTypeProto.INCOME:
      type = "Income";
      break;
    case AccountTypeProto.LIABILITY:
      type = "Liability";
      break;
  }

  if (type) {
    return new Account({ ...model, type });
  }

  return undefined;
}
