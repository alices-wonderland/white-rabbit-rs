export interface ReadApi<
  M extends ReadModel = ReadModel,
  Q extends Query = Query,
  S extends string = string
> {
  findById(id: string): Promise<[M, Map<string, ReadModel>] | null>;
  findAll(args: FindAllArgs<Q, S>): Promise<[M[], Map<string, ReadModel>]>;
}

export interface WriteApi<
  M extends WriteModel,
  Q extends Query,
  C extends Command,
  S extends string
> extends ReadApi<M, Q, S> {
  findPage(args: FindPageArgs<Q, S>): Promise<[Page<M>, Map<string, ReadModel>]>;
  handleCommand(command: C): Promise<M[]>;
}

export interface ReadModel {
  id: string;
  get modelType(): string;
}

export interface WriteModel extends ReadModel {
  permission: Permission;
}
export interface FindAllArgs<Q extends Query, S extends string> {
  query: Q;
  sort: Array<[S, Order]>;
  size?: number;
}

export interface FindPageArgs<Q extends Query, S extends string> extends FindAllArgs<Q, S> {
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
