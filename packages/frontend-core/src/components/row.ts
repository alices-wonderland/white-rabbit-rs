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

interface ParentArgs {
  readonly journal: Journal;
  readonly record?: Record_;
  readonly isDeleted?: boolean;
  readonly id?: string;
  readonly name?: string;
  readonly description?: string;
  readonly type?: RecordType;
  readonly date?: Date;
  readonly tags?: string[];
  readonly children?: Child[];
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

  constructor(args: ParentArgs) {
    this.journal = args.journal;
    this.record = args.record;
    this.isDeleted = args.isDeleted ?? false;
    this.id = (args.record?.id ?? args.id) || uuidv4();
    this.name = (args.record?.name ?? args.name) || "";
    this.description = (args.record?.description ?? args.description) || "";
    this.type = args.record?.type ?? args.type;
    this.date = args.record?.date ?? args.date;
    this.tags = args.record?.tags ?? args.tags;
    this.children = args.children ?? [];
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

  // eslint-disable-next-line sonarjs/cognitive-complexity
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

    for (const accountId of [
      ...this.children.map((c) => c.account?.id).filter((id): id is string => !!id),
      ...(this.record?.items?.map((i) => i.account) ?? []),
    ]) {
      const oldItem: RecordItem | undefined = this.record?.items?.find(
        (i) => i.account === accountId,
      );
      const newItem: Child | undefined = this.children.find((c) => c.account?.id === accountId);
      if (oldItem && !newItem) {
        results.set(oldItem.account, ["true", undefined]);
      } else if (!oldItem && newItem) {
        results.set(newItem.id, [undefined, "true"]);
      } else if (newItem && oldItem) {
        for (const [k, v] of newItem.editedFields) {
          results.set(`${newItem.id}::${k}`, v);
        }
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

  clone(): Parent {
    const parent = new Parent({
      journal: this.journal,
      record: undefined,
      isDeleted: false,
      id: uuidv4(),
      name: this.name,
      description: this.description,
      type: this.type,
      date: this.date,
      tags: this.tags,
      children: [],
    });
    for (const child of this.children) {
      parent.children.push(child.clone(parent));
    }
    return parent;
  }
}

interface ChildArgs {
  readonly isDeleted?: boolean;
  readonly id?: string;
  readonly parent: Parent;
  readonly recordItem?: RecordItem;
  readonly account?: Account;
  readonly amount?: number;
  readonly price?: number;
}

export class Child implements BaseRow {
  _isDeleted = false;

  id: string;
  parent: Parent;
  recordItem?: RecordItem;

  account?: Account;
  amount?: number;
  price?: number;

  constructor(args: ChildArgs) {
    this._isDeleted = args.isDeleted ?? false;
    this.id = args.id ?? args.account?.id ?? uuidv4();
    this.parent = args.parent;
    this.recordItem = args.recordItem;
    this.account = args.account;
    this.amount = args.recordItem?.amount ?? args.amount;
    this.price = args.recordItem?.price ?? args.price;
  }

  get dataPath(): string[] {
    return [this.parent.id, this.id];
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
    if (this.account && typeof this.amount === "number") {
      return {
        account: this.account?.id,
        amount: this.amount,
        price: typeof this.price === "number" ? this.price : undefined,
      };
    } else {
      return undefined;
    }
  }

  clone(parent?: Parent): Child {
    return new Child({
      isDeleted: false,
      id: uuidv4(),
      parent: parent ?? this.parent,
      account: this.account,
      amount: this.amount,
      price: this.price,
    });
  }
}

export type Row = Parent | Child;
