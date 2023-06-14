import {
  Journal,
  type JournalApi,
  type JournalCommand,
  type JournalQuery,
  type JournalSort,
  type Permission,
  type ReadModel,
  type UserQuery,
} from "@core/services";
import { AbstractWriteApi } from "./api";
import { userApi } from "./user";
import { toMap } from "@core/utils";

class JournalApiImpl extends AbstractWriteApi<Journal, JournalQuery, JournalCommand, JournalSort> {
  protected override get findAllKey(): string {
    return "journal_find_all";
  }

  protected override get findByIdKey(): string {
    return "journal_find_by_id";
  }

  protected override get findPageKey(): string {
    return "journal_find_page";
  }

  protected override get handleCommandKey(): string {
    return "journal_handle_command";
  }

  protected override async loadIncluded(models: Journal[]): Promise<Map<string, ReadModel>> {
    const userIds = new Set(models.flatMap((model) => [...model.admins, ...model.members]));
    const users = await userApi.findAll({ query: { id: [...userIds] } as UserQuery });
    return toMap(users[0]);
  }

  protected override convert(input: Record<string, unknown>): Journal {
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
