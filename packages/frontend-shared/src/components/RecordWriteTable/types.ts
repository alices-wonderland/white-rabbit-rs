import { RowNode } from "@ag-grid-community/core";
import {
  AccountNotInJournalError,
  DuplicateAccountsInRecord,
  SharedError,
} from "@shared/error";
import {
  RecordType,
  Journal,
  Record_,
  Account,
  RecordItem,
  AccountType,
} from "@shared/models";
import groupBy from "lodash/groupBy";

export const DATE_FORMAT = "yyyy-MM-dd";

export enum RowType {
  RECORD,
  ITEM,
}

export type Row = RecordRow | RecordItemRow;

export class RecordRowData {
  name: string;
  type: RecordType;
  journal?: Journal;
  date: Date;
  tags: Set<string>;
  description: string;

  static KEYS: Array<keyof RecordRowData> = [
    "name",
    "type",
    "journal",
    "date",
    "tags",
    "description",
  ];

  constructor(record: Record_, journal?: Journal) {
    this.name = record.name;
    this.type = record.type;
    this.journal = journal;
    this.date = record.date;
    this.tags = record.tags;
    this.description = record.description;
  }
}

export class RecordRow {
  hierarchy: [string];
  data: RecordRowData;
  snapshot: RecordRowData;
  isDeleted: boolean;

  constructor(record: Record_, journal: Journal) {
    this.hierarchy = [record.id];
    this.data = new RecordRowData(record, journal);
    this.snapshot = new RecordRowData(record, journal);
    this.isDeleted = false;
  }

  get rowType(): RowType {
    return RowType.RECORD;
  }

  get diffResults(): Map<keyof RecordRowData, [unknown, unknown]> {
    const result = new Map();
    for (const field of RecordRowData.KEYS) {
      const oldValue = this.snapshot[field];
      const newValue = this.data[field];
      if (oldValue !== newValue) {
        result.set(field, [oldValue, newValue]);
      }
    }
    return result;
  }

  get isEditable(): boolean {
    return !this.isDeleted;
  }

  errors(): Map<string, SharedError[]> | undefined {
    return undefined;
  }
}

export class RecordItemRowData {
  account?: Account;
  amount: number;
  price?: number;

  static KEYS: Array<keyof RecordItemRowData> = ["account", "amount", "price"];

  constructor(item: RecordItem, account?: Account) {
    this.account = account;
    this.amount = item.amount;
    this.price = item.price;
  }

  get type(): AccountType | undefined {
    return this.account?.type;
  }
}

export class RecordItemRow {
  hierarchy: [string, string];
  data: RecordItemRowData;
  snapshot: RecordItemRowData;
  isDeleted: boolean;
  isParentDeleted: boolean;

  constructor(item: RecordItem, record: Record_, account?: Account) {
    this.hierarchy = [record.id, item.accountId];
    this.data = new RecordItemRowData(item, account);
    this.snapshot = new RecordItemRowData(item, account);
    this.isDeleted = false;
    this.isParentDeleted = false;
  }

  get rowType(): RowType {
    return RowType.ITEM;
  }

  get diffResults(): Map<string, [unknown, unknown]> {
    const result = new Map();
    for (const field of RecordItemRowData.KEYS) {
      const oldValue = this.snapshot[field];
      const newValue = this.data[field];
      if (oldValue !== newValue) {
        result.set(field, [oldValue, newValue]);
      }
    }
    return result;
  }

  get isEditable(): boolean {
    return !this.isDeleted && !this.isParentDeleted;
  }

  errors(node: RowNode<Row>): Map<string, SharedError[]> | undefined {
    const result = new Map();
    const parent = node.parent as unknown as RowNode<RecordRow>;

    const journal = parent.data?.data.journal;
    if (journal?.id !== this.data.account?.journalId) {
      const errors: SharedError[] = result.get("account") ?? [];
      if (errors.length === 0) {
        result.set("account", errors);
      }

      errors.push(new AccountNotInJournalError(journal, this.data.account));
    }

    const rows =
      (parent.childrenAfterGroup as unknown as Array<RowNode<RecordItemRow>>) ??
      [];
    const accountItems = groupBy(
      rows.map((row) => [row.id, row.data?.data.account?.id]),
      (pair) => pair[1]
    );
    for (const [accountId, pairs] of Object.entries(accountItems)) {
      const rowIds = pairs.map((pair) => pair[0]);
      if (rowIds.includes(node.id) && rowIds.length > 1) {
        const errors: SharedError[] = result.get("account") ?? [];
        if (errors.length === 0) {
          result.set("account", errors);
        }
        errors.push(
          new DuplicateAccountsInRecord(this.hierarchy[0], accountId)
        );
      }
    }

    return result.size > 0 ? result : undefined;
  }
}
