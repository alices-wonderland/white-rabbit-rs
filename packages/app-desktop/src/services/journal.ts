import type {
  JournalApi,
  JournalCommand,
  JournalQuery,
  JournalSort,
  Permission,
  ReadModel,
} from "@core/services";
import { Journal } from "@core/services";
import { AbstractWriteApi } from "@desktop/services/api";

class JournalApiImpl extends AbstractWriteApi<Journal, JournalQuery, JournalCommand, JournalSort> {
  protected get findAllKey(): string {
    return "journal_find_all";
  }

  protected get findByIdKey(): string {
    return "journal_find_by_id";
  }

  protected get findPageKey(): string {
    return "journal_find_page";
  }

  protected get handleCommandKey(): string {
    return "journal_handle_command";
  }

  protected loadIncluded(models: Journal[]): Promise<Map<string, ReadModel>> {
    throw new Error("Method not implemented.");
  }

  protected convert(input: Record<string, unknown>): Journal {
    return new Journal({
      id: input.id as string,
      permission: input.permission as Permission,
      name: input.name as string,
      description: input.description as string,
      unit: input.unit as string,
      admins: input.admins as string[],
      members: input.members as string[],
    });
  }
}

export const journalApi: JournalApi = new JournalApiImpl();
