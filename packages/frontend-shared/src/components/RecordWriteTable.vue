<template>
  <ag-grid-vue
    class="ag-theme-alpine"
    :column-defs="columnDefs"
    :default-col-def="defaultColDef"
    :row-data="rows"
    :tree-data="true"
    :get-data-path="getDataPath"
    :auto-group-column-def="autoGroupColumnDef"
    :get-row-id="getRowId"
    @first-data-rendered="onFirstDataRendered"
  >
  </ag-grid-vue>
</template>

<script lang="ts" setup>
import { ref } from "vue";
import { AgGridVue } from "@ag-grid-community/vue3";
import { Record_, RecordType } from "@shared/models";
import {
  ColDef,
  EditableCallbackParams,
  FirstDataRenderedEvent,
  GetRowIdParams,
  ICellEditorParams,
  IRichCellEditorParams,
  ValueFormatterParams,
} from "@ag-grid-community/core";
import { computedAsync } from "@vueuse/core";
import { invoke } from "@tauri-apps/api/tauri";
import RecordWriteTableTagCell from "./RecordWriteTableTagCell.vue";
import RecordTableGroupCell from "./RecordTableGroupCell.vue";
import RecordWriteTableDateCell from "./RecordWriteTableDateCell.vue";
import RecordReadTableTagCell from "./RecordReadTableTagCell.vue";
import RecordWriteTableNameCell from "./RecordWriteTableNameCell.vue";
import RecordWriteTableAccountCell from "./RecordWriteTableAccountCell.vue";

type RowData = {
  hierarchy: string[];
  name?: string;
  type?: RecordType;
  journal?: string;
  date?: Date;
  tags?: Set<string>;
  account?: string;
  description?: string;
};

const props = defineProps<{
  records: Record_[];
}>();

const records = ref<Record_[]>(props.records);

const userId = computedAsync<string>(
  async () => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const users = await invoke<any[]>("get_users", {
      input: {
        query: { role: "Owner" },
        sort: { field: "date", order: "Desc" },
      },
    });
    return users[0].id;
  },
  "",
  {
    onError: (e) => console.error("Error when loading users: ", e),
  }
);

const journalNames = computedAsync<Record<string, string>>(
  async () => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const journals: any[] = await invoke("get_journals", {
      operator: userId.value,
      input: {
        query: {
          id: [...new Set(records.value.map((record) => record.journalId))],
          includeArchived: true,
        },
      },
    });

    return Object.fromEntries(
      journals.map((journal) => [journal.id, journal.name])
    );
  },
  {},
  {
    onError: (e) => console.error("Error when loading journals: ", e),
  }
);

const rows = computedAsync<RowData[]>(
  async () => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const accounts: any[] = await invoke("get_accounts", {
      operator: userId.value,
      input: {
        query: {
          id: [
            ...new Set(
              records.value.flatMap((record) =>
                [...record.items].map((item) => item.accountId)
              )
            ),
          ],
          includeArchived: true,
        },
      },
    });

    const accountTypes = Object.fromEntries(
      accounts.map((account) => [account.id, account.type])
    );

    return records.value.flatMap((record) => {
      const row: RowData = {
        hierarchy: [record.id],
        name: record.name,
        type: record.type,
        journal: record.journalId,
        date: record.date,
        tags: record.tags,
        description: record.description,
      };
      const rows: RowData[] = [...record.items].map<RowData>((item) => ({
        hierarchy: [record.id, item.accountId],
        recordType: record.type,
        type: accountTypes[item.accountId],
        amount: item.amount,
        price: item.price,
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
  {
    field: "type",
    editable: (params: EditableCallbackParams) => {
      return params.data.hierarchy.length === 1;
    },
    cellEditor: "agSelectCellEditor",
    cellEditorParams: (): { values: RecordType[] } => {
      return {
        values: [RecordType.RECORD, RecordType.CHECK],
      };
    },
  },
  {
    field: "date",
    editable: (params: EditableCallbackParams) => {
      return params.data.hierarchy.length === 1;
    },
    cellEditor: RecordWriteTableDateCell,
  },
  {
    field: "journal",
    editable: (params: EditableCallbackParams) => {
      return params.data.hierarchy.length === 1;
    },
    valueFormatter: (params: ValueFormatterParams) =>
      journalNames.value[params.value],
    cellEditor: "agRichSelectCellEditor",
    cellEditorParams: (): Partial<IRichCellEditorParams> => {
      return {
        values: Object.keys(journalNames.value),
        formatValue: (value) => journalNames.value[value],
      };
    },
  },
  {
    field: "amount",
    editable: (params: EditableCallbackParams) => {
      return params.data.hierarchy.length === 2;
    },
  },
  {
    field: "price",
    editable: (params: EditableCallbackParams) => {
      return params.data.hierarchy.length === 2;
    },
  },
  {
    field: "tags",
    editable: (params: EditableCallbackParams) => {
      return params.data.hierarchy.length === 1;
    },
    cellRenderer: RecordReadTableTagCell,
    cellEditor: RecordWriteTableTagCell,
  },
  {
    field: "description",
    editable: (params: EditableCallbackParams) => {
      return params.data.hierarchy.length === 1;
    },
    cellEditorPopup: true,
    cellEditor: "agLargeTextCellEditor",
  },
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
    innerRendererParams: {
      plainText: true,
    },
    suppressDoubleClickExpand: true,
  },

  valueSetter: (params) => {
    if (params.data.hierarchy.length === 1) {
      params.data.name = params.newValue;
    } else if (params.data.hierarchy.length === 2) {
      const journalId: string | undefined = params.node?.parent?.data?.journal;
      if (
        !params.newValue ||
        !journalId ||
        params.newValue.journalId !== journalId
      ) {
        return false;
      }

      params.data.hierarchy[1] = params.newValue.id;
      params.data.type = params.newValue.type;
    }

    if (params.node) {
      params.api.refreshCells({ rowNodes: [params.node] });
    }
    return true;
  },

  editable: true,
  cellEditorSelector: (params: ICellEditorParams) => {
    if (params.data.hierarchy.length === 2) {
      return {
        component: RecordWriteTableAccountCell,
        params: {
          userId: userId.value,
        },
      };
    } else {
      return {
        component: RecordWriteTableNameCell,
      };
    }
  },
};

const getDataPath = (data: RowData) => data.hierarchy;

const onFirstDataRendered = (event: FirstDataRenderedEvent) => {
  event.columnApi.autoSizeAllColumns();
};

const getRowId = (params: GetRowIdParams<RowData>) =>
  params.data.hierarchy.join(":");
</script>
