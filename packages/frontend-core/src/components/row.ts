import {
  type Journal,
  Record_,
  type Account,
  type RecordType,
  type RecordItem,
  type RecordStateItem,
  type RecordCommandUpdate,
  type RecordCommandCreate,
} from "@core/services";
import { v4 as uuidv4 } from "uuid";
import { NULL_PLACEHOLDER } from "@core/utils";
import format from "date-fns/format";

type EditedField = [string | undefined, string | undefined];

export interface BaseRow {
  deleted?: boolean;
  record?: Record_;

  get journal(): Journal;
  get id(): string;
  get name(): string;

  get dataPath(): string[];
  get state(): RecordStateItem | undefined;

  get editedFields(): Map<string, EditedField>;

  compare(another: Row): number;
}

export class Parent implements BaseRow {
  journal: Journal;
  record?: Record_;
  isDeleted = false;
  id: string;
  name: string;

  description: string;
  type?: RecordType;
  date?: Date;
  tags?: string[];
  children: Child[] = [];

  constructor(journal: Journal, record?: Record_) {
    this.journal = journal;
    this.record = record;

    this.id = record?.id || uuidv4();
    this.name = record?.name || "";
    this.description = record?.description ?? "";
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

    return this.name.localeCompare(another.name);
  }

  get editedFields(): Map<string, EditedField> {
    const results = new Map<string, EditedField>();

    if (this.record?.name !== this.name) {
      results.set("name", [this.record?.name ?? NULL_PLACEHOLDER, this.name]);
    }

    if (this.record?.date?.valueOf() !== this.date?.valueOf()) {
      results.set("date", [
        this.record?.date?.toDateString() ?? NULL_PLACEHOLDER,
        this.date?.toDateString() ?? NULL_PLACEHOLDER,
      ]);
    }

    if (this.record?.type !== this.type) {
      results.set("type", [this.record?.type ?? NULL_PLACEHOLDER, this.type ?? NULL_PLACEHOLDER]);
    }

    for (const child of this.children) {
      for (const [k, v] of child.editedFields) {
        results.set(`${child.id}::${k}`, v);
      }
    }

    return results;
  }

  get isEdited() {
    return (
      this.isDeleted ||
      this.editedFields.size > 0 ||
      this.children.some((child) => child.isDeleted !== false)
    );
  }

  generateCommand(): RecordCommandCreate | RecordCommandUpdate | undefined {
    const children = this.children
      .map((child) => child.generateModel())
      .filter((item): item is RecordItem => !!item);

    if (this.record) {
      return {
        commandType: "records:update",
        id: this.id,
        name: this.name,
        description: this.description,
        type: this.type,
        date: this.date && format(this.date, "yyyy-MM-dd"),
        tags: this.tags,
        items: children,
      };
    } else if (this.type && this.date) {
      return {
        commandType: "records:create",
        id: this.id,
        journal: this.journal.id,
        name: this.name,
        description: this.description,
        type: this.type,
        date: format(this.date, "yyyy-MM-dd"),
        tags: this.tags ?? [],
        items: children,
      };
    }
  }
}

export class Child implements BaseRow {
  _isDeleted = false;

  parent: Parent;
  recordItem?: RecordItem;

  account?: Account;
  amount?: number;
  price?: number;

  constructor(parent: Parent, recordItem?: RecordItem, account?: Account) {
    this.parent = parent;
    this.account = account;
    this.recordItem = recordItem;
    this.amount = recordItem?.amount;
    this.price = recordItem?.price;
  }

  get dataPath(): string[] {
    return [this.parent.id, this.account?.id ?? NULL_PLACEHOLDER];
  }

  get id(): string {
    return this.dataPath.join("::");
  }

  get journal(): Journal {
    return this.parent.journal;
  }

  get name(): string {
    return this.account?.name ?? "";
  }

  get record(): Record_ | undefined {
    return this.parent.record;
  }

  get isDeleted(): boolean | undefined {
    if (this.parent.isDeleted) {
      return undefined;
    }
    return this._isDeleted;
  }

  set isDeleted(value: boolean) {
    this._isDeleted = value;
  }

  get state(): RecordStateItem | undefined {
    return this.record?.type === "Check" && this.account
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

    return (this.account?.type ?? "").localeCompare(another.account?.type ?? "");
  }

  get editedFields(): Map<string, EditedField> {
    const results = new Map<string, EditedField>();

    if (this.recordItem?.account !== this.account?.id) {
      results.set("name", [this.recordItem?.account, this.account?.id]);
    }

    if (this.recordItem?.amount !== this.amount) {
      results.set("amount", [
        this.recordItem?.amount.toString() ?? NULL_PLACEHOLDER,
        this.amount?.toString() ?? NULL_PLACEHOLDER,
      ]);
    }

    if (this.recordItem?.price !== this.price) {
      results.set("price", [
        this.recordItem?.price?.toFixed(2) ?? NULL_PLACEHOLDER,
        this.price?.toFixed(2) ?? NULL_PLACEHOLDER,
      ]);
    }

    return results;
  }

  get isEdited() {
    return this._isDeleted || this.editedFields.size > 0;
  }

  generateModel(): RecordItem | undefined {
    if (this.account && this.amount) {
      return {
        account: this.account?.id,
        amount: this.amount,
        price: typeof this.price === "number" ? this.price : undefined,
      };
    } else {
      return undefined;
    }
  }
}

export type Row = Parent | Child;
