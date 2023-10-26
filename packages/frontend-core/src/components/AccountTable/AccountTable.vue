<script setup lang="ts">
import { Account, ACCOUNT_TYPES, Journal } from "@core/services";
import { AppTable, AppTableEditableCellRenderer } from "@core/components/AppTable";
import { computed, onMounted, shallowRef } from "vue";
import type { ColDef, ICellRendererParams } from "@ag-grid-community/core";
import { Row } from "./row";

const props = defineProps<{
  readonly modelValue: Account[];
  readonly journals: Map<string, Journal>;
}>();

const columnDefs = computed((): ColDef<Row>[] => [
  {
    field: "name",
    editable: true,
    cellRenderer: AppTableEditableCellRenderer,
    cellRendererParams: (params: ICellRendererParams<Row>) => ({
      editableValue: params.data?.nameValue,
    }),
  },
  {
    field: "description",
    editable: true,
    cellEditor: "agLargeTextCellEditor",
    filter: "agTextColumnFilter",
    cellRenderer: AppTableEditableCellRenderer,
    cellRendererParams: (params: ICellRendererParams<Row>) => ({
      editableValue: params.data?.descriptionValue,
    }),
  },
  {
    field: "unit",
    editable: true,
    cellRenderer: AppTableEditableCellRenderer,
    cellRendererParams: (params: ICellRendererParams<Row>) => ({
      editableValue: params.data?.unitValue,
    }),
  },
  {
    field: "type",
    editable: true,
    cellRenderer: AppTableEditableCellRenderer,
    cellRendererParams: (params: ICellRendererParams<Row>) => ({
      editableValue: params.data?.typeValue,
    }),
    cellEditor: "agSelectCellEditor",
    cellEditorParams: () => ({
      values: ACCOUNT_TYPES,
    }),
  },
  {
    field: "tags",
    editable: true,
    cellRenderer: AppTableEditableCellRenderer,
    cellRendererParams: (params: ICellRendererParams<Row>) => ({
      editableValue: params.data?.tagsValue,
    }),
  } as ColDef<Row, string[]>,
]);

const rows = shallowRef<Row[]>([]);

onMounted(() => {
  rows.value = Row.ofAll(props.modelValue);
});
</script>

<template>
  <AppTable :rows="rows" :column-defs="columnDefs"></AppTable>
</template>
