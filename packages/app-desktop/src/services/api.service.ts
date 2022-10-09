import { Account, Group, Journal, Record_, User } from "@shared/models";

import {
  AccountCommand,
  AccountQuery,
  ApiService,
  AuthService,
  FindAllInput,
  FindPageInput,
  GroupCommand,
  GroupQuery,
  JournalCommand,
  JournalQuery,
  Page,
  RecordCommand,
  RecordQuery,
  UserCommand,
  UserQuery,
} from "@shared/services";
import { invoke } from "@tauri-apps/api/tauri";

interface TauriApiServiceOptions {
  readonly findById: string;
  readonly findPage: string;
  readonly findAll: string;
  readonly handle: string;
  readonly handleAll: string;
}

abstract class TauriApiService<M, Q, C> implements ApiService<M, Q, C> {
  constructor(
    private auth: AuthService,
    private options: TauriApiServiceOptions
  ) {}

  private async getOperator(): Promise<string | undefined> {
    const authUser = await this.auth.getUser();
    return authUser?.user?.id;
  }

  async findById(input: string): Promise<M | null> {
    return await invoke(this.options.findById, {
      operator: await this.getOperator(),
      input,
    });
  }

  async findPage(input: FindAllInput<Q>): Promise<Page<M>> {
    return await invoke(this.options.findPage, {
      operator: await this.getOperator(),
      input,
    });
  }

  async findAll(input: FindPageInput<Q>): Promise<M[]> {
    return await invoke(this.options.findAll, {
      operator: await this.getOperator(),
      input,
    });
  }

  async handle(input: C): Promise<M | null> {
    return await invoke(this.options.handle, {
      operator: await this.getOperator(),
      input,
    });
  }

  async handleAll(input: C[]): Promise<(M | null)[]> {
    return await invoke(this.options.handleAll, {
      operator: await this.getOperator(),
      input,
    });
  }
}

export class UserApiService extends TauriApiService<
  User,
  UserQuery,
  UserCommand
> {
  constructor(auth: AuthService) {
    super(auth, {
      findById: "find_user_by_id",
      findPage: "find_user_page",
      findAll: "find_users",
      handle: "handle_user",
      handleAll: "handle_users",
    });
  }
}

export class GroupApiService extends TauriApiService<
  Group,
  GroupQuery,
  GroupCommand
> {
  constructor(auth: AuthService) {
    super(auth, {
      findById: "find_group_by_id",
      findPage: "find_group_page",
      findAll: "find_groups",
      handle: "handle_group",
      handleAll: "handle_groups",
    });
  }
}

export class JournalApiService extends TauriApiService<
  Journal,
  JournalQuery,
  JournalCommand
> {
  constructor(auth: AuthService) {
    super(auth, {
      findById: "find_journal_by_id",
      findPage: "find_journal_page",
      findAll: "find_journals",
      handle: "handle_journal",
      handleAll: "handle_journals",
    });
  }
}

export class AccountApiService extends TauriApiService<
  Account,
  AccountQuery,
  AccountCommand
> {
  constructor(auth: AuthService) {
    super(auth, {
      findById: "find_account_by_id",
      findPage: "find_account_page",
      findAll: "find_accounts",
      handle: "handle_account",
      handleAll: "handle_accounts",
    });
  }
}

export class RecordApiService extends TauriApiService<
  Record_,
  RecordQuery,
  RecordCommand
> {
  constructor(auth: AuthService) {
    super(auth, {
      findById: "find_record_by_id",
      findPage: "find_record_page",
      findAll: "find_records",
      handle: "handle_record",
      handleAll: "handle_records",
    });
  }
}
