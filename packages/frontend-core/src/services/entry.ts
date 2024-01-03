import type { Command, Model, Query, WriteApi } from "@core/services";

export const ENTRY_TYPE = "entries";

export const ENTRY_API_KEY = Symbol("ENTRY_API_KEY");

export class Entry implements Model<typeof ENTRY_TYPE> {
  id: string;
  journalId: string;
  name: string;
  description: string;
  type: EntryType;
  date: string;
  tags: string[];
  items: EntryItem[];
  state: EntryState;

  constructor({
    id,
    journalId,
    name,
    description,
    type,
    date,
    tags,
    items,
    state,
  }: Omit<Entry, "modelType">) {
    this.id = id;
    this.journalId = journalId;
    this.name = name;
    this.description = description;
    this.type = type;
    this.date = date;
    this.tags = tags;
    this.items = items;
    this.state = state;
  }

  get modelType(): typeof ENTRY_TYPE {
    return ENTRY_TYPE;
  }
}

export const ENTRY_TYPES = ["Record", "Check"] as const;

export type EntryType = (typeof ENTRY_TYPES)[number];

export interface EntryItem {
  account: string;
  amount: number;
  price?: number;
}

export type EntryState = EntryStateItem | Record<string, EntryStateItem>;

export type EntryStateItem =
  | { readonly type: "Valid"; readonly value: number }
  | { readonly type: "Invalid"; readonly value: [number, number] };

export type EntrySort = "name" | "journal" | "type" | "date";

export interface EntryQuery extends Query {
  readonly id?: string[];
  readonly journalId?: string[];
  readonly accountId?: string[];
  readonly name?: string;
  readonly type?: EntryType;
  readonly start?: string;
  readonly end?: string;
  readonly fullText?: [string, string[]];
}

export interface EntryCommandCreate extends Command<`${typeof ENTRY_TYPE}:create`> {
  readonly id?: string;
  readonly journalId: string;
  readonly name: string;
  readonly description: string;
  readonly type: EntryType;
  readonly date: string;
  readonly tags: string[];
  readonly items: EntryItem[];
}

export interface EntryCommandUpdate extends Command<`${typeof ENTRY_TYPE}:update`> {
  readonly id: string;
  readonly name?: string;
  readonly description?: string;
  readonly type?: EntryType;
  readonly date?: string;
  readonly tags?: string[];
  readonly items?: EntryItem[];
}

export interface EntryCommandDelete extends Command<`${typeof ENTRY_TYPE}:delete`> {
  readonly id: string[];
}

export interface EntryCommandBatch extends Command<`${typeof ENTRY_TYPE}:batch`> {
  readonly create?: Omit<EntryCommandCreate, "commandType">[];
  readonly update?: Omit<EntryCommandUpdate, "commandType">[];
  readonly delete?: string[];
}

export type EntryCommand =
  | EntryCommandCreate
  | EntryCommandUpdate
  | EntryCommandDelete
  | EntryCommandBatch;

export type EntryApi = WriteApi<Entry, EntryQuery, EntryCommand, EntrySort>;
