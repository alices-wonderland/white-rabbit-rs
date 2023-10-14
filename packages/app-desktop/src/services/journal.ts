import type { JournalApi, JournalCommand, JournalQuery, JournalSort } from "@core/services";
import { Journal } from "@core/services";
import { AbstractWriteApi } from "./api";

class JournalApiImpl extends AbstractWriteApi<Journal, JournalQuery, JournalCommand, JournalSort> {
  protected override get findAllKey(): string {
    return "journal_find_all";
  }

  protected override get findByIdKey(): string {
    return "journal_find_by_id";
  }

  protected override get handleCommandKey(): string {
    return "journal_handle_command";
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
}

export const journalApi: JournalApi = new JournalApiImpl();
