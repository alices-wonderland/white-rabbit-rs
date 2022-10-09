export type IdQuery = string | string[];

export interface TextQuery {
  readonly value: string;
  readonly __fullText: boolean;
}

export interface FullTextQuery {
  readonly value: string;
  readonly fields?: string[];
}

export type ContainingUserQuery =
  | string
  | {
      readonly id: string;
      readonly fields?: string[];
    };

export interface ComparableQuery<V> {
  readonly __eq?: V;
  readonly __gt?: V;
  readonly __lt?: V;
  readonly __gte?: V;
  readonly __lte?: V;
}
