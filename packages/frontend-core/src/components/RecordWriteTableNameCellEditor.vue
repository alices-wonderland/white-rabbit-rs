<template>
  <v-text-field v-if="isParent" v-model="value"></v-text-field>
  <v-select
    v-else
    v-model="value"
    :items="params.availableAccounts"
    item-value="id"
    item-title="name"
  ></v-select>
</template>

<script lang="ts">
import type { ICellEditorParams } from "@ag-grid-community/core";
import type { Row } from "./row";
import type { PropType } from "vue";
import { Parent } from "./row";
import { computed, ref, watch } from "vue";
import { Account } from "@core/services";

type Params = PropType<ICellEditorParams<Row, string> & { availableAccounts: Account[] }>;

export default {
  props: {
    params: {
      type: Object as Params,
      required: true,
    },
  },
  setup(props) {
    const value = ref<string>();
    watch(props, () => {
      value.value = props.params.value || undefined;
    });
    const getValue = () => value.value;
    const isParent = computed(() => props.params.data instanceof Parent);
    return { value, getValue, isParent };
  },
};
</script>
