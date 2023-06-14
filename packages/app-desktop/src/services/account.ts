import {
  Account,
  type AccountApi,
  type AccountCommand,
  type AccountQuery,
  type AccountSort,
  type AccountType,
  type JournalQuery,
  type Permission,
  type ReadModel,
} from "@core/services";
import { toMap } from "@core/utils";
import { AbstractWriteApi } from "./api";
import { journalApi } from "./journal";

class AccountApiImpl extends AbstractWriteApi<Account, AccountQuery, AccountCommand, AccountSort> {
  protected override get findAllKey(): string {
    return "account_find_all";
  }

  protected override get findByIdKey(): string {
    return "account_find_by_id";
  }

  protected override get findPageKey(): string {
    return "account_find_page";
  }

  protected override get handleCommandKey(): string {
    return "account_handle_command";
  }

  protected override async loadIncluded(models: Account[]): Promise<Map<string, ReadModel>> {
    const journalIds = new Set(models.map((model) => model.journal));
    const journals = await journalApi.findAll({ query: { id: [...journalIds] } as JournalQuery });
    return toMap(journals[0]);
  }

  protected override convert(input: Record<string, unknown>): Account {
    return new Account({
      id: input.id as string,
      permission: input.permission as Permission,
      name: input.name as string,
      description: input.description as string,
      unit: input.unit as string,
      type: input.type as AccountType,
      tags: input.tags as string[],
      journal: input.journal as string,
    });
  }
}

export const accountApi: AccountApi = new AccountApiImpl();
