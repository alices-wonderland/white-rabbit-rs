import type {
  FindAllArgs,
  FindPageArgs,
  Page,
  Permission,
  ReadModel,
  Role,
  UserApi,
  UserCommand,
  UserQuery,
  UserSort,
} from "@core/services";
import { User } from "@core/services";
import { invoke } from "@tauri-apps/api/tauri";

class UserApiImpl implements UserApi {
  private convert(input: Record<string, unknown>): User {
    return new User({
      id: input.id as string,
      permission: input.permission as Permission,
      name: input.name as string,
      role: input.role as Role,
    });
  }

  async findAll({ query, sort }: FindAllArgs<UserQuery, UserSort>): Promise<[User[], ReadModel[]]> {
    const models = await invoke<Record<string, unknown>[]>("user_find_all", {
      query,
      sort,
    });
    return [models.map(this.convert), []];
  }

  async findById(id: string): Promise<[User, ReadModel[]] | null> {
    const model = await invoke<Record<string, unknown> | null>("user_find_by_id", {
      id,
    });

    if (model) {
      return [this.convert(model), []];
    } else {
      return null;
    }
  }

  async findPage({
    query,
    sort,
    after,
    before,
    size,
  }: FindPageArgs<UserQuery, UserSort>): Promise<[Page<User>, ReadModel[]]> {
    const page = await invoke<Record<string, unknown>>("user_find_page", {
      query: query,
      sort: sort,
      after: after,
      before: before,
      size: size,
    });

    const items = page.items as Record<string, unknown>[];
    return [
      {
        hasPrevious: page.hasPrevious as boolean,
        hasNext: page.hasNext as boolean,
        items: items.map(this.convert),
      },
      [],
    ];
  }

  async handle(command: UserCommand): Promise<User[]> {
    console.log("Command: ", command);
    return [];
  }
}

export const userApi: UserApi = new UserApiImpl();
