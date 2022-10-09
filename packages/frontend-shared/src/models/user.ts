export interface User {
  readonly id: string;
  readonly name: string;
  readonly role: Role;
  readonly authIds: Set<AuthId>;
}

export type AuthId = [string, string];

export enum Role {
  USER = "User",
  ADMIN = "Admin",
  OWNER = "Owner",
}
