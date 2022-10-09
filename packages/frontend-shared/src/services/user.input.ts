import { Role } from "../models";
import { IdQuery, TextQuery } from "./query";

export interface UserQuery {
  readonly id?: IdQuery;
  readonly name?: TextQuery;
  readonly role?: Role;
  readonly authIdProviders?: Set<string>;
}

export type UserCommand =
  | {
      __type: "Create";
      targetId?: string;
      name: string;
      role: Role;
      authIds: Set<[string, string]>;
    }
  | {
      __type: "Update";
      targetId: string;
      name?: string;
      role?: Role;
      authIds?: Set<[string, string]>;
    }
  | { __type: "Delete"; targetId: string };
