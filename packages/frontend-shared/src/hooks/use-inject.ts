import { Account, Group, Journal, Record_, User } from "@shared/models";
import {
  AccountCommand,
  AccountQuery,
  ApiService,
  GroupCommand,
  GroupQuery,
  JournalCommand,
  JournalQuery,
  RecordCommand,
  RecordQuery,
  UserCommand,
  UserQuery,
} from "@shared/services";
import { inject } from "vue";

export function useInject<S>(key: string, name?: string): S {
  const service = inject<S>(key);
  if (service == null) {
    throw new Error(
      `${name ?? key} is not available, please inject it when starting`
    );
  }
  return service;
}

export const KEY_USER_API = "KEY_USER_API";

export function useUserApi(): ApiService<User, UserQuery, UserCommand> {
  return useInject(KEY_USER_API);
}

export const KEY_GROUP_API = "KEY_GROUP_API";

export function useGroupApi(): ApiService<Group, GroupQuery, GroupCommand> {
  return useInject(KEY_GROUP_API);
}

export const KEY_JOURNAL_API = "KEY_JOURNAL_API";

export function useJournalApi(): ApiService<
  Journal,
  JournalQuery,
  JournalCommand
> {
  return useInject(KEY_JOURNAL_API);
}

export const KEY_ACCOUNT_API = "KEY_ACCOUNT_API";

export function useAccountApi(): ApiService<
  Account,
  AccountQuery,
  AccountCommand
> {
  return useInject(KEY_ACCOUNT_API);
}

export const KEY_RECORD_API = "KEY_RECORD_API";

export function useRecordApi(): ApiService<
  Record_,
  RecordQuery,
  RecordCommand
> {
  return useInject(KEY_RECORD_API);
}
