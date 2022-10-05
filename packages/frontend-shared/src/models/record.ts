export interface Record_ {
  readonly id: string;
  readonly journalId: string;
  readonly name: string;
  readonly description: string;
  readonly type: RecordType;
  readonly date: Date;
  readonly tags: Set<string>;
  readonly items: Set<RecordItem>;
  readonly state: RecordState;
}

export interface RecordItem {
  readonly accountId: string;
  readonly amount: number;
  readonly price?: number;
}

export type RecordStateItem =
  | number
  | undefined
  | [number | undefined, number | undefined];

export type RecordState = RecordStateItem | Record<string, RecordStateItem>;

export interface CheckRecordState {
  readonly expacted: number;
  readonly actual: number;
}

export enum RecordType {
  RECORD = "Record",
  CHECK = "Check",
}
