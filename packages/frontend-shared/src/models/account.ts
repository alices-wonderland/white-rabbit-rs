export interface Account {
  readonly id: string;
  readonly journalId: string;
  readonly name: string;
  readonly description: string;
  readonly type: AccountType;
  readonly strategy: Strategy;
  readonly unit: string;
  readonly isArchived: boolean;
  readonly tags: Set<string>;
}

export enum AccountType {
  INCOME = "Income",
  EXPENSE = "Expense",
  ASSET = "Asset",
  LIABILITY = "Liability",
  EQUITY = "Equity",
}

export enum Strategy {
  FIFO = "Fifo",
  AVERAGE = "Average",
}
