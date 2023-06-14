import type {
  RecordApi,
  RecordCommand,
  RecordQuery,
  RecordSort,
  Permission,
  RecordType,
  RecordItem,
  RecordState,
  ReadModel,
  JournalQuery,
  AccountQuery,
} from "@core/services";
import { Record_ } from "@core/services";
import { AbstractWriteApi } from "./api";
import { journalApi } from "./journal";
import { toMap } from "@core/utils";
import { accountApi } from "./account";

class RecordApiImpl extends AbstractWriteApi<Record_, RecordQuery, RecordCommand, RecordSort> {
  protected override get findAllKey(): string {
    return "record_find_all";
  }

  protected override get findByIdKey(): string {
    return "record_find_by_id";
  }

  protected override get findPageKey(): string {
    return "record_find_page";
  }

  protected override get handleCommandKey(): string {
    return "record_handle_command";
  }

  protected override async loadIncluded(models: Record_[]): Promise<Map<string, ReadModel>> {
    const journalIds = new Set(models.map((model) => model.journal));
    const journals = await journalApi.findAll({ query: { id: [...journalIds] } as JournalQuery });

    const accountIds = new Set(models.flatMap((model) => model.items).map((item) => item.account));
    const accounts = await accountApi.findAll({ query: { id: [...accountIds] } as AccountQuery });

    return toMap([...journals[0], ...accounts[0]]);
  }

  protected override convert(input: Record<string, unknown>): Record_ {
    return new Record_({
      id: input.id as string,
      permission: input.permission as Permission,
      journal: input.journal as string,
      name: input.name as string,
      description: input.description as string,
      type: input.type as RecordType,
      date: new Date(input.date as string),
      tags: input.tags as string[],
      items: input.items as RecordItem[],
      state: input.state as RecordState,
    });
  }
}

export const recordApi: RecordApi = new RecordApiImpl();
