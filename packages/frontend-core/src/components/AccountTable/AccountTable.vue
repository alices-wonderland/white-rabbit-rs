<script setup lang="ts">
import type {
  AccountCommandDelete,
  AccountType,
  AccountCommandCreate,
  AccountCommandUpdate,
} from "@core/services";
import { Account, Journal, ACCOUNT_TYPES } from "@core/services";
import { AppTable, AppTableEditableCellRenderer } from "@core/components/AppTable";
import { computed, onMounted, shallowRef, triggerRef } from "vue";
import type { ColDef, ICellRendererParams } from "@ag-grid-community/core";
import { Row } from "./row";
import uniq from "lodash/uniq";

const props = defineProps<{
  readonly modelValue: Account[];
  readonly journal: Journal;
}>();

const columnDefs = computed((): ColDef<Row>[] => [
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
    editable: true,
    cellRenderer: AppTableEditableCellRenderer,
    cellRendererParams: (params: ICellRendererParams<Row>) => ({
      fieldState: params.data?.getFieldState("name"),
    }),
  } as ColDef<Row, string>,
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
    editable: true,
    cellEditor: "agLargeTextCellEditor",
    filter: "agTextColumnFilter",
    cellRenderer: AppTableEditableCellRenderer,
    cellRendererParams: (params: ICellRendererParams<Row>) => ({
      fieldState: params.data?.getFieldState("description"),
    }),
  } as ColDef<Row, string>,
  {
    headerName: "Unit",
    valueGetter: (params) => params.data?.unit,
    valueSetter: (params) => {
      if (params.newValue) {
        params.data.unit = params.newValue;
        triggerRef(rows);
        return true;
      }
      return false;
    },
    editable: true,
    cellRenderer: AppTableEditableCellRenderer,
    cellRendererParams: (params: ICellRendererParams<Row>) => ({
      fieldState: params.data?.getFieldState("unit"),
    }),
  } as ColDef<Row, string>,
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
    editable: true,
    cellRenderer: AppTableEditableCellRenderer,
    cellRendererParams: (params: ICellRendererParams<Row>) => ({
      fieldState: params.data?.getFieldState("type"),
    }),
    cellEditor: "agSelectCellEditor",
    cellEditorParams: () => ({
      values: ACCOUNT_TYPES,
    }),
  } as ColDef<Row, AccountType>,
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
    editable: true,
    cellRenderer: AppTableEditableCellRenderer,
    cellRendererParams: (params: ICellRendererParams<Row>) => ({
      fieldState: params.data?.getFieldState("tags"),
    }),
    useValueParserForImport: true,
    valueFormatter: (params) => params.value?.join(","),
    valueParser: (params) => params.newValue.split(","),
  } as ColDef<Row, string[]>,
]);

const rows = shallowRef<Row[]>([]);

onMounted(() => {
  rows.value = Row.ofAll(props.modelValue);
});

const createCommands = computed(() => {
  const commands: AccountCommandCreate[] = [];
  for (const row of rows.value) {
    if (row.rowState.state === "NEW" && row.name && row.unit) {
      commands.push({
        commandType: "accounts:create",
        journalId: props.journal.id,
        name: row.name,
        description: row.description,
        unit: row.unit,
        type: row.type,
        tags: row.tags,
      });
    }
  }
  return commands;
});

const updateCommands = computed(() => {
  const commands: AccountCommandUpdate[] = [];
  for (const row of rows.value) {
    if (row.rowState.state === "UPDATED" && row.name && row.unit) {
      commands.push({
        commandType: "accounts:update",
        id: row.id,
        name: row.name,
        description: row.description,
        unit: row.unit,
        type: row.type,
        tags: row.tags,
      });
    }
  }
  return commands;
});

const deleteCommand = computed((): AccountCommandDelete | undefined => {
  const ids = uniq(
    rows.value.filter((row) => row.rowState.state === "DELETED").map((row) => row.id),
  );
  if (ids.length > 0) {
    return {
      commandType: "accounts:delete",
      id: ids,
    };
  }

  return undefined;
});

const addRow = () => {
  rows.value = [new Row(), ...rows.value];
};
</script>

<template>
  <div class="flex flex-col gap-2">
    <div class="flex gap-1 items-center">
      <q-btn color="primary" icon="save" label="Save"></q-btn>
      <q-btn flat color="secondary" icon="add" label="Add" @click="addRow"></q-btn>
    </div>
    <div class="flex gap-1 items-center">
      <q-badge v-if="createCommands.length > 0" color="primary">
        {{ createCommands.length }} items added
      </q-badge>
      <q-badge v-if="updateCommands.length > 0" color="secondary">
        {{ updateCommands.length }} items updated
      </q-badge>
      <q-badge v-if="deleteCommand" color="negative">
        {{ deleteCommand.id.length }} items added
      </q-badge>
    </div>
    <AppTable :rows="rows" :column-defs="columnDefs"></AppTable>
  </div>
</template>
