/* eslint-disable sonarjs/no-duplicate-string */

import { Role, User } from "@shared/models";
import {
  AuthService,
  AuthUser,
  FindAllInput,
  UserQuery,
} from "@shared/services";
import { invoke } from "@tauri-apps/api/tauri";

const KEY_AUTH_USER = "KEY_AUTH_USER";

class LocalAuthService implements AuthService<string> {
  async getUser(): Promise<AuthUser<string> | null> {
    const token = localStorage.getItem(KEY_AUTH_USER);
    if (token) {
      return {
        token,
        user: undefined,
      };
    } else {
      // TODO: Need to check the token
      const users = await invoke<User[]>("find_users", {
        input: {
          query: { role: Role.OWNER },
          size: 1,
        } as FindAllInput<UserQuery>,
      });
      return {
        token: users[0].id,
        user: users[0],
      };
    }
  }

  signIn(): Promise<void> {
    throw new Error("Method not implemented.");
  }

  signInCallback(): Promise<void> {
    throw new Error("Method not implemented.");
  }
  signOut(): Promise<void> {
    throw new Error("Method not implemented.");
  }

  signOutCallback(): Promise<void> {
    throw new Error("Method not implemented.");
  }
}

export const AUTH_SERVICE = new LocalAuthService();
