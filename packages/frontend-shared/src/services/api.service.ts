import hash from "object-hash";

export interface ApiService<M, Q, C> {
  findById(id: string): Promise<M | null>;
  findPage(input: FindPageInput<Q>): Promise<Page<M>>;
  findAll(input: FindAllInput<Q>): Promise<M[]>;
  handle(input: C): Promise<M | null>;
  handleAll(input: C[]): Promise<Array<M | null>>;
}

export interface FindAllInput<Q> {
  readonly query?: Q;
  readonly size?: number;
  readonly sort?: Sort;
}

export interface FindPageInput<Q> {
  readonly query?: Q;
  readonly pagination?: Pagination;
  readonly sort: Sort;
}

export interface Page<M> {
  readonly info: PageInfo;
  readonly items: PageItem<M>[];
}

export interface PageInfo {
  readonly hasPrevious: boolean;
  readonly hasNext: boolean;
  readonly startCursor?: string;
  readonly endCursor?: string;
}

export interface PageItem<M> {
  readonly cursor: string;
  readonly item: M;
}

export interface Pagination {
  readonly after?: string;
  readonly before?: string;
  readonly size?: number;
}

export interface Sort {
  readonly field: string;
  readonly order: Order;
}

export enum Order {
  ASC = "Asc",
  DESC = "Desc",
}

export class CacheApiService<M, Q, C> implements ApiService<M, Q, C> {
  private readonly ttlMs: number;

  constructor(private type: string, private api: ApiService<M, Q, C>) {
    this.ttlMs = 15_000;
  }

  private hashInput(method: string, input: unknown): string {
    return hash(
      { type: this.type, method, input },
      { unorderedSets: true, unorderedObjects: true }
    );
  }

  private async doFind<I, O>(
    method: string,
    input: I,
    func: (input: I) => Promise<O>
  ): Promise<O> {
    const hash = this.hashInput(method, input);
    const cache = sessionStorage.getItem(hash);
    if (cache) {
      const { expiredAt, value } = JSON.parse(cache);
      if (new Date() <= new Date(expiredAt)) {
        return value;
      }
    }

    const value = await func(input);
    const expiredAt = new Date(new Date().getTime() + this.ttlMs).toISOString();
    sessionStorage.setItem(hash, JSON.stringify({ expiredAt, value }));
    return value;
  }

  async findById(id: string): Promise<M | null> {
    return await this.doFind("findById", id, (id) => this.api.findById(id));
  }

  async findPage(input: FindPageInput<Q>): Promise<Page<M>> {
    return await this.doFind("findPage", input, (input) =>
      this.api.findPage(input)
    );
  }

  async findAll(input: FindAllInput<Q>): Promise<M[]> {
    return await this.doFind("findAll", input, (input) =>
      this.api.findAll(input)
    );
  }

  async handle(input: C): Promise<M | null> {
    sessionStorage.clear();
    return await this.api.handle(input);
  }

  async handleAll(input: C[]): Promise<(M | null)[]> {
    sessionStorage.clear();
    return await this.api.handleAll(input);
  }
}
