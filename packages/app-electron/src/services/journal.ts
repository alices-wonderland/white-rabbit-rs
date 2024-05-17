import type {
  EntryCommand,
  JournalApi,
  JournalCommand,
  JournalQuery,
  JournalSort,
} from "@core/services";
import { Journal } from "@core/services";
import { AbstractWriteApi, type HttpMethod } from "./api";

class JournalApiImpl extends AbstractWriteApi<Journal, JournalQuery, JournalCommand, JournalSort> {
  protected override get modelType(): string {
    return "/journals";
  }

  protected override convert(input: Record<string, unknown>): Journal {
    return new Journal({
      id: input.id as string,
      name: input.name as string,
      description: input.description as string,
      unit: input.unit as string,
      tags: input.tags as string[],
    });
  }

  protected parseCommand(
    command: JournalCommand,
  ): [string | null, HttpMethod, Record<string, unknown>] {
    return [null, "GET", {}];
  }
}

export const journalApi: JournalApi = new JournalApiImpl();
