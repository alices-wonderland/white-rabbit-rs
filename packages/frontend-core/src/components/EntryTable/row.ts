import { AbstractRow } from "@core/components/AppTable";
import type { EntryItem, EntryStateItem, EntryType } from "@core/services";
import { Entry } from "@core/services";
import { v4 as uuidv4 } from "uuid";
import sortedUniq from "lodash/sortedUniq";
import sortBy from "lodash/sortBy";
import type { FieldState } from "@core/types";
import get from "lodash/get";
import isEqual from "lodash/isEqual";
import { format } from "date-fns";
import orderBy from "lodash/orderBy";

const EDITABLE_FIELDS = ["name", "description", "date", "type", "tags"] as const;

type EditableField = (typeof EDITABLE_FIELDS)[number];

export class ParentRow extends AbstractRow<Entry, EditableField> {
  readonly id: string;
  _name: string = "";
  _description: string = "";
  type: EntryType = "Record";
  date: string = format(new Date(), "yyyy-MM-dd");
  _tags: string[] = [];
  entryState?: EntryStateItem;

  constructor(entry?: Entry, readonly?: boolean) {
    super(entry, readonly);
    this.id = entry?.id ?? uuidv4();
    this.reset();
  }

  override reset() {
    if (this._existing) {
      this.name = this._existing.name;
      this.description = this._existing.description;
      this.type = this._existing.type;
      this.date = this._existing.date;
      this.tags = this._existing.tags;
      if (
        this._existing.type === "Record" &&
        this._existing.state &&
        "type" in this._existing.state
      ) {
        this.entryState = this._existing.state as EntryStateItem;
      }
    }
  }

  override get editableFields(): readonly EditableField[] {
    return EDITABLE_FIELDS;
  }

  override getFieldState<V>(field: EditableField): FieldState<V> {
    const value = get(this, field) as V;

    if (this._existing) {
      let existing: V;
      if (field === "tags") {
        existing = sortedUniq(sortBy(get(this._existing, "tags"))) as V;
      } else {
        existing = get(this._existing, field) as V;
      }

      if (!isEqual(value, existing)) {
        return {
          state: "UPDATED",
          value: value as V,
          existing,
        };
      }
    }

    return {
      state: "NORMAL",
      value: value as V,
    };
  }

  get name() {
    return this._name;
  }

  set name(value: string) {
    this._name = value.trim();
  }

  get description() {
    return this._description;
  }

  set description(value: string) {
    this._description = value.trim();
  }

  get tags() {
    return this._tags;
  }

  set tags(value: string[]) {
    this._tags = sortedUniq(sortBy(value.map((tag) => tag.trim()).filter((tag) => !!tag)));
  }
}

const CHILD_EDITABLE_FIELDS = ["account", "amount", "price"] as const;

type ChildEditableField = (typeof CHILD_EDITABLE_FIELDS)[number];

export class ChildRow extends AbstractRow<[Entry, EntryItem], ChildEditableField> {
  readonly parentId: string;
  accountId = "";
  amount = 0;
  price = 1;
  entryState?: EntryStateItem;

  constructor(parent: Entry | string, item?: EntryItem, readonly?: boolean) {
    super(parent instanceof Entry && item ? [parent, item] : undefined, readonly);
    this.parentId = typeof parent === "string" ? parent : parent.id;
    this.reset();
  }

  override get id(): string {
    return `${this.parentId}:${uuidv4()}`;
  }

  override reset() {
    if (this._existing) {
      const [entry, item] = this._existing;
      this.accountId = item.account;
      this.amount = item.amount;
      this.price = item.price ?? 1;
      if (entry.type === "Check") {
        this.entryState = (entry.state as Record<string, EntryStateItem>)[
          item.account
        ] as EntryStateItem;
      }
    } else {
      this.accountId = "";
      this.amount = 0;
      this.price = 1;
      this.entryState = undefined;
    }
  }

  override get editableFields(): readonly ChildEditableField[] {
    return CHILD_EDITABLE_FIELDS;
  }

  override getFieldState<V>(field: ChildEditableField): FieldState<V> {
    let current = get(this, field) as V;
    if (field === "account") {
      current = this.accountId as V;
    }

    if (this._existing) {
      const [_entry, item] = this._existing;
      const existing = get(item, field);
      if (!isEqual(existing, current)) {
        return {
          state: "UPDATED",
          value: current,
          existing: existing as V,
        };
      }
    }

    return {
      state: "NORMAL",
      value: current,
    };
  }
}

export type Row = ParentRow | ChildRow;

export const createAll = (entries: Entry[], readonly?: boolean): Row[] => {
  const rows = entries.flatMap((e) => {
    const rows: Row[] = [];
    rows.push(new ParentRow(e, readonly));
    for (const item of e.items) {
      rows.push(new ChildRow(e, item, readonly));
    }
    return rows;
  });
  return orderBy(rows, ["date", "type", "name"], ["desc", "asc", "asc"]);
};
