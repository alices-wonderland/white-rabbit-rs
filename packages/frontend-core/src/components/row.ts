import type {
  Journal,
  Record_,
  Account,
  RecordType,
  RecordItem,
  RecordStateItem,
} from "@core/services";
import { v4 as uuidv4 } from "uuid";

export interface BaseRow {
  deleted?: boolean;
  record?: Record_;

  get journal(): Journal;
  get id(): string;
  get name(): string;

  get dataPath(): string[];
  get state(): RecordStateItem | undefined;

  compare(another: Row): number;
}

export class Parent implements BaseRow {
  journal: Journal;
  record?: Record_;
  deleted = false;
  id: string;
  name: string;

  description?: string;
  type?: RecordType;
  date?: Date;
  tags?: string[];
  children: Child[] = [];

  constructor(journal: Journal, record?: Record_) {
    this.journal = journal;
    this.record = record;

    this.id = record?.id || uuidv4();
    this.name = record?.name || "";
    this.description = record?.description;
    this.type = record?.type;
    this.date = record?.date;
    this.tags = record?.tags;
  }

  get dataPath(): string[] {
    return [this.id];
  }

  get state(): RecordStateItem | undefined {
    return this.record?.type === "Record" ? (this.record.state as RecordStateItem) : undefined;
  }

  compare(another: Row): number {
    if (another instanceof Child) {
      return -1;
    }

    const dateCompare = (this.date?.valueOf() ?? 0) - (another.date?.valueOf() ?? 0);
    if (dateCompare !== 0) {
      return dateCompare;
    }

    const typeCompare = (this.type ?? "").localeCompare(another.type ?? "");
    if (typeCompare !== 0) {
      return typeCompare;
    }

    return (this.name ?? "").localeCompare(another.name ?? "");
  }
}

export class Child implements BaseRow {
  _deleted = false;

  parent: Parent;
  account: Account;
  amount: number;
  price?: number;

  constructor(parent: Parent, recordItem: RecordItem, account: Account) {
    this.parent = parent;
    this.account = account;
    this.amount = recordItem.amount;
    this.price = recordItem.price;
  }

  get dataPath(): string[] {
    return [this.parent.id, this.account.id];
  }

  get id(): string {
    return this.dataPath.join("::");
  }

  get journal(): Journal {
    return this.parent.journal;
  }

  get name(): string {
    return this.account.name;
  }

  get record(): Record_ | undefined {
    return this.parent.record;
  }

  get deleted(): boolean | undefined {
    if (this.parent.deleted) {
      return undefined;
    }
    return this._deleted;
  }

  set deleted(value: boolean) {
    this._deleted = value;
  }

  get state(): RecordStateItem | undefined {
    return this.record?.type === "Check"
      ? (this.record.state as Record<string, RecordStateItem>)[this.account.id]
      : undefined;
  }

  compare(another: Row): number {
    if (another instanceof Parent) {
      return 1;
    }

    const parentCompare = this.parent.compare(another.parent);
    if (parentCompare !== 0) {
      return parentCompare;
    }

    return this.account.type.localeCompare(another.account.type);
  }
}

export type Row = Parent | Child;
