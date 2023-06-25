<template>
  <v-btn color="primary" size="small" @click="saveEditedRows">Save</v-btn>
  <ag-grid-vue
    :style="{ minHeight: '150px', maxHeight: '80vh', height }"
    :class="theme.global.name.value === 'dark' ? 'ag-theme-alpine-dark' : 'ag-theme-alpine'"
    :column-defs="columnDefs"
    :default-col-def="defaultColDef"
    group-display-type="custom"
    :get-data-path="getDataPath"
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
  ColumnApi,
  GridApi,
} from "@ag-grid-community/core";
import { AgGridVue } from "@ag-grid-community/vue3";
import { useInject } from "@core/composable";
import {
  type JournalApi,
  type RecordApi,
  type RecordType,
  Account,
  Journal,
  ACCOUNT_TYPE,
  JOURNAL_API_KEY,
  JOURNAL_TYPE,
  RECORD_API_KEY,
  type AccountApi,
  ACCOUNT_API_KEY,
} from "@core/services";
import { computed, ref, watch } from "vue";
import { useTheme } from "vuetify";
import RecordWriteTableActionsCellRenderer from "./RecordWriteTableActionsCellRenderer.vue";
import RecordWriteTableGroupCellRenderer from "./RecordWriteTableGroupCellRenderer.vue";
import RecordWriteTableStateCellRenderer from "./RecordWriteTableStateCellRenderer.vue";
import { Child, Parent, type Row } from "./row";

const theme = useTheme();

const rows = ref<Row[]>([]);
const recordApi = useInject<RecordApi>(RECORD_API_KEY);
const journalApi = useInject<JournalApi>(JOURNAL_API_KEY);
const accountApi = useInject<AccountApi>(ACCOUNT_API_KEY);
const gridApi = ref<GridApi>();
const columnApi = ref<ColumnApi>();

const columnDefs = ref<AbstractColDef[]>([
  {
    headerName: "Actions",
    cellRenderer: RecordWriteTableActionsCellRenderer,
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
        editable: (params: EditableCallbackParams<Row>) => params.data instanceof Parent,
        valueGetter: (params: ValueGetterParams<Row>) =>
          params.data instanceof Parent ? params.data.name : params.data?.account?.id,
        valueSetter(params: ValueSetterParams<Row>): boolean {
          if (params.data instanceof Parent) {
            params.data.name = params.newValue;
            return true;
          }
          return false;
        },
        filter: true,
        showRowGroup: true,
        filterParams: {
          valueGetter: (params: ValueGetterParams<Row>) => params.data?.name,
        },
        cellClassRules: {
          "cell cell-edited": (params: CellClassParams<Row>) =>
            params.data?.editedFields?.includes("name"),
        },
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
        params.data?.editedFields?.includes("date") ?? false,
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
      "cell cell-edited": (params: CellClassParams<Row>) =>
        params.data?.editedFields?.includes("type"),
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
        params.data?.editedFields?.includes("amount") ?? false,
    },
  } as ColDef,
  {
    headerName: "Price",
    field: "price",
    sortable: false,
    editable: (params: EditableCallbackParams<Row, number>) => params.data instanceof Child,
    cellEditor: "agNumberCellEditor",
    cellClassRules: {
      "cell cell-edited": (params: CellClassParams<Row>) =>
        params.data?.editedFields?.includes("price") ?? false,
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
};

const onGridReady = async (params: GridReadyEvent) => {
  const journals = await journalApi.findAll({ query: { name: ["Journal 1", false] } });
  const records = await recordApi.findAll(
    { query: { journal: journals[0].map((j) => j.id) } },
    true
  );
  const newRows = records[0].flatMap((record) => {
    const journal = records[1].get(`${JOURNAL_TYPE}:${record.journal}`) as Journal;
    const parent = new Parent(journal, record);
    const children = record.items.map(
      (item) =>
        new Child(parent, item, records[1].get(`${ACCOUNT_TYPE}:${item.account}`) as Account)
    );
    parent.children = children;
    return [parent, ...children];
  });
  rows.value = newRows.sort((a, b) => a.compare(b));
  gridApi.value = params.api;
  columnApi.value = params.columnApi;
};

const height = computed(() => `${rows.value.filter((row) => row instanceof Parent).length * 60}px`);

watch(rows, (newRows) => gridApi.value?.setRowData(newRows));

const getDataPath = (data: Row) => data.dataPath;

const saveEditedRows = async () => {
  const _editedRows = rows.value.filter((row) => row.editedFields.length > 0);
  const [journals, _included] = await journalApi.findAll({ query: { name: ["Journal 1", false] } });
  const [accounts, _aIncluded] = await accountApi.findAll({
    query: { journal: journals.map((j) => j.id) },
  });
  const result = await recordApi.handleCommand({
    commandType: "records:batchUpdate",
    create: [
      {
        journal: journals[0].id,
        name: "name 2",
        description: "description 1",
        type: "Record",
        date: "2023-01-31",
        tags: ["new 1", "old 2"],
        items: accounts.map((a) => ({
          account: a.id,
          amount: 321,
          price: 123,
        })),
      },
    ],
  });
  console.log("After save: ", result);
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
    background-color: rgba(var(--v-theme-primary), 0.2);
    color: var(--v-theme-on-background);
  }
}
</style>
