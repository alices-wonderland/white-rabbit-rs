import type {
  ReadApi,
  ReadModel,
  WriteApi,
  WriteModel,
  FindAllArgs,
  FindPageArgs,
  Page,
  Query,
  Command,
} from "@core/services";
import { invoke } from "@tauri-apps/api/tauri";

export abstract class AbstractReadApi<M extends ReadModel, Q extends Query, S extends string>
  implements ReadApi<M, Q, S>
{
  protected abstract convert(input: Record<string, unknown>): M;

  protected abstract loadIncluded(models: M[]): Promise<Map<string, ReadModel>>;

  protected abstract get findAllKey(): string;

  protected abstract get findByIdKey(): string;

  async findAll({ query, sort }: FindAllArgs<Q, S>): Promise<[M[], Map<string, ReadModel>]> {
    const response = await invoke<Record<string, unknown>[]>(this.findAllKey, {
      query,
      sort,
    });
    const models = response.map(this.convert);
    const included = await this.loadIncluded(models);
    return [models, included];
  }

  async findById(id: string): Promise<[M, Map<string, ReadModel>] | null> {
    const response = await invoke<Record<string, unknown> | null>(this.findByIdKey, {
      id,
    });

    if (response) {
      const model = this.convert(response);
      const included = await this.loadIncluded([model]);
      return [model, included];
    } else {
      return null;
    }
  }
}

export abstract class AbstractWriteApi<
    M extends WriteModel,
    Q extends Query,
    C extends Command,
    S extends string
  >
  extends AbstractReadApi<M, Q, S>
  implements WriteApi<M, Q, C, S>
{
  protected abstract get findPageKey(): string;

  protected abstract get handleCommandKey(): string;

  async findPage({
    query,
    sort,
    after,
    before,
    size,
  }: FindPageArgs<Q, S>): Promise<[Page<M>, Map<string, ReadModel>]> {
    const page = await invoke<Record<string, unknown>>(this.findPageKey, {
      query: query,
      sort: sort,
      after: after,
      before: before,
      size: size,
    });

    const models = (page.items as Record<string, unknown>[]).map(this.convert);
    const included = await this.loadIncluded(models);
    return [
      {
        hasPrevious: page.hasPrevious as boolean,
        hasNext: page.hasNext as boolean,
        items: models,
      },
      included,
    ];
  }

  async handleCommand(command: C): Promise<M[]> {
    console.log("Command: ", command);
    return [];
  }
}
