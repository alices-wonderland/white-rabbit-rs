import type { Command, FindAllArgs, Query, ReadApi, Model, WriteApi } from "@core/services";
import { invoke } from "@tauri-apps/api/tauri";
import { Notify } from "quasar";

export abstract class AbstractReadApi<M extends Model, Q extends Query, S extends string>
  implements ReadApi<M, Q, S>
{
  protected abstract convert(input: Record<string, unknown>): M;

  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  protected async loadIncluded(_models: M[]): Promise<Map<string, Model>> {
    return new Map();
  }

  protected abstract get findAllKey(): string;

  protected abstract get findByIdKey(): string;

  async findAll(
    { query, sort }: FindAllArgs<Q, S>,
    loadIncluded?: boolean,
  ): Promise<[M[], Map<string, Model>]> {
    let response: Record<string, unknown>[] = [];
    try {
      response = await invoke<Record<string, unknown>[]>(this.findAllKey, {
        query,
        sort,
      });
    } catch (e) {
      console.error(e);
      throw e;
    }

    const models = response.map(this.convert);
    return [models, loadIncluded ? await this.loadIncluded(models) : new Map()];
  }

  async findById(id: string, loadIncluded?: boolean): Promise<[M, Map<string, Model>] | null> {
    let response: Record<string, unknown> | null = null;
    try {
      response = await invoke<Record<string, unknown> | null>(this.findByIdKey, {
        id,
      });
    } catch (e) {
      console.error(e);
      throw e;
    }

    if (response) {
      const model = this.convert(response);
      return [model, loadIncluded ? await this.loadIncluded([model]) : new Map()];
    } else {
      return response;
    }
  }
}

export abstract class AbstractWriteApi<
    M extends Model,
    Q extends Query,
    C extends Command,
    S extends string,
  >
  extends AbstractReadApi<M, Q, S>
  implements WriteApi<M, Q, C, S>
{
  protected abstract get handleCommandKey(): string;
  async handleCommand(command: C): Promise<M[]> {
    let response: Record<string, unknown>[] = [];

    try {
      response = await invoke<Record<string, unknown>[]>(this.handleCommandKey, {
        command,
      });
    } catch (e) {
      Notify.create({
        color: "negative",
        message: e as string,
      });
      throw new Error(e as string);
    }

    return response.map(this.convert);
  }
}
