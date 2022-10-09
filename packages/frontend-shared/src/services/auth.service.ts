import { User } from "@shared/models";

export type AuthService<T = unknown> = {
  getUser(): Promise<AuthUser<T> | null>;
  signIn(): Promise<void>;
  signInCallback(): Promise<void>;
  signOut(): Promise<void>;
  signOutCallback(): Promise<void>;
};

export type AuthUser<T = unknown> = {
  readonly user?: User;
  readonly token: T;
};
