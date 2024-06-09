import type { MaybeRef } from "vue";
import { useQuery, type UseQueryOptions } from "@tanstack/vue-query";
import isEmpty from "lodash/isEmpty";
import { computed, toValue } from "vue";

import useInject from "./useInject";

import { ReadApi, type Model } from "src/services/api";
import { Journal, JOURNAL_API_KEY, JOURNAL_TYPE, type JournalApi } from "src/services/journal";
import { Account, ACCOUNT_API_KEY, ACCOUNT_TYPE, type AccountApi } from "src/services/account";
import { Entry, ENTRY_API_KEY, ENTRY_TYPE, type EntryApi } from "src/services/entry";
import {
  HierarchyReport,
  HIERARCHY_REPORT_API_KEY,
  HIERARCHY_REPORT_TYPE,
  type HierarchyReportApi,
} from "src/services/hierarchy-report";

const useById = <A extends ReadApi<M>, M extends Model>(
  key: symbol,
  methodName: string,
  id: MaybeRef<string | undefined>,
  options?: UseQueryOptions<[M, Map<string, Model>] | null>,
) => {
  const api = useInject<A>(key);
  const enabled = computed(() => !isEmpty(toValue(id)));
  const queryKey = computed((): [string, string | undefined] => [methodName, toValue(id)]);

  return useQuery<[M, Map<string, Model>] | null>({
    queryKey: queryKey,
    queryFn: async ({ queryKey: [_key, idValue] }) => {
      if (idValue && !isEmpty(idValue)) {
        return await api.findById(idValue as string, true);
      }
      return null;
    },
    enabled,
    ...(options ?? {}),
  });
};

export const useJournal = (
  id: MaybeRef<string | undefined>,
  options?: UseQueryOptions<[Journal, Map<string, Model>] | null>,
) => useById<JournalApi, Journal>(JOURNAL_API_KEY, JOURNAL_TYPE, id, options);

export const useAccount = (
  id?: MaybeRef<string | undefined>,
  options?: UseQueryOptions<[Account, Map<string, Model>] | null>,
) => useById<AccountApi, Account>(ACCOUNT_API_KEY, ACCOUNT_TYPE, id, options);

export const useEntry = (
  id?: MaybeRef<string | undefined>,
  options?: UseQueryOptions<[Entry, Map<string, Model>] | null>,
) => useById<EntryApi, Entry>(ENTRY_API_KEY, ENTRY_TYPE, id, options);

export const useHierarchyReport = (
  id?: MaybeRef<string | undefined>,
  options?: UseQueryOptions<[HierarchyReport, Map<string, Model>] | null>,
) =>
  useById<HierarchyReportApi, HierarchyReport>(
    HIERARCHY_REPORT_API_KEY,
    HIERARCHY_REPORT_TYPE,
    id,
    options,
  );
