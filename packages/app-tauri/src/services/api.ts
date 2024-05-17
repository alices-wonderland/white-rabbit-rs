import type {
  Command,
  FindAllArgs,
  Query,
  ReadApi,
  Model,
  WriteApi,
  ProblemDetail,
} from "@core/services";
import { invoke } from "@tauri-apps/api/core";
import { Notify } from "quasar";

const handleError = (e: ProblemDetail) => {
  Notify.create({
    color: "negative",
    message: `<strong>${e.title}</strong>
<br>
${e.detail}`,
    html: true,
  });
};

export abstract class AbstractReadApi<M extends Model, Q extends Query, S extends string = string>
  implements ReadApi<M, Q, S>
{
  protected abstract convert(input: Record<string, unknown>): M;

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
      handleError(e as ProblemDetail);
    }

    const models = response.map((record) => this.convert(record));
    return [models, loadIncluded ? await this.loadIncluded(models) : new Map()];
  }

  async findById(id: string, loadIncluded?: boolean): Promise<[M, Map<string, Model>] | null> {
    let response: Record<string, unknown> | null = null;
    try {
      response = await invoke<Record<string, unknown> | null>(this.findByIdKey, {
        id,
      });
    } catch (e) {
      handleError(e as ProblemDetail);
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
      handleError(e as ProblemDetail);
    }

    return response.map((record) => this.convert(record));
  }
}
