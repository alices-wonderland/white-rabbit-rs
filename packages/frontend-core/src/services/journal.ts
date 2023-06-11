import type { Command, Permission, WriteApi, WriteModel } from "@core/services";

export class Journal implements WriteModel {
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

  get modelType(): string {
    return "journals";
  }
}

export type JournalSort = "name" | "unit";

export interface JournalQuery {
  readonly id?: string[];
  readonly name?: [string, boolean];
  readonly description?: string;
  readonly unit?: string;
  readonly admins?: string[];
  readonly members?: string[];
}

export interface JournalCommandCreate extends Command<"journals:create"> {
  readonly id?: string;
  readonly name: string;
  readonly description: string;
  readonly unit: string;
  readonly admins: string[];
  readonly members: string[];
}

export interface JournalCommandUpdate extends Command<"journals:update"> {
  readonly id: string;
  readonly name?: string;
  readonly description?: string;
  readonly unit?: string;
  readonly admins?: string[];
  readonly members?: string[];
}

export interface JournalCommandDelete extends Command<"journals:delete"> {
  readonly id: string[];
}

export type JournalCommand = JournalCommandCreate | JournalCommandUpdate | JournalCommandDelete;

export const JOURNAL_API_KEY = Symbol("JOURNAL_API_KEY");

export type JournalApi = WriteApi<Journal, JournalQuery, JournalCommand, JournalSort>;
