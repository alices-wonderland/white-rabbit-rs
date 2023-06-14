import type { Command, Permission, Query, WriteApi, WriteModel } from "@core/services";

export const USER_TYPE = "users";

export const USER_API_KEY = Symbol("USER_API_KEY");

export class User implements WriteModel<typeof USER_TYPE> {
  id: string;
  permission: Permission;
  name: string;
  role: Role;

  constructor({ id, permission, name, role }: Omit<User, "modelType">) {
    this.id = id;
    this.permission = permission;
    this.name = name;
    this.role = role;
  }

  get modelType(): typeof USER_TYPE {
    return USER_TYPE;
  }
}

export type Role = "User" | "Admin";

export type UserSort = "name" | "role";

export interface UserQuery extends Query {
  readonly id?: string[];
  readonly name?: [string, boolean];
  readonly role?: Role;
}

export interface UserCommandCreate extends Command<`${typeof USER_TYPE}:create`> {
  readonly id?: string;
  readonly name: string;
  readonly role: Role;
}

export interface UserCommandUpdate extends Command<`${typeof USER_TYPE}:update`> {
  readonly id: string;
  readonly name?: string;
  readonly role?: Role;
}

export interface UserCommandDelete extends Command<`${typeof USER_TYPE}:delete`> {
  readonly id: string[];
}

export type UserCommand = UserCommandCreate | UserCommandUpdate | UserCommandDelete;

export type UserApi = WriteApi<User, UserQuery, UserCommand, UserSort>;
