import type { AccountApi, EntryApi, JournalApi, Model, ReadApi } from "@core/services";
import type { MaybeRef } from "vue";
import useInject from "./useInject";
import {
  Account,
  ACCOUNT_API_KEY,
  Entry,
  ENTRY_API_KEY,
  Journal,
  JOURNAL_API_KEY,
} from "@core/services";
import { useQuery, type UseQueryOptions } from "@tanstack/vue-query";
import isEmpty from "lodash/isEmpty";
import { computed, isRef } from "vue";

const useById = <A extends ReadApi<M>, M extends Model>(
  key: symbol,
  methodName: string,
  id: MaybeRef<string | undefined>,
  options?: UseQueryOptions<[M, Map<string, Model>] | null>,
) => {
  const api = useInject<A>(key);
  const enabled = computed(() =>
    isRef(id) ? !!id.value && !isEmpty(id.value) : !!id && !isEmpty(id),
  );

  return useQuery<[M, Map<string, Model>] | null>({
    queryKey: [methodName, id],
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
) => useById<JournalApi, Journal>(JOURNAL_API_KEY, "journal", id, options);

export const useAccount = (
  id?: MaybeRef<string | undefined>,
  options?: UseQueryOptions<[Account, Map<string, Model>] | null>,
) => useById<AccountApi, Account>(ACCOUNT_API_KEY, "account", id, options);

export const useEntry = (
  id?: MaybeRef<string | undefined>,
  options?: UseQueryOptions<[Entry, Map<string, Model>] | null>,
) => useById<EntryApi, Entry>(ENTRY_API_KEY, "entry", id, options);
