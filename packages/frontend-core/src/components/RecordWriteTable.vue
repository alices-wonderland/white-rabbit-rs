<template>
  <v-btn color="primary" size="small" @click="saveEditedRows">Save</v-btn>
  <ag-grid-vue
    ref="tableRef"
    :style="{ minHeight: '150px', maxHeight: '80vh', height: heightPx }"
    :class="theme.global.name.value === 'dark' ? 'ag-theme-alpine-dark' : 'ag-theme-alpine'"
    :column-defs="columnDefs"
    :default-col-def="defaultColDef"
    group-display-type="custom"
    :get-data-path="getDataPath"
    :get-context-menu-items="getContextMenuItems"
    group-default-expanded
    animate-rows
    tree-data
    enable-range-selection
    @grid-ready="onGridReady"
  ></ag-grid-vue>
</template>

<script setup lang="ts">
import {
  type ColDef,
  type EditableCallbackParams,
  type GridReadyEvent,
  type ValueGetterParams,
  type ValueSetterParams,
  type CellClassParams,
  type AbstractColDef,
  type ColGroupDef,
  type GetContextMenuItems,
  ColumnApi,
  GridApi,
} from "@ag-grid-community/core";
import { AgGridVue } from "@ag-grid-community/vue3";
import { useInject } from "@core/composable";
import {
  type JournalApi,
  type RecordApi,
  type RecordType,
  type AccountApi,
  ACCOUNT_TYPE,
  JOURNAL_API_KEY,
  RECORD_API_KEY,
  ACCOUNT_API_KEY,
  Journal,
  type RecordCommandUpdate,
  type RecordCommandCreate,
} from "@core/services";
import { ref, watch } from "vue";
import { useTheme } from "vuetify";
import RecordWriteTableActionsCellRenderer from "./RecordWriteTableActionsCellRenderer.vue";
import RecordWriteTableGroupCellRenderer from "./RecordWriteTableGroupCellRenderer.vue";
import RecordWriteTableStateCellRenderer from "./RecordWriteTableStateCellRenderer.vue";
import RecordWriteTableNameCellEditor from "./RecordWriteTableNameCellEditor.vue";
import { Child, Parent, type Row } from "./row";
import { NULL_PLACEHOLDER, toMap } from "@core/utils";
import { computedAsync } from "@vueuse/core";
import { computed } from "vue";

const props = defineProps<{ journal: Journal }>();
const theme = useTheme();

const rows = ref<Row[]>([]);
const recordApi = useInject<RecordApi>(RECORD_API_KEY);
const _journalApi = useInject<JournalApi>(JOURNAL_API_KEY);
const accountApi = useInject<AccountApi>(ACCOUNT_API_KEY);
const gridApi = ref<GridApi<Row>>();
const columnApi = ref<ColumnApi>();
const tableRef = ref<HTMLElement>();

const columnDefs = ref<AbstractColDef[]>([
  {
    headerName: "Actions",
    cellRenderer: RecordWriteTableActionsCellRenderer,
    cellRendererParams: (params: CellClassParams<Row>) => ({
      addChild: () => {
        const data = params.data;
        if (data instanceof Parent) {
          const child = new Child({ parent: data, isDeleted: false });
          data.children.push(child);
          rows.value = [...rows.value, child];
        }
      },
      clone: () => {
        const data = params.data;
        if (data instanceof Parent) {
          const parent = data.clone();
          rows.value = [...rows.value, parent, ...parent.children];
        } else if (data instanceof Child) {
          rows.value = [...rows.value, data.clone()];
        }
      },
    }),
    pinned: "left",
    sortable: false,
  } as ColDef,
  {
    headerName: "Record",
    children: [
      {
        headerName: "Name",
        cellRenderer: "agGroupCellRenderer",
        cellRendererParams: {
          innerRenderer: RecordWriteTableGroupCellRenderer,
          suppressDoubleClickExpand: true,
        },
        valueGetter: (params: ValueGetterParams<Row>) =>
          params.data instanceof Parent ? params.data.name : params.data?.account?.id,
        valueSetter(params: ValueSetterParams<Row, string>): boolean {
          if (!params.newValue) {
            return false;
          }
          if (params.data instanceof Parent) {
            params.data.name = params.newValue;
            return true;
          } else {
            const account = accounts.value.find((a) => params.newValue === a.id);
            if (account) {
              params.data.account = account;
            }
            return !!account;
          }
        },
        editable: true,
        cellEditor: RecordWriteTableNameCellEditor,
        cellEditorParams: () => ({
          availableAccounts: accounts.value,
        }),
        filter: true,
        showRowGroup: true,
        filterParams: {
          valueGetter: (params: ValueGetterParams<Row>) => params.data?.name,
        },
        cellClassRules: {
          "cell cell-edited": (params: CellClassParams<Row>) =>
            params.data?.editedFields?.has("name"),
        },
      } as ColDef,
      {
        headerName: "Unit",
        sortable: false,
        columnGroupShow: "open",
        valueGetter: (params: ValueGetterParams<Row>) =>
          params.data instanceof Parent
            ? params.data.journal.unit
            : params.data?.account?.unit ?? NULL_PLACEHOLDER,
      } as ColDef,
      {
        headerName: "State",
        cellRenderer: RecordWriteTableStateCellRenderer,
        sortable: false,
        columnGroupShow: "open",
      } as ColDef,
    ],
  } as ColGroupDef,
  {
    headerName: "Date",
    field: "date",
    filter: "agDateColumnFilter",
    editable: (params: EditableCallbackParams<Row, Date>) => params.data instanceof Parent,
    cellClassRules: {
      "cell cell-edited": (params: CellClassParams<Row>) =>
        params.data?.editedFields?.has("date") ?? false,
    },
    valueSetter: (params: ValueSetterParams<Row, Date>) => {
      if (params.newValue && params.data instanceof Parent) {
        params.data.date = params.newValue;
        return true;
      }
      return false;
    },
  } as ColDef,
  {
    headerName: "Type",
    valueGetter: (params: ValueGetterParams<Row>) =>
      params.data instanceof Parent ? params.data.type : params.data?.account?.type,
    editable: (params: EditableCallbackParams<Row, RecordType>) => params.data instanceof Parent,
    sortable: false,
    filter: true,
    cellEditor: "agSelectCellEditor",
    cellEditorParams: {
      values: ["Record", "Check"] as RecordType[],
    },
    valueSetter: (params: ValueSetterParams<Row, RecordType>) => {
      if (params.data instanceof Parent && params.newValue) {
        params.data.type = params.newValue;
        return true;
      }
      return false;
    },
    cellClassRules: {
      "cell cell-edited": (params: CellClassParams<Row>) => params.data?.editedFields?.has("type"),
    },
  } as ColDef,
  {
    headerName: "Amount",
    field: "amount",
    sortable: false,
    editable: (params: EditableCallbackParams<Row, number>) => params.data instanceof Child,
    cellEditor: "agNumberCellEditor",
    cellClassRules: {
      "cell cell-edited": (params: CellClassParams<Row>) =>
        params.data?.editedFields?.has("amount") ?? false,
    },
  } as ColDef,
  {
    headerName: "Price",
    field: "price",
    sortable: false,
    editable: (params: EditableCallbackParams<Row, number>) => params.data instanceof Child,
    cellEditor: "agNumberCellEditor",
    cellClassRules: {
      "cell cell-edited": (params: CellClassParams<Row>) => {
        const isEdited = params.data?.editedFields?.has("price");
        if (isEdited) {
          console.log("edited: ", params.data?.editedFields?.get("price"));
        }
        return isEdited ?? false;
      },
    },
  } as ColDef,
]);

const defaultColDef: ColDef = {
  flex: 1,
  minWidth: 100,
  sortable: true,
  floatingFilter: true,
  resizable: true,
  suppressMovable: true,
  suppressMenu: true,
};

const accounts = computedAsync(async () => {
  const [results, _i] = await accountApi.findAll({
    query: { journal: [props.journal.id] },
  });
  return results;
});

const records = computedAsync(async () => {
  const [records, _i] = await recordApi.findAll({ query: { journal: [props.journal.id] } });
  return records;
});

const onGridReady = async (params: GridReadyEvent) => {
  gridApi.value = params.api;
  columnApi.value = params.columnApi;
};

const height = ref<number>(400);
const heightPx = computed(() => height.value + "px");

const generateRows = () => {
  if (!accounts.value || !records.value) {
    return [];
  }

  const accountMap = toMap(accounts.value);
  const newRows = records.value.flatMap((record) => {
    const parent = new Parent({ journal: props.journal, record });
    const children = record.items
      .map((item) => {
        const account = accountMap.get(`${ACCOUNT_TYPE}:${item.account}`);
        if (!account) {
          return null;
        }
        return new Child({ parent, recordItem: item, account });
      })
      .filter((item): item is Child => !!item);
    parent.children = children;
    return [parent, ...children];
  });
  return newRows.sort((a, b) => a.compare(b));
};

watch([accounts, records], () => {
  rows.value = generateRows();
});

watch(rows, (newRows) => {
  if (!newRows) {
    return;
  }

  const elHeight = tableRef.value?.clientHeight ?? height.value;

  const expandedId = new Set<string>();
  gridApi.value?.forEachNode((row) => {
    if (row.expanded && row.data?.id) {
      expandedId.add(row.data.id);
    }
  });

  gridApi.value?.setRowData(newRows);
  gridApi.value?.forEachNode((row) => {
    if (row.data?.id && expandedId.has(row.data.id)) {
      row.setExpanded(true);
    }
  });

  height.value = elHeight;
});

const getDataPath = (data: Row) => data.dataPath;

const saveEditedRows = async () => {
  const commands = rows.value
    .filter((row): row is Parent => row instanceof Parent && row.isEdited)
    .map((row) => row.generateCommand());
  const update: RecordCommandUpdate[] = [];
  const create: RecordCommandCreate[] = [];
  for (const command of commands) {
    if (!command) {
      continue;
    } else if (command.commandType === "records:create") {
      create.push(command);
    } else {
      update.push(command);
    }
  }
  console.log("RecordCommandBatchUpdate: ", { create, update });
  const result = await recordApi.handleCommand({
    commandType: "records:batchUpdate",
    update,
    create,
  });
  console.log("  Result: ", result);
};

const getContextMenuItems: GetContextMenuItems<Row> = (_params) => {
  return ["copy"];
};
</script>

<style scoped lang="scss">
.ag-theme-alpine,
.ag-theme-alpine-dark {
  padding: 6px;
  width: 100%;
  resize: vertical;
  overflow: auto;
}
</style>

<style lang="scss">
.cell {
  &-edited {
    border: 1px dashed rgba(var(--v-theme-primary)) !important;
  }
}
</style>
