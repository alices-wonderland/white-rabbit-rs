<script lang="ts">
import { Account } from "@core/services";

import type { ICellEditorParams } from "@ag-grid-community/core";
import { computed, ref } from "vue";
import sortBy from "lodash/sortBy";
import sortedUniqBy from "lodash/sortedUniqBy";

type Params = ICellEditorParams<unknown, string> & {
  readonly accounts: Account[];
};

type Props = { readonly params: Params };

export default {
  setup(props: Props) {
    const value = ref<string>(props.params.value ?? "");

    const input = ref("");

    const options = computed(() => {
      const trimmed = input.value.trim().toLowerCase();

      return sortedUniqBy(
        sortBy(
          props.params.accounts
            .filter((account) => {
              if (trimmed) {
                return account.name.toLowerCase().includes(trimmed);
              }
              return true;
            })
            .map((account) => ({
              label: account.name,
              value: account.id,
            })),
          "label",
        ),
        "value",
      );
    });

    const getValue = () => value.value;

    return {
      value,
      input,
      options,
      getValue,
    };
  },
};
</script>

<template>
  <q-select
    v-model="value"
    dense
    :options="options"
    use-chips
    use-input
    emit-value
    map-options
    @input-value="input = $event"
  ></q-select>
</template>
