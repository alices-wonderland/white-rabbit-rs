<script setup lang="ts">
import { AgGridVue } from "@ag-grid-community/vue3";
import type { AbstractColDef, GridOptions } from "@ag-grid-community/core";
import { GridApi } from "@ag-grid-community/core";

import { useQuasar } from "quasar";
import { computed, ref } from "vue";
import { watchDebounced } from "@vueuse/core";
import { omitBy } from "lodash";

const props: GridOptions = defineProps<GridOptions>();

const emits = defineEmits<{
  "update:gridApi": [value: GridApi];
}>();

const gridOptions = computed(
  (): GridOptions => ({
    ...omitBy(props, (value) => !value),
    enableRangeSelection: true,
    defaultColDef: {
      resizable: true,
      suppressMovable: true,
      floatingFilter: true,
      filter: true,
      useValueFormatterForExport: true,
      ...omitBy(props.defaultColDef ?? {}, (value) => !value),
    },
    onGridReady: (params) => {
      gridApi.value = params.api;
      emits("update:gridApi", params.api);
      props.onGridReady?.(params);
    },
  }),
);

const quasar = useQuasar();
const gridApi = ref<GridApi>();

const theme = computed(() => {
  return quasar.dark.isActive ? "ag-theme-alpine-dark" : "ag-theme-alpine";
});

watchDebounced(
  (): [unknown[] | undefined | null, AbstractColDef[] | undefined | null] => [
    props.rowData,
    props.columnDefs,
  ],
  ([newRows, newColDefs]) => {
    gridApi.value?.updateGridOptions({
      rowData: newRows,
      columnDefs: newColDefs,
    });
  },
  { debounce: 10 },
);
</script>

<template>
  <ag-grid-vue
    :style="{ minHeight: '150px', maxHeight: '80vh', height: '500px' }"
    :class="theme"
    v-bind="gridOptions"
    @grid-ready="onGridReady"
  ></ag-grid-vue>
</template>

<style scoped lang="scss">
.ag-theme-alpine,
.ag-theme-alpine-dark {
  padding: 6px;
  width: 100%;
  resize: vertical;
  overflow: auto;
}
</style>
