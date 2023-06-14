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

  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  protected async loadIncluded(models: M[]): Promise<Map<string, ReadModel>> {
    return new Map();
  }

  protected abstract get findAllKey(): string;

  protected abstract get findByIdKey(): string;

  async findAll(
    { query, sort }: FindAllArgs<Q, S>,
    loadIncluded?: boolean
  ): Promise<[M[], Map<string, ReadModel>]> {
    const response = await invoke<Record<string, unknown>[]>(this.findAllKey, {
      query,
      sort,
    });
    const models = response.map(this.convert);
    return [models, loadIncluded ? await this.loadIncluded(models) : new Map()];
  }

  async findById(id: string, loadIncluded?: boolean): Promise<[M, Map<string, ReadModel>] | null> {
    const response = await invoke<Record<string, unknown> | null>(this.findByIdKey, {
      id,
    });

    if (response) {
      const model = this.convert(response);
      return [model, loadIncluded ? await this.loadIncluded([model]) : new Map()];
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

  async findPage(
    { query, sort, after, before, size }: FindPageArgs<Q, S>,
    loadIncluded?: boolean
  ): Promise<[Page<M>, Map<string, ReadModel>]> {
    const page = await invoke<Record<string, unknown>>(this.findPageKey, {
      query: query,
      sort: sort,
      after: after,
      before: before,
      size: size,
    });

    const models = (page.items as Record<string, unknown>[]).map(this.convert);
    return [
      {
        hasPrevious: page.hasPrevious as boolean,
        hasNext: page.hasNext as boolean,
        items: models,
      },
      loadIncluded ? await this.loadIncluded(models) : new Map(),
    ];
  }

  async handleCommand(command: C): Promise<M[]> {
    console.log("Command: ", command);
    return [];
  }
}
