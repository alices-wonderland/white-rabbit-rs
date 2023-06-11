import type {
  Permission,
  ReadModel,
  Role,
  UserApi,
  UserCommand,
  UserQuery,
  UserSort,
} from "@core/services";
import { User } from "@core/services";
import { AbstractWriteApi } from "@desktop/services/api";

class UserApiImpl extends AbstractWriteApi<User, UserQuery, UserCommand, UserSort> {
  protected get findAllKey(): string {
    return "user_find_all";
  }

  protected get findByIdKey(): string {
    return "user_find_by_id";
  }

  protected get findPageKey(): string {
    return "user_find_page";
  }

  protected get handleCommandKey(): string {
    return "user_handle_command";
  }

  protected loadIncluded(models: User[]): Promise<Map<string, ReadModel>> {
    throw new Error("Method not implemented.");
  }

  protected convert(input: Record<string, unknown>): User {
    return new User({
      id: input.id as string,
      permission: input.permission as Permission,
      name: input.name as string,
      role: input.role as Role,
    });
  }
}

export const userApi: UserApi = new UserApiImpl();
