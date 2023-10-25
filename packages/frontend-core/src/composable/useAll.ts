import type {
  AccountApi,
  AccountQuery,
  AccountSort,
  EntryApi,
  EntryQuery,
  EntrySort,
  FindAllArgs,
  JournalApi,
  JournalQuery,
  JournalSort,
  Model,
  Query,
  ReadApi,
} from "@core/services";
import type { Ref } from "vue";
import useInject from "./useInject";
import { ref, isRef, onMounted, watch } from "vue";
import isEqual from "lodash/isEqual";
import {
  Account,
  ACCOUNT_API_KEY,
  Entry,
  ENTRY_API_KEY,
  Journal,
  JOURNAL_API_KEY,
} from "@core/services";

export type UseAllArgs<Q extends Query, S extends string> =
  | FindAllArgs<Q, S>
  | Ref<FindAllArgs<Q, S>>
  | undefined;

type UseAllResult<M extends Model> = {
  readonly models: Ref<M[]>;
  readonly included: Ref<Map<string, Model>>;
  readonly loading: Ref<boolean>;
  readonly reload: () => Promise<void>;
};

const useAll = <A extends ReadApi<M, Q, S>, M extends Model, Q extends Query, S extends string>(
  key: symbol,
  args: UseAllArgs<Q, S>,
): UseAllResult<M> => {
  const api = useInject<A>(key);
  const models = ref<M[]>([]) as Ref<M[]>;
  const included = ref<Map<string, Model>>(new Map());
  const loading = ref(false);

  const reload = async () => {
    const argsValue = isRef(args) ? args.value : args;

    if (!argsValue) {
      models.value = [];
      return;
    }

    loading.value = true;
    try {
      const [newModels, newIncluded] = await api.findAll(argsValue);
      models.value = newModels;
      included.value = newIncluded;
    } finally {
      loading.value = false;
    }
  };

  onMounted(() => reload());

  if (isRef(args)) {
    watch(args, async (newArgs, oldArgs) => {
      if (!isEqual(newArgs, oldArgs)) {
        await reload();
      }
    });
  }

  return { models, included, loading, reload };
};

export const useJournals = (args: UseAllArgs<JournalQuery, JournalSort>) =>
  useAll<JournalApi, Journal, JournalQuery, JournalSort>(JOURNAL_API_KEY, args);

export const useAccounts = (args: UseAllArgs<AccountQuery, AccountSort>) =>
  useAll<AccountApi, Account, AccountQuery, AccountSort>(ACCOUNT_API_KEY, args);

export const useEntries = (args: UseAllArgs<EntryQuery, EntrySort>) =>
  useAll<EntryApi, Entry, EntryQuery, EntrySort>(ENTRY_API_KEY, args);
