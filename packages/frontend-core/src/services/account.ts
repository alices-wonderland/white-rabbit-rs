import type { Command, Permission, Query, WriteApi, WriteModel } from "@core/services";

export const ACCOUNT_API_KEY = Symbol("ACCOUNT_API_KEY");

export const ACCOUNT_TYPE = "accounts";

export class Account implements WriteModel<typeof ACCOUNT_TYPE> {
  id: string;
  permission: Permission;
  journal: string;
  name: string;
  description: string;
  unit: string;
  type: AccountType;
  tags: string[];

  constructor({
    id,
    permission,
    journal,
    name,
    description,
    unit,
    type,
    tags,
  }: Omit<Account, "modelType">) {
    this.id = id;
    this.permission = permission;
    this.journal = journal;
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

export type AccountType = "Income" | "Expense" | "Asset" | "Liability" | "Equity";

export type AccountSort = "name" | "unit" | "type" | "journal";

export interface AccountQuery extends Query {
  readonly id?: string[];
  readonly journal?: string[];
  readonly name?: [string, boolean];
  readonly description?: string;
  readonly type?: AccountType;
  readonly tag?: string;
}

export interface AccountCommandCreate extends Command<`${typeof ACCOUNT_TYPE}:create`> {
  readonly id?: string;
  readonly journal: string;
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

export type AccountCommand = AccountCommandCreate | AccountCommandUpdate | AccountCommandDelete;

export type AccountApi = WriteApi<Account, AccountQuery, AccountCommand, AccountSort>;
