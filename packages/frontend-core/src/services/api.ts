export interface ReadApi<M extends ReadModel, Q, S extends string> {
  findById(id: string): Promise<[M, ReadModel[]] | null>;

  findAll(args: FindAllArgs<Q, S>): Promise<[M[], ReadModel[]]>;
}

export interface WriteApi<M extends WriteModel, Q, C, S extends string> extends ReadApi<M, Q, S> {
  findPage(args: FindPageArgs<Q, S>): Promise<[Page<M>, ReadModel[]]>;

  handle(command: C): Promise<M[]>;
}

export interface ReadModel {
  id: string;

  get modelType(): string;
}

export interface WriteModel extends ReadModel {
  permission: Permission;
}

export interface FindAllArgs<Q, S extends string> {
  readonly query: Q;
  readonly sort: Array<[S, Order]>;
}

export interface FindPageArgs<Q, S extends string> extends FindAllArgs<Q, S> {
  readonly after?: string;
  readonly before?: string;
  readonly size: number;
}

export interface Page<M extends WriteModel> {
  readonly hasPrevious: boolean;
  readonly hasNext: boolean;
  readonly items: M[];
}

export type Permission = "ReadWrite" | "ReadOnly";

export type Order = "Asc" | "Desc";

export type Command<T extends string> = { commandType: T };
