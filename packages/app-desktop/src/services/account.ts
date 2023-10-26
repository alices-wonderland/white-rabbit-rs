import type {
  AccountApi,
  AccountCommand,
  AccountQuery,
  AccountSort,
  AccountType,
  JournalQuery,
  Model,
} from "@core/services";
import { Account } from "@core/services";
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

  protected override get handleCommandKey(): string {
    return "account_handle_command";
  }

  protected override async loadIncluded(models: Account[]): Promise<Map<string, Model>> {
    const journalIds = new Set(models.map((model) => model.journalId));
    const journals = await journalApi.findAll({ query: { id: [...journalIds] } as JournalQuery });
    return toMap(journals[0]);
  }

  protected override convert(input: Record<string, unknown>): Account {
    return new Account({
      id: input.id as string,
      name: input.name as string,
      description: input.description as string,
      unit: input.unit as string,
      type: input.type as AccountType,
      tags: input.tags as string[],
      journalId: input.journalId as string,
    });
  }
}

export const accountApi: AccountApi = new AccountApiImpl();
