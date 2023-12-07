<script setup lang="ts">
import { AgGridVue } from "@ag-grid-community/vue3";
import type { GridOptions } from "@ag-grid-community/core";
import { GridApi } from "@ag-grid-community/core";
import { useQuasar } from "quasar";
import { computed, ref, watch } from "vue";
import omitBy from "lodash/omitBy";
import { useElementSize } from "@vueuse/core";

const props: GridOptions = defineProps<GridOptions>();

const emits = defineEmits<{
  "update:gridApi": [value: GridApi];
}>();

const el = ref<HTMLElement>();
const { height } = useElementSize(el);

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
    onFirstDataRendered: (params) => {
      params.api.autoSizeAllColumns();
      props.onFirstDataRendered?.(params);
    },
  }),
);

const quasar = useQuasar();
const gridApi = ref<GridApi>();

const theme = computed(() => {
  return quasar.dark.isActive ? "ag-theme-alpine-dark" : "ag-theme-alpine";
});

watch(
  () => [props.rowData, props.columnDefs],
  ([newRowData, newColumnDefs]) => {
    console.log("AppTable Props Update");
    console.log("  rowData:", props.rowData);
    gridApi.value?.updateGridOptions({
      rowData: newRowData,
      columnDefs: newColumnDefs,
    });
  },
);
</script>

<template>
  <ag-grid-vue
    ref="el"
    :style="{ minHeight: '500px', maxHeight: '80vh', height }"
    :class="theme"
    v-bind="gridOptions"
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
