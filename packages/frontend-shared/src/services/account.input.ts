import { AccountType, Strategy } from "../models";
import { FullTextQuery, IdQuery, TextQuery } from "./query";

export interface AccountQuery {
  readonly __fullText?: FullTextQuery;
  readonly id?: IdQuery;
  readonly journal?: string;
  readonly name?: TextQuery;
  readonly description?: string;
  readonly type?: AccountType;
  readonly strategy?: Strategy;
  readonly unit?: string;
  readonly tag?: TextQuery;
  readonly includeArchived?: boolean;
}

export type AccountCommand =
  | {
      __type: "Create";
      targetId?: string;
      journalId: string;
      name: string;
      description: string;
      type: AccountType;
      strategy: Strategy;
      unit: string;
      tags: Set<string>;
    }
  | {
      __type: "Update";
      targetId: string;
      name?: string;
      description?: string;
      type?: AccountType;
      strategy?: Strategy;
      unit?: string;
      tags?: Set<string>;
      isArchived?: boolean;
    }
  | { __type: "Delete"; targetId: string };
