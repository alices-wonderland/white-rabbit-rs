import type {
  AccountApi,
  AccountCommand,
  AccountQuery,
  AccountSort,
  Permission,
  AccountType,
  ReadModel,
} from "@core/services";
import { Account } from "@core/services";
import { AbstractWriteApi } from "@desktop/services/api";

class AccountApiImpl extends AbstractWriteApi<Account, AccountQuery, AccountCommand, AccountSort> {
  protected get findAllKey(): string {
    return "account_find_all";
  }

  protected get findByIdKey(): string {
    return "account_find_by_id";
  }

  protected get findPageKey(): string {
    return "account_find_page";
  }

  protected get handleCommandKey(): string {
    return "account_handle_command";
  }

  protected loadIncluded(models: Account[]): Promise<Map<string, ReadModel>> {
    throw new Error("Method not implemented.");
  }

  protected convert(input: Record<string, unknown>): Account {
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
