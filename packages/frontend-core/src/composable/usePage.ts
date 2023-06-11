import { inject, ref, watch } from "vue";
import type { Ref } from "vue";
import type { Command, FindPageArgs, Query, ReadModel, WriteApi, WriteModel } from "@core/services";

export function usePage<
  M extends WriteModel,
  Q extends Query,
  S extends string,
  A extends WriteApi<M, Q, Command, S>
>(
  key: symbol,
  defaultArgs: FindPageArgs<Q, S>
): [Ref<FindPageArgs<Q, S>>, Ref<M[]>, Ref<Map<string, ReadModel>>, () => Promise<void>] {
  const api = inject<A>(key);
  if (!api) {
    throw new Error(`Api with ${String(key)} cannot be found`);
  }

  const args = ref<FindPageArgs<Q, S>>(defaultArgs) as Ref<FindPageArgs<Q, S>>;
  const models = ref<M[]>([]) as Ref<M[]>;
  const included = ref<Map<string, ReadModel>>(new Map());
  const loadNext = async () => {
    if (args.value) {
      const page = await api.findPage({
        ...args.value,
        after: models.value[models.value.length - 1]?.id,
      });
      if (page) {
        models.value.push(...page[0].items);
        included.value = new Map([...included.value, ...page[1]]);
      }
    }
  };

  watch(args, () => {
    models.value = [];
    included.value = new Map();
  });

  return [args, models, included, loadNext];
}
