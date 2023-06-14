import {
  User,
  type Permission,
  type Role,
  type UserApi,
  type UserCommand,
  type UserQuery,
  type UserSort,
} from "@core/services";
import { AbstractWriteApi } from "./api";

class UserApiImpl extends AbstractWriteApi<User, UserQuery, UserCommand, UserSort> {
  protected override get findAllKey(): string {
    return "user_find_all";
  }

  protected override get findByIdKey(): string {
    return "user_find_by_id";
  }

  protected override get findPageKey(): string {
    return "user_find_page";
  }

  protected override get handleCommandKey(): string {
    return "user_handle_command";
  }

  protected override convert(input: Record<string, unknown>): User {
    return new User({
      id: input.id as string,
      permission: input.permission as Permission,
      name: input.name as string,
      role: input.role as Role,
    });
  }
}

export const userApi: UserApi = new UserApiImpl();
