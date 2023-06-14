import type { Command, Permission, Query, WriteApi, WriteModel } from "@core/services";

export const JOURNAL_TYPE = "journals";

export const JOURNAL_API_KEY = Symbol("JOURNAL_API_KEY");

export class Journal implements WriteModel<typeof JOURNAL_TYPE> {
  id: string;
  permission: Permission;
  name: string;
  description: string;
  unit: string;
  admins: string[];
  members: string[];

  constructor({
    id,
    permission,
    name,
    description,
    unit,
    admins,
    members,
  }: Omit<Journal, "modelType">) {
    this.id = id;
    this.permission = permission;
    this.name = name;
    this.description = description;
    this.unit = unit;
    this.admins = admins;
    this.members = members;
  }

  get modelType(): typeof JOURNAL_TYPE {
    return JOURNAL_TYPE;
  }
}

export type JournalSort = "name" | "unit";

export interface JournalQuery extends Query {
  readonly id?: string[];
  readonly name?: [string, boolean];
  readonly description?: string;
  readonly unit?: string;
  readonly admins?: string[];
  readonly members?: string[];
}

export interface JournalCommandCreate extends Command<`${typeof JOURNAL_TYPE}:create`> {
  readonly id?: string;
  readonly name: string;
  readonly description: string;
  readonly unit: string;
  readonly admins: string[];
  readonly members: string[];
}

export interface JournalCommandUpdate extends Command<`${typeof JOURNAL_TYPE}:update`> {
  readonly id: string;
  readonly name?: string;
  readonly description?: string;
  readonly unit?: string;
  readonly admins?: string[];
  readonly members?: string[];
}

export interface JournalCommandDelete extends Command<`${typeof JOURNAL_TYPE}:delete`> {
  readonly id: string[];
}

export type JournalCommand = JournalCommandCreate | JournalCommandUpdate | JournalCommandDelete;

export type JournalApi = WriteApi<Journal, JournalQuery, JournalCommand, JournalSort>;
