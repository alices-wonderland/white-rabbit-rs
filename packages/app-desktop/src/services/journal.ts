import type {
  FindAllArgs,
  FindPageArgs,
  JournalApi,
  JournalCommand,
  JournalQuery,
  JournalSort,
  Page,
  Permission,
  ReadModel,
} from "@core/services";
import { Journal } from "@core/services";

class JournalApiImpl implements JournalApi {
  private convert(input: Record<string, unknown>): Journal {
    return new Journal({
      id: input.id as string,
      permission: input.permission as Permission,
      name: input.name as string,
      description: input.description as string,
      unit: input.unit as string,
      admins: input.admins as string[],
      members: input.admins as string[],
    });
  }

  async findAll(args: FindAllArgs<JournalQuery, JournalSort>): Promise<[Journal[], ReadModel[]]> {
    return [[], []];
  }

  async findPage(
    args: FindPageArgs<JournalQuery, JournalSort>
  ): Promise<[Page<Journal>, ReadModel[]]> {
    return [{ hasNext: false, hasPrevious: false, items: [] }, []];
  }

  async handle(command: JournalCommand): Promise<Journal[]> {
    return [];
  }

  async findById(id: string): Promise<[Journal, ReadModel[]] | null> {
    return null;
  }
}

export const journalApi: JournalApi = new JournalApiImpl();
