<script setup lang="ts">
import { AppTable, AppTableEditableCellRenderer } from "@core/components/AppTable";
import { Entry } from "@core/services";
import { computed, ref, triggerRef } from "vue";
import { createAll } from "./row";
import type { Row } from "./row";
import type { ColDef, ICellRendererParams } from "@ag-grid-community/core";
import EntryTableDateCellEditor from "./EntryTableDateCellEditor.vue";

const props = defineProps<{
  readonly modelValue: Entry[];
}>();

const readonly = ref(true);
const loading = ref(false);

const rows = computed(() => createAll(props.modelValue));

const columnDefs = computed((): ColDef<Row>[] => {
  return [
    {
      headerName: "Name",
      valueGetter: (params) => params.data?.name,
      valueSetter: (params) => {
        if (params.newValue) {
          params.data.name = params.newValue;
          triggerRef(rows);
          return true;
        }
        return false;
      },
      editable: !readonly.value,
      cellRenderer: AppTableEditableCellRenderer,
      cellRendererParams: (params: ICellRendererParams<Row>) => ({
        fieldState: params.data?.getFieldState("name"),
      }),
    },
    {
      headerName: "Date",
      valueGetter: (params) => params.data?.date,
      valueSetter: (params) => {
        if (params.newValue) {
          params.data.date = params.newValue;
          triggerRef(rows);
          return true;
        }
        return false;
      },
      editable: !readonly.value,
      cellRenderer: AppTableEditableCellRenderer,
      cellRendererParams: (params: ICellRendererParams<Row>) => ({
        fieldState: params.data?.getFieldState("date"),
      }),
      cellEditor: EntryTableDateCellEditor,
      filter: "agDateColumnFilter",
    },
  ];
});
</script>

<template>
  <div class="flex flex-col gap-2">
    <div class="flex gap-1 items-center">
      <template v-if="readonly">
        <q-btn
          color="primary"
          icon="edit"
          label="Edit"
          :loading="loading"
          @click="readonly = false"
        ></q-btn>
      </template>
      <template v-else>
        <q-btn flat color="secondary" icon="add" label="Add" :loading="loading"></q-btn>
        <q-btn
          flat
          icon="cancel"
          label="Cancel"
          :loading="loading"
          @click="readonly = true"
        ></q-btn>
      </template>
    </div>
    <div class="flex gap-1 items-center"></div>
    <AppTable :row-data="rows" :column-defs="columnDefs"></AppTable>
  </div>
</template>
