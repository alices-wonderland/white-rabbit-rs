import type { Command, Model, Query, WriteApi } from "@core/services";

export const JOURNAL_TYPE = "journals";

export const JOURNAL_API_KEY = Symbol("JOURNAL_API_KEY");

export class Journal implements Model<typeof JOURNAL_TYPE> {
  id: string;
  name: string;
  description: string;
  unit: string;
  tags: string[];

  constructor({ id, name, description, unit, tags }: Omit<Journal, "modelType">) {
    this.id = id;
    this.name = name;
    this.description = description;
    this.unit = unit;
    this.tags = tags;
  }

  get modelType(): typeof JOURNAL_TYPE {
    return JOURNAL_TYPE;
  }
}

export type JournalSort = "name" | "unit";

export interface JournalQuery extends Query {
  readonly id?: string[];
  readonly name?: string[];
  readonly unit?: string;
  readonly fullText?: [string, string[]];
}

export interface JournalCommandCreate extends Command<`${typeof JOURNAL_TYPE}:create`> {
  readonly id?: string;
  readonly name: string;
  readonly description: string;
  readonly unit: string;
}

export interface JournalCommandUpdate extends Command<`${typeof JOURNAL_TYPE}:update`> {
  readonly id: string;
  readonly name?: string;
  readonly description?: string;
  readonly unit?: string;
}

export interface JournalCommandDelete extends Command<`${typeof JOURNAL_TYPE}:delete`> {
  readonly id: string[];
}

export type JournalCommand = JournalCommandCreate | JournalCommandUpdate | JournalCommandDelete;

export type JournalApi = WriteApi<Journal, JournalQuery, JournalCommand, JournalSort>;
