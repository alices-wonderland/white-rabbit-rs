import type {
  AccountApi,
  AccountQuery,
  AccountSort,
  EntryApi,
  EntryQuery,
  EntrySort,
  FindAllArgs,
  HierarchyReportApi,
  HierarchyReportQuery,
  JournalApi,
  JournalQuery,
  JournalSort,
  Model,
  Query,
  ReadApi,
} from "@core/services";
import { computed, isRef, type MaybeRef } from "vue";
import useInject from "./useInject";
import {
  Account,
  ACCOUNT_API_KEY,
  Entry,
  ENTRY_API_KEY,
  Journal,
  JOURNAL_API_KEY,
  EMPTY_RESULTS,
  HierarchyReport,
  HIERARCHY_REPORT_API_KEY,
  HIERARCHY_REPORT_TYPE,
  ACCOUNT_TYPE,
  ENTRY_TYPE,
  JOURNAL_TYPE,
} from "@core/services";
import { useQuery, type UseQueryOptions } from "@tanstack/vue-query";

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
  const enabled = computed(() => (isRef(args) ? !!args.value : !!args));

  return useQuery({
    queryKey: [methodName, args],
    queryFn: async ({ queryKey: [_key, argsValue] }) => {
      if (argsValue) {
        return await api.findAll(argsValue as FindAllArgs<Q, S>);
      }
      return EMPTY_RESULTS as [M[], Map<string, Model>];
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
