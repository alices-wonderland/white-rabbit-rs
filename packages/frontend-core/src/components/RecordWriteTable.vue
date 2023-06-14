<template>
  <v-text-field v-model="date" type="datetime-local"> </v-text-field>
  <ag-grid-vue
    :style="{ minHeight: '150px', maxHeight: '80vh', height }"
    :class="theme.global.name.value === 'dark' ? 'ag-theme-alpine-dark' : 'ag-theme-alpine'"
    animate-rows="true"
    :column-defs="columnDefs"
    :default-col-def="defaultColDef"
    :auto-group-column-def="autoGroupColumnDef"
    tree-data="true"
    :get-data-path="getDataPath"
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
} from "@core/services";
import { computed, ref, watch } from "vue";
import { useTheme } from "vuetify";
import RecordWriteTableActionsCellRenderer from "./RecordWriteTableActionsCellRenderer.vue";
import RecordWriteTableGroupCellRenderer from "./RecordWriteTableGroupCellRenderer.vue";
import RecordWriteTableStateCellRenderer from "./RecordWriteTableStateCellRenderer.vue";
import { Child, Parent, type Row } from "./row";

const theme = useTheme();

const date = ref<string>("2022-03-31T00:11:22");

const rows = ref<Row[]>([]);
const recordApi = useInject<RecordApi>(RECORD_API_KEY);
const journalApi = useInject<JournalApi>(JOURNAL_API_KEY);
const gridApi = ref<GridApi>();
const columnApi = ref<ColumnApi>();

const columnDefs = ref<ColDef[]>([
  { headerName: "Actions", cellRenderer: RecordWriteTableActionsCellRenderer, pinned: "left" },
  {
    headerName: "Date",
    field: "date",
    editable: (params: EditableCallbackParams<Row, Date>) => params.data instanceof Parent,
  },
  {
    headerName: "Type",
    valueGetter: (params: ValueGetterParams<Row>) =>
      params.data instanceof Parent ? params.data.type : params.data?.account?.type,
    editable: (params: EditableCallbackParams<Row, RecordType>) => params.data instanceof Parent,
  },
  {
    headerName: "State",
    cellRenderer: RecordWriteTableStateCellRenderer,
  },
  {
    headerName: "Amount",
    field: "amount",
    editable: (params: EditableCallbackParams<Row, number>) => params.data instanceof Child,
  },
  {
    headerName: "Price",
    field: "price",
    editable: (params: EditableCallbackParams<Row, number>) => params.data instanceof Child,
  },
]);

const defaultColDef: ColDef = {
  flex: 1,
  minWidth: 100,
  sortable: true,
  filter: true,
  resizable: true,
  suppressMovable: true,
};

const autoGroupColumnDef: ColDef = {
  headerName: "Record",
  editable: (params: EditableCallbackParams<Row, number>) => params.data instanceof Parent,
  cellRendererParams: {
    innerRenderer: RecordWriteTableGroupCellRenderer,
    suppressDoubleClickExpand: true,
  },
  valueSetter(params: ValueSetterParams<Row>): boolean {
    if (params.data instanceof Parent) {
      params.data.name = params.newValue;
      return true;
    }
    return false;
  },
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
