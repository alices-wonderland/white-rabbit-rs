import { AbstractRow } from "@core/components/AppTable";
import type { EntryType } from "@core/services";
import { Entry } from "@core/services";
import { v4 as uuidv4 } from "uuid";
import sortedUniq from "lodash/sortedUniq";
import sortBy from "lodash/sortBy";
import type { FieldState } from "@core/types";
import get from "lodash/get";
import isEqual from "lodash/isEqual";
import { format } from "date-fns";

const UPDATABLE_FIELDS = ["name", "description", "date", "type", "tags"] as const;

type UpdatableField = (typeof UPDATABLE_FIELDS)[number];

export class ParentRow extends AbstractRow<Entry, UpdatableField> {
  _name: string = "";
  _description: string = "";
  type: EntryType = "Record";
  date: string = format(new Date(), "yyyy-MM-dd");
  _tags: string[] = [];

  constructor(entry?: Entry) {
    super(entry?.id ?? uuidv4(), entry);
    this.reset();
  }

  override reset() {
    if (this._existing) {
      this.name = this._existing.name;
      this.description = this._existing.description;
      this.type = this._existing.type;
      this.date = this._existing.date;
      this.tags = this._existing.tags;
    }
  }

  override get updatableFields(): readonly UpdatableField[] {
    return UPDATABLE_FIELDS;
  }

  override getFieldState<V>(field: UpdatableField): FieldState<V> {
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

export type Row = ParentRow;

export const createAll = (entries: Entry[]): Row[] => {
  return entries.map((e) => new ParentRow(e));
};
