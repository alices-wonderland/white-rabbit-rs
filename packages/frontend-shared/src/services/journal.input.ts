import { AccessItem } from "../models";
import {
  ContainingUserQuery,
  FullTextQuery,
  IdQuery,
  TextQuery,
} from "./query";

export interface JournalQuery {
  readonly __fullText?: FullTextQuery;
  readonly __containingUser?: ContainingUserQuery;
  readonly id?: IdQuery;
  readonly name?: TextQuery;
  readonly description?: string;
  readonly unit?: TextQuery;
  readonly tag?: TextQuery;
  readonly admins?: AccessItem[];
  readonly members?: AccessItem[];
  readonly includeArchived?: boolean;
}

export type JournalCommand =
  | {
      __type: "Create";
      targetId?: string;
      name: string;
      description: string;
      unit: string;
      tags: Set<string>;
      admins: Set<AccessItem>;
      members: Set<AccessItem>;
    }
  | {
      __type: "Update";
      targetId: string;
      name?: string;
      description?: string;
      unit?: string;
      isArchived?: boolean;
      tags?: Set<string>;
      admins?: Set<AccessItem>;
      members?: Set<AccessItem>;
    }
  | { __type: "Delete"; targetId: string };
