export interface ReadApi<
  M extends ReadModel = ReadModel,
  Q extends Query = Query,
  S extends string = string
> {
  findById(id: string, loadIncluded?: boolean): Promise<[M, Map<string, ReadModel>] | null>;
  findAll(args: FindAllArgs<Q, S>, loadIncluded?: boolean): Promise<[M[], Map<string, ReadModel>]>;
}

export interface WriteApi<
  M extends WriteModel = WriteModel,
  Q extends Query = Query,
  C extends Command = Command,
  S extends string = string
> extends ReadApi<M, Q, S> {
  findPage(
    args: FindPageArgs<Q, S>,
    loadIncluded?: boolean
  ): Promise<[Page<M>, Map<string, ReadModel>]>;
  handleCommand(command: C): Promise<M[]>;
}

export interface ReadModel<T extends string = string> {
  id: string;
  get modelType(): T;
}

export interface WriteModel<T extends string = string> extends ReadModel<T> {
  permission: Permission;
}
export interface FindAllArgs<Q extends Query, S extends string> {
  query: Q;
  sort?: Array<[S, Order]>;
  size?: number;
}

export interface FindPageArgs<Q extends Query, S extends string> extends FindAllArgs<Q, S> {
  sort: Array<[S, Order]>;
  after?: string;
  before?: string;
  size: number;
}

export interface Page<M extends WriteModel> {
  readonly hasPrevious: boolean;
  readonly hasNext: boolean;
  readonly items: M[];
}

export type Permission = "ReadWrite" | "ReadOnly";

export type Order = "Asc" | "Desc";

export type Command<T extends string = string> = { commandType: T };

export interface Query {
  id?: string[];
}
