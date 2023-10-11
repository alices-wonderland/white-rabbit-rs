import {
  type Journal,
  Entry,
  type Account,
  type EntryType,
  type EntryItem,
  type EntryStateItem,
  type EntryCommandUpdate,
  type EntryCommandCreate,
} from "@core/services";
import { v4 as uuidv4 } from "uuid";
import { NULL_PLACEHOLDER } from "@core/utils";
import format from "date-fns/format";

type EditedField = [string | undefined, string | undefined];

export interface BaseRow {
  deleted?: boolean;
  entry?: Entry;

  get journal(): Journal;
  get id(): string;
  get name(): string;

  get dataPath(): string[];
  get state(): EntryStateItem | undefined;

  get editedFields(): Map<string, EditedField>;

  compare(another: Row): number;
}

interface ParentArgs {
  readonly journal: Journal;
  readonly entry?: Entry;
  readonly isDeleted?: boolean;
  readonly id?: string;
  readonly name?: string;
  readonly description?: string;
  readonly type?: EntryType;
  readonly date?: Date;
  readonly tags?: string[];
  readonly children?: Child[];
}

export class Parent implements BaseRow {
  journal: Journal;
  entry?: Entry;
  isDeleted = false;
  id: string;
  name: string;

  description: string;
  type?: EntryType;
  date?: Date;
  tags?: string[];
  children: Child[] = [];

  constructor(args: ParentArgs) {
    this.journal = args.journal;
    this.entry = args.entry;
    this.isDeleted = args.isDeleted ?? false;
    this.id = args.entry?.id ?? args.id ?? uuidv4();
    this.name = args.entry?.name ?? args.name ?? "";
    this.description = args.entry?.description ?? args.description ?? "";
    this.type = args.entry?.type ?? args.type;
    this.date = args.entry?.date ?? args.date;
    this.tags = args.entry?.tags ?? args.tags;
    this.children = args.children ?? [];
  }

  get dataPath(): string[] {
    return [this.id];
  }

  get state(): EntryStateItem | undefined {
    return this.entry?.type === "Record" ? (this.entry.state as EntryStateItem) : undefined;
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

    if (this.entry?.name !== this.name) {
      results.set("name", [this.entry?.name ?? NULL_PLACEHOLDER, this.name]);
    }

    if (this.entry?.date?.valueOf() !== this.date?.valueOf()) {
      results.set("date", [
        this.entry?.date?.toDateString() ?? NULL_PLACEHOLDER,
        this.date?.toDateString() ?? NULL_PLACEHOLDER,
      ]);
    }

    if (this.entry?.type !== this.type) {
      results.set("type", [this.entry?.type ?? NULL_PLACEHOLDER, this.type ?? NULL_PLACEHOLDER]);
    }

    for (const accountId of [
      ...this.children.map((c) => c.account?.id).filter((id): id is string => !!id),
      ...(this.entry?.items?.map((i) => i.account) ?? []),
    ]) {
      const oldItem: EntryItem | undefined = this.entry?.items?.find(
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

  generateCommand(): EntryCommandCreate | EntryCommandUpdate | undefined {
    const children = this.children
      .map((child) => child.generateModel())
      .filter((item): item is EntryItem => !!item);

    if (this.entry) {
      return {
        commandType: "entries:update",
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
        commandType: "entries:create",
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
      entry: undefined,
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
  readonly entryItem?: EntryItem;
  readonly account?: Account;
  readonly amount?: number;
  readonly price?: number;
}

export class Child implements BaseRow {
  _isDeleted = false;

  id: string;
  parent: Parent;
  entryItem?: EntryItem;

  account?: Account;
  amount?: number;
  price?: number;

  constructor(args: ChildArgs) {
    this._isDeleted = args.isDeleted ?? false;
    this.id = args.id ?? args.account?.id ?? uuidv4();
    this.parent = args.parent;
    this.entryItem = args.entryItem;
    this.account = args.account;
    this.amount = args.entryItem?.amount ?? args.amount;
    this.price = args.entryItem?.price ?? args.price;
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

  get entry(): Entry | undefined {
    return this.parent.entry;
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

  get state(): EntryStateItem | undefined {
    return this.entry?.type === "Check" && this.account
      ? (this.entry.state as Record<string, EntryStateItem>)[this.account.id]
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

    if (this.entryItem?.account !== this.account?.id) {
      results.set("name", [this.entryItem?.account, this.account?.id]);
    }

    if (this.entryItem?.amount !== this.amount) {
      results.set("amount", [
        this.entryItem?.amount.toString() ?? NULL_PLACEHOLDER,
        this.amount?.toString() ?? NULL_PLACEHOLDER,
      ]);
    }

    const oldPrice = this.entryItem?.price?.toFixed(2) ?? NULL_PLACEHOLDER;
    const newPrice = this.price?.toFixed(2) ?? NULL_PLACEHOLDER;
    if (oldPrice !== newPrice) {
      results.set("price", [oldPrice, newPrice]);
    }

    return results;
  }

  get isEdited() {
    return this._isDeleted || this.editedFields.size > 0;
  }

  generateModel(): EntryItem | undefined {
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
