import type { Command, Permission, WriteApi, WriteModel } from "@core/services";

export class Account implements WriteModel {
  id: string;
  permission: Permission;
  name: string;
  description: string;
  unit: string;
  type: AccountType;
  tags: string[];
  journal: string;

  constructor({
    id,
    permission,
    name,
    description,
    unit,
    type,
    tags,
    journal,
  }: Omit<Account, "modelType">) {
    this.id = id;
    this.permission = permission;
    this.name = name;
    this.description = description;
    this.unit = unit;
    this.type = type;
    this.tags = tags;
    this.journal = journal;
  }

  get modelType(): string {
    return "accounts";
  }
}

export type AccountType = "Income" | "Expense" | "Asset" | "Liability" | "Equity";

export type AccountSort = "name" | "unit" | "type" | "journal";

export interface AccountQuery {
  readonly id?: string[];
  readonly name?: [string, boolean];
  readonly description?: string;
  readonly type?: AccountType;
  readonly tag?: string;
  readonly journal?: string[];
}

export interface AccountCommandCreate extends Command<"accounts:create"> {
  readonly id?: string;
  readonly name: string;
  readonly description: string;
  readonly unit: string;
  readonly type: AccountType;
  readonly tags: string[];
  readonly journal: string;
}

export interface AccountCommandUpdate extends Command<"accounts:update"> {
  readonly id: string;
  readonly name?: string;
  readonly description?: string;
  readonly unit?: string;
  readonly type?: AccountType;
  readonly tags?: string[];
  readonly journal?: string;
}

export interface AccountCommandDelete extends Command<"accounts:delete"> {
  readonly id: string[];
}

export type AccountCommand = AccountCommandCreate | AccountCommandUpdate | AccountCommandDelete;

export const ACCOUNT_API_KEY = Symbol("ACCOUNT_API_KEY");

export type AccountApi = WriteApi<Account, AccountQuery, AccountCommand, AccountSort>;
