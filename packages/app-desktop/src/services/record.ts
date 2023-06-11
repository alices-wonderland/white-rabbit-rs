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
} from "@core/services";
import { Record_ } from "@core/services";
import { AbstractWriteApi } from "@desktop/services/api";

class RecordApiImpl extends AbstractWriteApi<Record_, RecordQuery, RecordCommand, RecordSort> {
  protected get findAllKey(): string {
    return "record_find_all";
  }

  protected get findByIdKey(): string {
    return "record_find_by_id";
  }

  protected get findPageKey(): string {
    return "record_find_page";
  }

  protected get handleCommandKey(): string {
    return "record_handle_command";
  }

  protected loadIncluded(models: Record_[]): Promise<Map<string, ReadModel>> {
    throw new Error("Method not implemented.");
  }

  protected convert(input: Record<string, unknown>): Record_ {
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
