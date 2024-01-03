<script setup lang="ts">
import type {
  AccountType,
  AccountCommandCreate,
  AccountCommandUpdate,
  AccountCommandBatch,
} from "@core/services";
import { Account, Journal, ACCOUNT_TYPES } from "@core/services";
import {
  AppTable,
  AppTableEditableCellRenderer,
  AppTableTagsCellRenderer,
  AppTableTagsCellEditor,
} from "@core/components/AppTable";
import { useAccountCommand } from "@core/composable";
import { computed, ref, shallowRef, triggerRef, watch } from "vue";
import type { CellValueChangedEvent, ColDef, ICellRendererParams } from "@ag-grid-community/core";
import { Row } from "./row";
import uniq from "lodash/uniq";

import AccountTableActionsCellRenderer from "./AccountTableActionsCellRenderer.vue";
import { useQueryClient } from "@tanstack/vue-query";

const props = defineProps<{
  readonly modelValue: Account[];
  readonly journal: Journal;
}>();

const queryClient = useQueryClient();
const readonly = ref(true);

// eslint-disable-next-line sonarjs/cognitive-complexity
const columnDefs = computed(() => {
  let results: ColDef<Row>[] = [
    {
      headerName: "Name",
      sortable: true,
      valueGetter: (params) => params.data?.name,
      valueSetter: (params) => {
        if (params.newValue) {
          params.data.name = params.newValue;
          return true;
        }
        return false;
      },
      editable: (params) => params.data?.editable("name"),
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
          return true;
        }
        return false;
      },
      editable: (params) => params.data?.editable("description"),
      cellEditor: "agLargeTextCellEditor",
      filter: "agTextColumnFilter",
      cellRenderer: AppTableEditableCellRenderer,
      cellRendererParams: (params: ICellRendererParams<Row>) => ({
        fieldState: params.data?.getFieldState("description"),
      }),
    } as ColDef<Row, string>,
    {
      headerName: "Unit",
      sortable: true,
      valueGetter: (params) => params.data?.unit,
      valueSetter: (params) => {
        if (params.newValue) {
          params.data.unit = params.newValue;
          return true;
        }
        return false;
      },
      editable: (params) => params.data?.editable("unit"),
      cellRenderer: AppTableEditableCellRenderer,
      cellRendererParams: (params: ICellRendererParams<Row>) => ({
        fieldState: params.data?.getFieldState("unit"),
      }),
    } as ColDef<Row, string>,
    {
      headerName: "Type",
      sortable: true,
      valueGetter: (params) => params.data?.type,
      valueSetter: (params) => {
        if (params.newValue) {
          params.data.type = params.newValue;
          return true;
        }
        return false;
      },
      editable: (params) => params.data?.editable("type"),
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
          return true;
        }
        return false;
      },
      editable: (params) => params.data?.editable("tags"),
      cellRenderer: AppTableTagsCellRenderer,
      cellRendererParams: (params: ICellRendererParams<Row>) => ({
        fieldState: params.data?.getFieldState("tags"),
      }),
      cellEditor: AppTableTagsCellEditor,
      useValueParserForImport: true,
      valueFormatter: (params) => params.value?.join(","),
      valueParser: (params) => params.newValue.split(","),
    } as ColDef<Row, string[]>,
  ];

  if (!readonly.value) {
    results = [
      {
        headerName: "Actions",
        cellRenderer: AccountTableActionsCellRenderer,
        cellRendererParams: (params: ICellRendererParams<Row>) => {
          return {
            toggleDeleted: () => {
              if (params.data?.rowState?.state === "NEW") {
                rows.value = rows.value.filter((row) => row.id !== params.data?.id);
              } else if (params.data) {
                params.data.deleted = !params.data.deleted;
                params.node && params.api.redrawRows({ rowNodes: [params.node] });
                triggerRef(rows);
              }
            },
          };
        },
        filter: false,
      } as ColDef<Row>,
      ...results,
    ];
  }

  return results;
});

const rows = shallowRef<Row[]>([]);
watch(
  (): [Account[], boolean] => [props.modelValue, readonly.value],
  ([newValues, newReadonly]) => {
    rows.value = Row.ofAll(newValues, newReadonly);
  },
  { immediate: true },
);

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

const deleteIds = computed((): string[] | undefined => {
  const ids = uniq(
    rows.value.filter((row) => row.rowState.state === "DELETED").map((row) => row.id),
  );
  if (ids.length > 0) {
    return ids;
  }

  return undefined;
});

const batchCommand = computed((): AccountCommandBatch | undefined => {
  if (
    !readonly.value &&
    (createCommands.value.length > 0 || updateCommands.value.length > 0 || deleteIds.value)
  ) {
    return {
      commandType: "accounts:batch",
      create: createCommands.value,
      update: updateCommands.value,
      delete: deleteIds.value,
    };
  }

  return undefined;
});

const addRow = () => {
  rows.value = [new Row(), ...rows.value];
};

const onCellValueChanged = (event: CellValueChangedEvent<Row>) => {
  console.log("On Cell Value Changed:", event);
  event.api.redrawRows({ rowNodes: [event.node] });
  triggerRef(rows);
};

const { mutateAsync: batchAsync, isPending: batchPending } = useAccountCommand({
  async onSuccess() {
    readonly.value = true;
    await queryClient.invalidateQueries({ queryKey: ["accounts"] });
  },
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
          :loading="batchPending"
          @click="readonly = false"
        ></q-btn>
      </template>
      <template v-else>
        <q-btn
          color="primary"
          icon="save"
          label="Save"
          :loading="batchPending"
          :disable="!batchCommand"
          @click="batchCommand && batchAsync(batchCommand)"
        ></q-btn>
        <q-btn
          flat
          color="secondary"
          icon="add"
          label="Add"
          :loading="batchPending"
          @click="addRow"
        ></q-btn>
        <q-btn
          flat
          icon="cancel"
          label="Cancel"
          :loading="batchPending"
          @click="readonly = true"
        ></q-btn>
      </template>
    </div>
    <div class="flex gap-1 items-center">
      <q-badge v-if="createCommands.length > 0" color="primary">
        {{ createCommands.length }} items added
      </q-badge>
      <q-badge v-if="updateCommands.length > 0" color="secondary">
        {{ updateCommands.length }} items updated
      </q-badge>
      <q-badge v-if="deleteIds" color="negative"> {{ deleteIds.length }} items deleted </q-badge>
    </div>
    <AppTable
      :row-data="rows"
      :column-defs="columnDefs"
      @cell-value-changed="onCellValueChanged"
    ></AppTable>
  </div>
</template>
