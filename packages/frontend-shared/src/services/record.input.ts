import { RecordItem, RecordType } from "../models";
import { ComparableQuery, FullTextQuery, IdQuery, TextQuery } from "./query";

export interface RecordQuery {
  readonly __fullText: FullTextQuery;
  readonly id?: IdQuery;
  readonly journal?: string;
  readonly name?: TextQuery;
  readonly description?: string;
  readonly type?: RecordType;
  readonly date?: ComparableQuery<Date>;
  readonly tag?: TextQuery;
  readonly account?: string;
}

export type RecordCommand =
  | {
      __type: "Create";
      targetId?: string;
      journalId: string;
      name: string;
      description: string;
      type: RecordType;
      date: Date;
      tags: Set<string>;
      items: Set<RecordItem>;
    }
  | {
      __type: "Update";
      targetId: string;
      name?: string;
      description?: string;
      type?: RecordType;
      date?: Date;
      tags?: Set<string>;
      items?: Set<RecordItem>;
    }
  | { __type: "Delete"; targetId: string };
