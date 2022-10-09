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
import { Record_, RecordType, AccountType } from "@shared/models";
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
import RecordWriteTableTagCell from "./RecordWriteTableTagCell.vue";
import RecordTableGroupCell from "./RecordTableGroupCell.vue";
import RecordWriteTableDateCell from "./RecordWriteTableDateCell.vue";
import RecordReadTableTagCell from "./RecordReadTableTagCell.vue";
import RecordWriteTableNameCell from "./RecordWriteTableNameCell.vue";
import RecordWriteTableAccountCell from "./RecordWriteTableAccountCell.vue";
import { useAccountApi, useJournalApi } from "@shared/hooks";

type RowData = {
  hierarchy: string[];
  name?: string;
  type?: RecordType | AccountType;
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

const accountApi = useAccountApi();
const journalApi = useJournalApi();

const journalNames = computedAsync<Record<string, string>>(
  async () => {
    const journals = await journalApi.findAll({
      query: {
        id: [...new Set(records.value.map((record) => record.journalId))],
        includeArchived: true,
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
    const accounts = await accountApi.findAll({
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
