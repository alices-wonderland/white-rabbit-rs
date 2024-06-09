<script setup lang="ts">
import { AgGridVue } from "@ag-grid-community/vue3";
import type { GridOptions } from "@ag-grid-community/core";
import { GridApi } from "@ag-grid-community/core";
import { useQuasar } from "quasar";
import { computed, reactive, ref, watch } from "vue";
import omitBy from "lodash/omitBy";
import { useElementSize } from "@vueuse/core";

const props: GridOptions = defineProps<GridOptions>();

const emits = defineEmits<{
  "update:gridApi": [value: GridApi];
}>();

const el = ref<HTMLElement>();
const { height } = useElementSize(el);
const expandedRowKeys = reactive(new Set<string>());

const gridOptions = computed(
  (): GridOptions => ({
    ...omitBy(props, (value) => !value),
    enableRangeSelection: true,
    suppressGroupRowsSticky: true,
    defaultColDef: {
      resizable: true,
      suppressMovable: true,
      floatingFilter: true,
      filter: true,
      sortable: false,
      useValueFormatterForExport: true,
      ...omitBy(props.defaultColDef ?? {}, (value) => !value),
    },
    onGridReady: (params) => {
      console.log("On Grid Ready");
      gridApi.value = params.api;
      emits("update:gridApi", params.api);
      props.onGridReady?.(params);
    },
    onRowDataUpdated: (params) => {
      console.log("On Row Data Updated");
      props.onRowDataUpdated?.(params);
      if (expandedRowKeys.size > 0) {
        params.api.forEachNode((node) => {
          if (node.key && expandedRowKeys.has(node.key)) {
            params.api.setRowNodeExpanded(node, true);
          }
        });
        expandedRowKeys.clear();
      }
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
    const api = gridApi.value;
    if (api) {
      expandedRowKeys.clear();
      api.forEachNode((node) => {
        if (node.expanded && node.key) {
          expandedRowKeys.add(node.key);
        }
      });

      api.updateGridOptions({
        rowData: newRowData,
        columnDefs: newColumnDefs,
      });
    }
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
