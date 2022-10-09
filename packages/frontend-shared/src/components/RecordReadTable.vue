<template>
  <ag-grid-vue
    class="ag-theme-alpine"
    :column-defs="columnDefs"
    :default-col-def="defaultColDef"
    :row-data="rows"
    :tree-data="true"
    :get-data-path="getDataPath"
    :auto-group-column-def="autoGroupColumnDef"
    @first-data-rendered="onFirstDataRendered"
  >
  </ag-grid-vue>
</template>

<script lang="ts" setup>
import { ref } from "vue";
import { AgGridVue } from "@ag-grid-community/vue3";
import {
  Record_,
  RecordStateItem,
  RecordType,
  AccountType,
} from "@shared/models";
import { ColDef, FirstDataRenderedEvent } from "@ag-grid-community/core";
import { computedAsync } from "@vueuse/core";
import RecordReadTableTagCell from "./RecordReadTableTagCell.vue";
import RecordTableGroupCell from "./RecordTableGroupCell.vue";
import RecordReadTableStateCell from "./RecordReadTableStateCell.vue";
import { useAccountApi, useJournalApi } from "@shared/hooks";

type RowData = {
  hierarchy: string[];
  name?: string;
  type?: RecordType | AccountType;
  journal?: string;
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

const accountApi = useAccountApi();

const journalApi = useJournalApi();

const rows = computedAsync<RowData[]>(
  async () => {
    const accounts = await accountApi.findAll({
      query: {
        id: [
          ...new Set(
            props.records.flatMap((record) =>
              [...record.items].map((item) => item.accountId)
            )
          ),
        ],
        includeArchived: true,
      },
    });

    const accountTypes = Object.fromEntries(
      accounts.map((account) => [account.id, account.type])
    );

    const journals = await journalApi.findAll({
      query: {
        id: [...new Set(props.records.map((record) => record.journalId))],
        includeArchived: true,
      },
    });

    const journalNames = Object.fromEntries(
      journals.map((journal) => [journal.id, journal.name])
    );

    return props.records.flatMap((record) => {
      const row: RowData = {
        hierarchy: [record.id],
        name: record.name,
        type: record.type,
        journal: journalNames[record.journalId],
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
  { field: "journal" },
  { field: "tags", cellRenderer: RecordReadTableTagCell },
  { field: "state", cellRenderer: RecordReadTableStateCell },
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

const onFirstDataRendered = (event: FirstDataRenderedEvent) => {
  event.columnApi.autoSizeAllColumns();
};
</script>
