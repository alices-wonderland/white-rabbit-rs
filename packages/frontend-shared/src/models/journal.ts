export interface Journal {
  readonly id: string;
  readonly name: string;
  readonly description: string;
  readonly unit: string;
  readonly isArchived: boolean;
  readonly tags: Set<string>;
  readonly admins: Set<AccessItem>;
  readonly members: Set<AccessItem>;
}

export enum AccessItemType {
  USER = "User",
  GROUP = "Group",
}

export type AccessItem = [AccessItemType, string];
