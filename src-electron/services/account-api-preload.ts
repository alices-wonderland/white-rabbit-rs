import pick from "lodash/pick";
import { ipcRenderer } from "electron";

import { Model, FindAllArgs } from "src/services/api";
import {
  ACCOUNT_FIELDS,
  Account,
  AccountApi,
  AccountCommand,
  AccountField,
  AccountQuery,
  AccountSort,
} from "src/services/account";

// You cannot write API here, must back to electron side for referencing GRPC related model
class AccountApiPreloadImpl implements AccountApi {
  async handleCommand(command: AccountCommand): Promise<Account[]> {
    const results = await ipcRenderer.invoke("accountApi.handleCommand", { command });
    return AccountApiPreloadImpl.fromAll(results);
  }

  async findById(
    id: string,
    loadIncluded?: boolean,
  ): Promise<[Account, Map<string, Model<string>>] | undefined> {
    const resp = await ipcRenderer.invoke("accountApi.findById", { id, loadIncluded });
    if (resp.value) {
      const parsed = AccountApiPreloadImpl.from(resp.value);
      if (parsed) {
        return [parsed, new Map()];
      }
    }

    return undefined;
  }

  async findAll(
    args: FindAllArgs<AccountQuery, AccountSort>,
    loadIncluded?: boolean,
  ): Promise<[Account[], Map<string, Model<string>>]> {
    const resp = await ipcRenderer.invoke("accountApi.findAll", { args, loadIncluded });
    if (Array.isArray(resp.values)) {
      return [AccountApiPreloadImpl.fromAll(resp.values), new Map()];
    }

    return [[], new Map()];
  }

  static from(item: object): Account | undefined {
    for (const key of ACCOUNT_FIELDS) {
      if (!(key in item)) {
        return undefined;
      }
    }

    return new Account(pick(item, ACCOUNT_FIELDS) as Pick<Account, AccountField>);
  }

  static fromAll(items: object[]): Account[] {
    return items
      .map((item) => AccountApiPreloadImpl.from(item))
      .filter((item): item is Account => !!item);
  }
}

export const accountApiPreload = new AccountApiPreloadImpl();
