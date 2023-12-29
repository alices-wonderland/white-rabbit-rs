import { Account } from "@core/services";
import type { AccountType } from "@core/services";
import sortBy from "lodash/sortBy";
import sortedUniq from "lodash/sortedUniq";
import { AbstractRow } from "@core/components/AppTable";
import type { FieldState } from "@core/types";
import get from "lodash/get";
import isEqual from "lodash/isEqual";
import { v4 as uuidv4 } from "uuid";

const EDITABLE_FIELDS = ["name", "description", "unit", "type", "tags"] as const;
type EditableField = (typeof EDITABLE_FIELDS)[number];

export class Row extends AbstractRow<Account, EditableField> {
  _name: string = "";
  _description: string = "";
  _unit: string = "";
  type: AccountType = "Asset";
  _tags: string[] = [];

  constructor(account?: Account, readonly?: boolean) {
    super(account, readonly);
    this.reset();
  }

  override get id(): string {
    return this._existing?.id ?? uuidv4();
  }

  override reset() {
    if (this._existing) {
      this.name = this._existing.name;
      this.description = this._existing.description;
      this.unit = this._existing.unit;
      this.type = this._existing.type;
      this.tags = this._existing.tags;
    } else {
      this.name = "";
      this.description = "";
      this.unit = "";
      this.type = "Asset";
      this.tags = [];
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

  get unit() {
    return this._unit;
  }

  set unit(value: string) {
    this._unit = value.trim();
  }

  get tags() {
    return this._tags;
  }

  set tags(value: string[]) {
    this._tags = sortedUniq(sortBy(value.map((tag) => tag.trim()).filter((tag) => !!tag)));
  }

  static ofAll(accounts: Account[], readonly?: boolean): Row[] {
    return sortBy(
      accounts.map((account) => new Row(account, readonly)),
      "name",
    );
  }
}
