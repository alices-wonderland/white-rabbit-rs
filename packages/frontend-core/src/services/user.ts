import type { Command, Permission, Query, WriteApi, WriteModel } from "@core/services";

export class User implements WriteModel {
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

  get modelType(): string {
    return "users";
  }
}

export type Role = "User" | "Admin";

export type UserSort = "name" | "role";

export interface UserQuery extends Query {
  readonly id?: string[];
  readonly name?: [string, boolean];
  readonly role?: Role;
}

export interface UserCommandCreate extends Command<"users:create"> {
  readonly id?: string;
  readonly name: string;
  readonly role: Role;
}

export interface UserCommandUpdate extends Command<"users:update"> {
  readonly id: string;
  readonly name?: string;
  readonly role?: Role;
}

export interface UserCommandDelete extends Command<"users:delete"> {
  readonly id: string[];
}

export type UserCommand = UserCommandCreate | UserCommandUpdate | UserCommandDelete;

export const USER_API_KEY = Symbol("USER_API_KEY");

export type UserApi = WriteApi<User, UserQuery, UserCommand, UserSort>;
