export interface Group {
  readonly id: string;
  readonly name: string;
  readonly description: string;
  readonly admins: Set<string>;
  readonly members: Set<string>;
}
