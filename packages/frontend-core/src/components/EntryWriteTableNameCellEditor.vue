<template>
  <q-input v-if="isParent" v-model="value" label="Name" filled></q-input>
  <q-select
    v-else
    v-model="value"
    :options="params.availableAccounts"
    option-value="id"
    option-label="name"
    emit-value
    map-options
  ></q-select>
</template>

<script lang="ts">
import type { ICellEditorParams } from "@ag-grid-community/core";
import type { Row } from "./row";
import type { PropType } from "vue";
import { Parent } from "./row";
import { computed, onMounted, ref, watch } from "vue";
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
    watch(value, (newValue) => {
      console.log("New Value:", newValue);
    });
    onMounted(() => {
      value.value = props.params.value || undefined;
    });
    const getValue = () => value.value;
    const isParent = computed(() => props.params.data instanceof Parent);
    return { value, getValue, isParent };
  },
};
</script>
