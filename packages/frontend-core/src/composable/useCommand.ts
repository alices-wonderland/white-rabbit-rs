import type {
  AccountCommand,
  Command,
  EntryCommand,
  JournalCommand,
  Model,
  Query,
  WriteApi,
} from "@core/services";
import useInject from "./useInject";
import {
  Account,
  ACCOUNT_API_KEY,
  Entry,
  ENTRY_API_KEY,
  Journal,
  JOURNAL_API_KEY,
} from "@core/services";
import { useMutation, type UseMutationOptions } from "@tanstack/vue-query";
import type { DefaultError } from "@tanstack/query-core";

const useCommand = <
  A extends WriteApi<M, Q, C, S>,
  M extends Model,
  Q extends Query,
  C extends Command,
  S extends string,
>(
  key: symbol,
  options?: UseMutationOptions<M[], DefaultError, C>,
) => {
  const api = useInject<A>(key);
  return useMutation<M[], DefaultError, C>({
    ...(options ?? {}),
    mutationFn: (command) => api.handleCommand(command),
  });
};

export const useJournalCommand = (
  options?: UseMutationOptions<Journal[], DefaultError, JournalCommand>,
) => useCommand(JOURNAL_API_KEY, options);

export const useAccountCommand = (
  options?: UseMutationOptions<Account[], DefaultError, AccountCommand>,
) => useCommand(ACCOUNT_API_KEY, options);

export const useEntryCommand = (
  options?: UseMutationOptions<Entry[], DefaultError, EntryCommand>,
) => useCommand(ENTRY_API_KEY, options);
