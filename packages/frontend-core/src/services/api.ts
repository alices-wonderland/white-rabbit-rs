export interface ReadApi<
  M extends Model = Model,
  Q extends Query = Query,
  S extends string = string,
> {
  findById(id: string, loadIncluded?: boolean): Promise<[M, Map<string, Model>] | null>;
  findAll(args: FindAllArgs<Q, S>, loadIncluded?: boolean): Promise<[M[], Map<string, Model>]>;
}

export interface WriteApi<
  M extends Model = Model,
  Q extends Query = Query,
  C extends Command = Command,
  S extends string = string,
> extends ReadApi<M, Q, S> {
  handleCommand(command: C): Promise<M[]>;
}

export interface Model<T extends string = string> {
  id: string;
  get modelType(): T;
}

export interface FindAllArgs<Q extends Query, S extends string> {
  query?: Q;
  sort?: S;
  size?: number;
}

export type Command<T extends string = string> = { readonly commandType: T };

export interface Query {
  id?: string[];
}
