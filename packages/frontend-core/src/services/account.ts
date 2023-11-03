import type { Command, Model, Query, WriteApi } from "@core/services";

export const ACCOUNT_API_KEY = Symbol("ACCOUNT_API_KEY");

export const ACCOUNT_TYPE = "accounts";

export class Account implements Model<typeof ACCOUNT_TYPE> {
  id: string;
  journalId: string;
  name: string;
  description: string;
  unit: string;
  type: AccountType;
  tags: string[];

  constructor({ id, journalId, name, description, unit, type, tags }: Omit<Account, "modelType">) {
    this.id = id;
    this.journalId = journalId;
    this.name = name;
    this.description = description;
    this.unit = unit;
    this.type = type;
    this.tags = tags;
  }

  get modelType(): typeof ACCOUNT_TYPE {
    return ACCOUNT_TYPE;
  }
}

export const ACCOUNT_TYPES = ["Income", "Expense", "Asset", "Liability", "Equity"] as const;

export type AccountType = (typeof ACCOUNT_TYPES)[number];

export type AccountSort = "name" | "unit" | "type" | "journal";

export interface AccountQuery extends Query {
  readonly id?: string[];
  readonly journalId?: string[];
  readonly name?: string;
  readonly unit?: string;
  readonly type?: AccountType;
  readonly fullText?: [string, string[]];
}

export interface AccountCommandCreate extends Command<`${typeof ACCOUNT_TYPE}:create`> {
  readonly id?: string;
  readonly journalId: string;
  readonly name: string;
  readonly description: string;
  readonly unit: string;
  readonly type: AccountType;
  readonly tags: string[];
}

export interface AccountCommandUpdate extends Command<`${typeof ACCOUNT_TYPE}:update`> {
  readonly id: string;
  readonly name?: string;
  readonly description?: string;
  readonly unit?: string;
  readonly type?: AccountType;
  readonly tags?: string[];
}

export interface AccountCommandDelete extends Command<`${typeof ACCOUNT_TYPE}:delete`> {
  readonly id: string[];
}

export interface AccountCommandBatch extends Command<`${typeof ACCOUNT_TYPE}:batch`> {
  readonly create?: Omit<AccountCommandCreate, "commandType">[];
  readonly update?: Omit<AccountCommandUpdate, "commandType">[];
  readonly delete?: string[];
}

export type AccountCommand =
  | AccountCommandCreate
  | AccountCommandUpdate
  | AccountCommandDelete
  | AccountCommandBatch;

export type AccountApi = WriteApi<Account, AccountQuery, AccountCommand, AccountSort>;
