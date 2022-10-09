import { ContainingUserQuery, IdQuery, TextQuery } from "./query";

export interface GroupQuery {
  readonly __containingUser?: ContainingUserQuery;
  readonly id?: IdQuery;
  readonly name?: TextQuery;
  readonly description?: string;
  readonly admins?: string[];
  readonly members?: string[];
}

export type GroupCommand =
  | {
      __type: "Create";
      targetId?: string;
      name: string;
      description: string;
      admins: Set<string>;
      members: Set<string>;
    }
  | {
      __type: "Update";
      targetId: string;
      name?: string;
      description?: string;
      admins?: Set<string>;
      members?: Set<string>;
    }
  | { __type: "Delete"; targetId: string };
