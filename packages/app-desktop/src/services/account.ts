import type {
  FindAllArgs,
  FindPageArgs,
  AccountApi,
  AccountCommand,
  AccountQuery,
  AccountSort,
  Page,
  Permission,
  ReadModel,
  AccountType,
} from "@core/services";
import { Account } from "@core/services";

class AccountApiImpl implements AccountApi {
  private convert(input: Record<string, unknown>): Account {
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

  async findAll(args: FindAllArgs<AccountQuery, AccountSort>): Promise<[Account[], ReadModel[]]> {
    return [[], []];
  }

  async findPage(
    args: FindPageArgs<AccountQuery, AccountSort>
  ): Promise<[Page<Account>, ReadModel[]]> {
    return [{ hasNext: false, hasPrevious: false, items: [] }, []];
  }

  async handle(command: AccountCommand): Promise<Account[]> {
    return [];
  }

  async findById(id: string): Promise<[Account, ReadModel[]] | null> {
    return null;
  }
}

export const accountApi: AccountApi = new AccountApiImpl();
