import { computed, toValue, type MaybeRef } from "vue";
import { useQuery, type UseQueryOptions } from "@tanstack/vue-query";
import useInject from "./useInject";

import { ReadApi, type Model, type Query, type FindAllArgs } from "src/services/api";
import {
  Journal,
  JOURNAL_API_KEY,
  JOURNAL_TYPE,
  type JournalApi,
  type JournalQuery,
  type JournalSort,
} from "src/services/journal";
import {
  Account,
  ACCOUNT_API_KEY,
  ACCOUNT_TYPE,
  type AccountApi,
  type AccountQuery,
  type AccountSort,
} from "src/services/account";
import {
  Entry,
  ENTRY_API_KEY,
  ENTRY_TYPE,
  type EntryApi,
  type EntryQuery,
  type EntrySort,
} from "src/services/entry";
import {
  HierarchyReport,
  HIERARCHY_REPORT_API_KEY,
  HIERARCHY_REPORT_TYPE,
  type HierarchyReportApi,
  type HierarchyReportQuery,
} from "src/services/hierarchy-report";

export type UseAllArgs<Q extends Query, S extends string = string> = MaybeRef<
  FindAllArgs<Q, S> | undefined
>;

const useAll = <
  A extends ReadApi<M, Q, S>,
  M extends Model,
  Q extends Query,
  S extends string = string,
>(
  key: symbol,
  methodName: string,
  args: UseAllArgs<Q, S>,
  options?: UseQueryOptions<[M[], Map<string, Model>]>,
) => {
  const api = useInject<A>(key);
  const enabled = computed(() => !!toValue(args));
  const queryKey = computed<[string, FindAllArgs<Q, S> | undefined]>(() => [
    methodName,
    toValue(args),
  ]);

  return useQuery({
    queryKey: queryKey,
    queryFn: async ({ queryKey: [_key, argsValue] }) => {
      if (argsValue) {
        return await api.findAll(argsValue as FindAllArgs<Q, S>);
      }
      return [[], new Map()] satisfies [M[], Map<string, Model>];
    },
    enabled: enabled,
    ...(options ?? {}),
  });
};

export const useJournals = (
  args: UseAllArgs<JournalQuery, JournalSort>,
  options?: UseQueryOptions<[Journal[], Map<string, Model>]>,
) =>
  useAll<JournalApi, Journal, JournalQuery, JournalSort>(
    JOURNAL_API_KEY,
    JOURNAL_TYPE,
    args,
    options,
  );

export const useAccounts = (
  args: UseAllArgs<AccountQuery, AccountSort>,
  options?: UseQueryOptions<[Account[], Map<string, Model>]>,
) =>
  useAll<AccountApi, Account, AccountQuery, AccountSort>(
    ACCOUNT_API_KEY,
    ACCOUNT_TYPE,
    args,
    options,
  );

export const useEntries = (
  args: UseAllArgs<EntryQuery, EntrySort>,
  options?: UseQueryOptions<[Entry[], Map<string, Model>]>,
) => useAll<EntryApi, Entry, EntryQuery, EntrySort>(ENTRY_API_KEY, ENTRY_TYPE, args, options);

export const useHierarchyReports = (
  args: UseAllArgs<HierarchyReportQuery>,
  options?: UseQueryOptions<[HierarchyReport[], Map<string, Model>]>,
) =>
  useAll<HierarchyReportApi, HierarchyReport, HierarchyReportQuery>(
    HIERARCHY_REPORT_API_KEY,
    HIERARCHY_REPORT_TYPE,
    args,
    options,
  );
