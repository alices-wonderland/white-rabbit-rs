import { Account } from "@core/services";
import type { AccountType } from "@core/services";
import { v4 as uuidv4 } from "uuid";
import sortBy from "lodash/sortBy";
import sortedUniq from "lodash/sortedUniq";
import type { EditableValue } from "@core/components/AppTable";
import isEqual from "lodash/isEqual";

export class Row {
  readonly existing?: Account;

  id: string;
  _name: string = "";
  _description: string = "";
  _unit: string = "";
  type: AccountType = "Asset";
  _tags: string[] = [];

  constructor(account?: Account) {
    this.existing = account;
    this.id = account?.id ?? uuidv4();

    if (account) {
      this.name = account.name;
      this.description = account.description;
      this.unit = account.unit;
      this.type = account.type;
      this.tags = account.tags;
    }
  }

  get nameValue(): EditableValue<string> {
    if (!this.existing) {
      return {
        state: "NEW",
        value: this.name,
      };
    } else if (!isEqual(this.existing.name, this.name)) {
      return {
        state: "UPDATED",
        value: this.name,
        existing: this.existing.name,
      };
    } else {
      return {
        state: "NORMAL",
        value: this.name,
      };
    }
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
  get descriptionValue(): EditableValue<string> {
    if (!this.existing) {
      return {
        state: "NEW",
        value: this.description,
      };
    } else if (!isEqual(this.existing.description, this.description)) {
      return {
        state: "UPDATED",
        value: this.description,
        existing: this.existing.description,
      };
    } else {
      return {
        state: "NORMAL",
        value: this.description,
      };
    }
  }

  set description(value: string) {
    this._description = value.trim();
  }

  get unit() {
    return this._unit;
  }

  get unitValue(): EditableValue<string> {
    if (!this.existing) {
      return {
        state: "NEW",
        value: this.unit,
      };
    } else if (!isEqual(this.existing.unit, this.unit)) {
      return {
        state: "UPDATED",
        value: this.unit,
        existing: this.existing.unit,
      };
    } else {
      return {
        state: "NORMAL",
        value: this.unit,
      };
    }
  }

  set unit(value: string) {
    this._unit = value.trim();
  }

  get typeValue(): EditableValue<string> {
    if (!this.existing) {
      return {
        state: "NEW",
        value: this.type,
      };
    } else if (!isEqual(this.existing.type, this.type)) {
      return {
        state: "UPDATED",
        value: this.type,
        existing: this.existing.type,
      };
    } else {
      return {
        state: "NORMAL",
        value: this.type,
      };
    }
  }

  get tagsValue(): EditableValue<string[]> {
    if (!this.existing) {
      return {
        state: "NEW",
        value: this.tags,
      };
    } else if (!isEqual(sortBy(this.existing.tags), this.tags)) {
      return {
        state: "UPDATED",
        value: this.tags,
        existing: this.existing.tags,
      };
    } else {
      return {
        state: "NORMAL",
        value: this.tags,
      };
    }
  }

  get tags() {
    return this._tags;
  }

  set tags(value: string[]) {
    this._tags = sortedUniq(sortBy(value.map((tag) => tag.trim()).filter((tag) => !!tag)));
  }

  static ofAll(accounts: Account[]): Row[] {
    return sortBy(
      accounts.map((account) => new Row(account)),
      "name",
    );
  }
}
