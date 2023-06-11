import type { Command, Permission, Query, WriteApi, WriteModel } from "@core/services";

export class Record_ implements WriteModel {
  id: string;
  permission: Permission;
  journal: string;
  name: string;
  description: string;
  type: RecordType;
  date: Date;
  tags: string[];
  items: RecordItem[];
  state: RecordState;

  constructor({
    id,
    permission,
    journal,
    name,
    description,
    type,
    date,
    tags,
    items,
    state,
  }: Omit<Record_, "modelType">) {
    this.id = id;
    this.permission = permission;
    this.journal = journal;
    this.name = name;
    this.description = description;
    this.type = type;
    this.date = date;
    this.tags = tags;
    this.items = items;
    this.state = state;
  }

  get modelType(): string {
    return "records";
  }
}

export type RecordType = "Record" | "Check";

export interface RecordItem {
  account: string;
  amount: number;
  price?: number;
}

export type RecordState = RecordStateItem | Record<string, RecordStateItem>;

export type RecordStateItem =
  | { readonly type: "Valid"; readonly value: number }
  | { readonly type: "Invalid"; readonly value: [number, number] };

export type RecordSort = "name" | "journal" | "type" | "date";

export interface RecordQuery extends Query {
  readonly id?: string[];
  readonly type?: RecordType;
  readonly journal?: string[];
  readonly account?: string[];
  readonly start?: Date;
  readonly end?: Date;
}

export interface RecordCommandCreate extends Command<"records:create"> {
  readonly id?: string;
  readonly journal: string;
  readonly name: string;
  readonly description: string;
  readonly type: RecordType;
  readonly date: Date;
  readonly tags: string[];
  readonly items: RecordItem[];
}

export interface RecordCommandUpdate extends Command<"records:update"> {
  readonly id: string;
  readonly name?: string;
  readonly description?: string;
  readonly type?: RecordType;
  readonly date?: Date;
  readonly tags?: string[];
  readonly items?: RecordItem[];
}

export interface RecordCommandDelete extends Command<"records:delete"> {
  readonly id: string[];
}

export type RecordCommand = RecordCommandCreate | RecordCommandUpdate | RecordCommandDelete;

export const RECORD_API_KEY = Symbol("RECORD_API_KEY");

export type RecordApi = WriteApi<Record_, RecordQuery, RecordCommand, RecordSort>;
