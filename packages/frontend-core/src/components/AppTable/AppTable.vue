<script setup lang="ts" generic="R">
import { AgGridVue } from "@ag-grid-community/vue3";
import { ColumnApi, GridApi } from "@ag-grid-community/core";
import type { ColDef, GridReadyEvent, AbstractColDef } from "@ag-grid-community/core";
import { useQuasar } from "quasar";
import { computed, ref } from "vue";

const props = defineProps<{
  readonly rows: R[];
  readonly columnDefs: AbstractColDef[];
  readonly defaultColDef?: AbstractColDef;
}>();

const quasar = useQuasar();
const gridApi = ref<GridApi<R>>();
const columnApi = ref<ColumnApi>();

const theme = computed(() => {
  return quasar.dark.isActive ? "ag-theme-alpine-dark" : "ag-theme-alpine";
});

const actualDefaultColDef = computed(
  (): ColDef<R> => ({
    resizable: true,
    suppressMovable: true,
    floatingFilter: true,
    filter: true,
    useValueFormatterForExport: true,
    ...(props.defaultColDef ?? {}),
  }),
);
const onGridReady = async (params: GridReadyEvent) => {
  gridApi.value = params.api;
  columnApi.value = params.columnApi;
};
</script>

<template>
  <ag-grid-vue
    :style="{ minHeight: '150px', maxHeight: '80vh', height: '500px' }"
    :class="theme"
    :row-data="rows"
    :column-defs="columnDefs"
    :default-col-def="actualDefaultColDef"
    enable-range-selection
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
