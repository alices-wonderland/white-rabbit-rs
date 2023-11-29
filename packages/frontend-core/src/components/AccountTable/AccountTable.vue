<script setup lang="ts">
import type {
  AccountType,
  AccountCommandCreate,
  AccountCommandUpdate,
  AccountCommandBatch,
  AccountApi,
} from "@core/services";
import { Account, Journal, ACCOUNT_TYPES, ACCOUNT_API_KEY } from "@core/services";
import {
  AppTable,
  AppTableEditableCellRenderer,
  AppTableTagsCellRenderer,
  AppTableTagsCellEditor,
} from "@core/components/AppTable";
import { computed, onMounted, ref, shallowRef, triggerRef } from "vue";
import type { CellValueChangedEvent, ColDef, ICellRendererParams } from "@ag-grid-community/core";
import { Row } from "./row";
import uniq from "lodash/uniq";
import AccountTableActionsCellRenderer from "./AccountTableActionsCellRenderer.vue";
import { useInject } from "@core/composable";

const props = defineProps<{
  readonly modelValue: Account[];
  readonly journal: Journal;
}>();

const emits = defineEmits<{
  reload: [];
}>();

const accountApi = useInject<AccountApi>(ACCOUNT_API_KEY);
const loading = ref(false);
const readonly = ref(true);

// eslint-disable-next-line sonarjs/cognitive-complexity
const columnDefs = computed(() => {
  let results: ColDef<Row>[] = [
    {
      headerName: "Name",
      valueGetter: (params) => params.data?.name,
      valueSetter: (params) => {
        if (params.newValue) {
          params.data.name = params.newValue;
          return true;
        }
        return false;
      },
      editable: !readonly.value,
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
      editable: !readonly.value,
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
          return true;
        }
        return false;
      },
      editable: !readonly.value,
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
          return true;
        }
        return false;
      },
      editable: !readonly.value,
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
      editable: !readonly.value,
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
              if (params.data?.isNew) {
                rows.value = rows.value.filter((row) => row.id !== params.data?.id);
              } else if (params.data) {
                params.data.deleted = !params.data.deleted;
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

const save = async () => {
  if (batchCommand.value) {
    try {
      loading.value = true;
      await accountApi.handleCommand(batchCommand.value);
      readonly.value = true;
      emits("reload");
    } finally {
      loading.value = false;
    }
  }
};

const onCellValueChanged = (event: CellValueChangedEvent<Row>) => {
  event.api.redrawRows({ rowNodes: [event.node] });
  triggerRef(rows);
};
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
        <q-btn
          color="primary"
          icon="save"
          label="Save"
          :loading="loading"
          :disable="!batchCommand"
          @click="save"
        ></q-btn>
        <q-btn
          flat
          color="secondary"
          icon="add"
          label="Add"
          :loading="loading"
          @click="addRow"
        ></q-btn>
        <q-btn
          flat
          icon="cancel"
          label="Cancel"
          :loading="loading"
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
