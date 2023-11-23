<script lang="ts">
import type { ICellEditorParams } from "@ag-grid-community/core";
import { computed, ref } from "vue";
import sortBy from "lodash/sortBy";
import sortedUniq from "lodash/sortedUniq";

type Props = { readonly params: ICellEditorParams<unknown, string[]> };

export default {
  setup(props: Props) {
    const value = ref<string[]>(props.params.value ?? []);

    const input = ref("");

    const options = computed(() =>
      sortedUniq(sortBy([...value.value, input.value].filter((item) => !!item))),
    );

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
    use-input
    use-chips
    multiple
    @input-value="input = $event"
  ></q-select>
</template>
