<script setup lang="ts">
import {
  AppTable,
  AppTableEditableCellRenderer,
  AppTableTagsCellRenderer,
} from "@core/components/AppTable";
import { Entry } from "@core/services";
import { computed, onMounted, ref, shallowRef, triggerRef } from "vue";
import { createAll } from "./row";
import type { Row } from "./row";
import type { ColDef, ICellRendererParams } from "@ag-grid-community/core";

import EntryTableDateCellEditor from "./EntryTableDateCellEditor.vue";

const props = defineProps<{
  readonly modelValue: Entry[];
}>();

const readonly = ref(true);
const loading = ref(false);

const rows = shallowRef<Row[]>([]);

onMounted(() => {
  rows.value = createAll(props.modelValue);
});

const columnDefs = computed((): ColDef<Row>[] => {
  const results: ColDef<Row>[] = [
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
      headerName: "Description",
      valueGetter: (params) => params.data?.description,
      valueSetter: (params) => {
        if (params.newValue) {
          params.data.description = params.newValue;
          triggerRef(rows);
          return true;
        }
        return false;
      },
      editable: !readonly.value,
      cellRenderer: AppTableEditableCellRenderer,
      cellRendererParams: (params: ICellRendererParams<Row>) => ({
        fieldState: params.data?.getFieldState("description"),
      }),
    },
    {
      headerName: "Type",
      valueGetter: (params) => params.data?.type,
      valueSetter: (params) => {
        if (params.newValue) {
          params.data.type = params.newValue;
          triggerRef(rows);
          return true;
        }
        return false;
      },
      editable: !readonly.value,
      cellRenderer: AppTableEditableCellRenderer,
      cellRendererParams: (params: ICellRendererParams<Row>) => ({
        fieldState: params.data?.getFieldState("type"),
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
    {
      headerName: "Tags",
      valueGetter: (params) => params.data?.tags,
      valueSetter: (params) => {
        if (params.newValue) {
          params.data.tags = params.newValue;
          triggerRef(rows);
          return true;
        }
        return false;
      },
      editable: !readonly.value,
      cellRenderer: AppTableTagsCellRenderer,
      cellRendererParams: (params: ICellRendererParams<Row>) => ({
        fieldState: params.data?.getFieldState("tags"),
      }),
    },
  ];

  if (readonly.value) {
    return [
      ...results,
      {
        headerName: "State",
        valueGetter: (params) => params.data?.entryState,
      },
    ];
  }

  return results;
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
