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
import { AbstractWriteApi, type HttpMethod } from "./api";
import { journalApi } from "./journal";
import isString from "lodash/isString";
import { validate as uuidValidate } from "uuid";

class AccountApiImpl extends AbstractWriteApi<Account, AccountQuery, AccountCommand, AccountSort> {
  protected override get modelType(): string {
    return "/accounts";
  }

  protected override async loadIncluded(models: Account[]): Promise<Map<string, Model>> {
    const journalIds = new Set(models.map((model) => model.journalId));
    const journals = await journalApi.findAll({ query: { id: [...journalIds] } as JournalQuery });
    return toMap(journals[0]);
  }

  protected override convert(input: Record<string, unknown>): Account | undefined {
    if (
      isString(input.id) &&
      uuidValidate(input.id) &&
      isString(input.name) &&
      isString(input.description) &&
      isString(input.unit) &&
      isString(input.type) &&
      Array.isArray(input.tags) &&
      isString(input.journalId)
    ) {
      return new Account({
        id: input.id,
        name: input.name,
        description: input.description,
        unit: input.unit,
        type: input.type as AccountType,
        tags: input.tags as string[],
        journalId: input.journalId,
      });
    }

    return undefined;
  }

  protected parseCommand(
    _command: AccountCommand,
  ): [string | null, HttpMethod, Record<string, unknown>] {
    return [null, "GET", {}];
  }
}

export const accountApi: AccountApi = new AccountApiImpl();
