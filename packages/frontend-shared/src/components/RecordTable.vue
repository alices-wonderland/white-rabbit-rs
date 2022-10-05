<template>
  <ag-grid-vue
    class="ag-theme-alpine"
    :column-defs="columnDefs"
    :default-col-def="defaultColDef"
    :row-data="rows"
    :tree-data="true"
    :get-data-path="getDataPath"
    :auto-group-column-def="autoGroupColumnDef"
  >
  </ag-grid-vue>
</template>

<script lang="ts" setup>
import { ref } from "vue";
import { AgGridVue } from "@ag-grid-community/vue3";
import { Record_, RecordStateItem, RecordType } from "@shared/models";
import { ColDef } from "@ag-grid-community/core";
import {
  RecordTableGroupCell,
  RecordTableStateCell,
  RecordTableTagCell,
} from "@shared/components";
import { computedAsync } from "@vueuse/core";
import { invoke } from "@tauri-apps/api/tauri";

type RowData = {
  hierarchy: string[];
  name?: string;
  type?: RecordType;
  date?: Date;
  tags?: Set<string>;
  account?: string;
  state?: RecordStateItem;
  description?: string;
};

const props = defineProps<{
  records: Record_[];
  editable: boolean;
}>();

const rows = computedAsync<RowData[]>(
  async () => {
    const users = await invoke<any[]>("get_users", {
      input: {
        query: { role: "Owner" },
        sort: { field: "date", order: "Desc" },
      },
    });

    console.log("props.records: ", props.records);
    const query = {
      id: [
        ...new Set(
          props.records.flatMap((record) =>
            [...record.items].map((item) => item.accountId)
          )
        ),
      ],
      includeArchived: true,
    };
    console.log("account query: ", query);
    const accounts: any[] = await invoke("get_accounts", {
      operator: users[0].id,
      input: { query },
    });

    const accountTypes = Object.fromEntries(
      accounts.map((account) => [account.id, account.type])
    );

    return props.records.flatMap((record) => {
      const row: RowData = {
        hierarchy: [record.id],
        name: record.name,
        type: record.type,
        date: record.date,
        tags: record.tags,
        state: record.state instanceof Array ? record.state : undefined,
        description: record.description,
      };
      const rows: RowData[] = [...record.items].map<RowData>((item) => ({
        hierarchy: [record.id, item.accountId],
        recordType: record.type,
        type: accountTypes[item.accountId],
        state: (record.state as Record<string, RecordStateItem>)[
          item.accountId
        ] ?? [item.amount, item.price],
      }));

      return [row, ...rows];
    });
  },
  undefined,
  {
    onError: (e) => console.error("Error when loading records: ", e),
  }
);

const columnDefs = ref<ColDef[]>([
  { field: "type" },
  { field: "date" },
  { field: "tags", cellRenderer: RecordTableTagCell },
  { field: "state", cellRenderer: RecordTableStateCell },
  { field: "description" },
]);

const defaultColDef: ColDef = {
  flex: 1,
  minWidth: 100,
  resizable: true,
  suppressMovable: true,
};

const autoGroupColumnDef: ColDef = {
  headerName: "Records",
  menuTabs: ["filterMenuTab", "generalMenuTab", "columnsMenuTab"],
  cellRendererParams: {
    innerRenderer: RecordTableGroupCell,
  },
};

const getDataPath = (data: RowData) => data.hierarchy;
</script>

<style lang="scss">
@use "@ag-grid-community/styles" as ag;
@include ag.grid-styles(
  (
    themes: (
      alpine,
      alpine-dark,
    ),
  )
);
</style>
