<script setup lang="ts" generic="R">
import { AgGridVue } from "@ag-grid-community/vue3";
import type { AbstractColDef, GridOptions, GridReadyEvent } from "@ag-grid-community/core";
import { ColumnApi, GridApi } from "@ag-grid-community/core";

import { useQuasar } from "quasar";
import { computed, ref, watch } from "vue";

const props = defineProps<GridOptions<R>>();

const gridOptions = computed(
  (): GridOptions<R> => ({
    ...props,
    rowData: undefined,
    columnDefs: undefined,
    enableRangeSelection: true,
    defaultColDef: {
      resizable: true,
      suppressMovable: true,
      floatingFilter: true,
      filter: true,
      useValueFormatterForExport: true,
      ...(props.defaultColDef ?? {}),
    },
  }),
);

const quasar = useQuasar();
const gridApi = ref<GridApi<R>>();
const columnApi = ref<ColumnApi>();

const theme = computed(() => {
  return quasar.dark.isActive ? "ag-theme-alpine-dark" : "ag-theme-alpine";
});

watch(
  (): [GridApi<R> | undefined, R[] | undefined | null, AbstractColDef<R>[] | undefined | null] => [
    gridApi.value,
    props.rowData,
    props.columnDefs,
  ],
  ([api, newRows, newColDefs]) => {
    if (!api) {
      return;
    }
    if (newRows) {
      api.setRowData(newRows);
    }
    if (newColDefs) {
      api.setColumnDefs(newColDefs);
    }
  },
);

const onGridReady = async (params: GridReadyEvent<R>) => {
  gridApi.value = params.api;
  columnApi.value = params.columnApi;
};
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
