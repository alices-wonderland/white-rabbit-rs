import type {
  FindAllArgs,
  FindPageArgs,
  RecordApi,
  RecordCommand,
  RecordQuery,
  RecordSort,
  Page,
  Permission,
  ReadModel,
  RecordType,
  RecordItem,
  RecordState,
} from "@core/services";
import { Record_ } from "@core/services";

class RecordApiImpl implements RecordApi {
  private convert(input: Record<string, unknown>): Record_ {
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

  async findAll(args: FindAllArgs<RecordQuery, RecordSort>): Promise<[Record_[], ReadModel[]]> {
    return [[], []];
  }

  async findPage(
    args: FindPageArgs<RecordQuery, RecordSort>
  ): Promise<[Page<Record_>, ReadModel[]]> {
    return [{ hasNext: false, hasPrevious: false, items: [] }, []];
  }

  async handle(command: RecordCommand): Promise<Record_[]> {
    return [];
  }

  async findById(id: string): Promise<[Record_, ReadModel[]] | null> {
    return null;
  }
}

export const recordApi: RecordApi = new RecordApiImpl();
