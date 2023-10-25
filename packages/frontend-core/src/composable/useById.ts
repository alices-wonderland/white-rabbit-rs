import type { AccountApi, EntryApi, JournalApi, Model, ReadApi } from "@core/services";
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

type UseByIdResult<M extends Model> = {
  readonly model: Ref<M | undefined>;
  readonly included: Ref<Map<string, Model>>;
  readonly loading: Ref<boolean>;
  readonly reload: () => Promise<void>;
};

const useById = <A extends ReadApi<M>, M extends Model>(
  key: symbol,
  id?: string | Ref<string>,
): UseByIdResult<M> => {
  const api = useInject<A>(key);
  const model = ref<M>();
  const included = ref<Map<string, Model>>(new Map());
  const loading = ref(false);

  const reload = async () => {
    const argsValue = isRef(id) ? id.value : id;

    if (!argsValue) {
      model.value = undefined;
      return;
    }

    loading.value = true;
    try {
      const result = await api.findById(argsValue);
      if (result) {
        model.value = result[0];
        included.value = result[1];
      }
    } finally {
      loading.value = false;
    }
  };

  onMounted(() => reload());

  if (isRef(id)) {
    watch(id, async (newArgs, oldArgs) => {
      if (!isEqual(newArgs, oldArgs)) {
        await reload();
      }
    });
  }

  return { model, included, loading, reload };
};

export const useJournal = (id?: string | Ref<string>) =>
  useById<JournalApi, Journal>(JOURNAL_API_KEY, id);

export const useAccount = (id?: string | Ref<string>) =>
  useById<AccountApi, Account>(ACCOUNT_API_KEY, id);

export const useEntry = (id?: string | Ref<string>) => useById<EntryApi, Entry>(ENTRY_API_KEY, id);
