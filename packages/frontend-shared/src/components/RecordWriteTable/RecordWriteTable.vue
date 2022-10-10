<template>
  <ag-grid-vue
    class="ag-grid-vue ag-theme-alpine"
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
import { Record_, RecordType, Journal } from "@shared/models";
import {
  CellClassParams,
  ColDef,
  EditableCallbackParams,
  FirstDataRenderedEvent,
  ICellEditorParams,
  RowNode,
  ValueFormatterParams,
  ValueSetterParams,
} from "@ag-grid-community/core";
import { computedAsync } from "@vueuse/core";
import RecordWriteTableTagCell from "./RecordWriteTableTagCell.vue";
import RecordTableGroupCell from "../RecordTableGroupCell.vue";
import RecordWriteTableDateCell from "./RecordWriteTableDateCell.vue";
import RecordReadTableTagCell from "../RecordReadTableTagCell.vue";
import RecordWriteTableNameCell from "./RecordWriteTableNameCell.vue";
import RecordWriteTableAccountCell from "./RecordWriteTableAccountCell.vue";
import { useAccountApi, useJournalApi } from "@shared/hooks";
import { Row, RecordRow, RecordItemRow, RowType, DATE_FORMAT } from "./types";
import { format } from "date-fns";

const EDITED_CLASS = "table-hint table-hint-edited";
const ERROR_CLASS = "table-hint table-hint-error";

const props = defineProps<{
  records: Record_[];
}>();

const records = ref<Record_[]>(props.records);

const accountApi = useAccountApi();
const journalApi = useJournalApi();

const journals = computedAsync<Record<string, Journal>>(
  async () => {
    const journals = await journalApi.findAll({
      query: {
        id: [...new Set(records.value.map((record) => record.journalId))],
        includeArchived: true,
      },
    });

    return Object.fromEntries(journals.map((journal) => [journal.id, journal]));
  },
  {},
  {
    onError: (e) => console.error("Error when loading journals: ", e),
  }
);

const rows = computedAsync<Row[]>(
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

    const accountMap = Object.fromEntries(
      accounts.map((account) => [account.id, account])
    );

    return records.value.flatMap((record) => {
      const row = new RecordRow(record, journals.value[record.journalId]);
      const rows: RecordItemRow[] = [...record.items].map(
        (item) => new RecordItemRow(item, record, accountMap[item.accountId])
      );

      return [row, ...rows];
    });
  },
  undefined,
  {
    onError: (e) => console.error("Error when loading records: ", e),
  }
);

// TODO: For error and edited classes, should integrate the logic into row class, rather than scattered everywhere
const columnDefs = ref<ColDef[]>([
  {
    field: "data.type",
    headerName: "Type",
    editable: (params: EditableCallbackParams<Row>) =>
      params.data?.rowType === RowType.RECORD,
    cellEditor: "agSelectCellEditor",
    cellEditorParams: (): { values: RecordType[] } => {
      return {
        values: [RecordType.RECORD, RecordType.CHECK],
      };
    },
    cellClass: (params: CellClassParams<Row>) => {
      if (params.value !== params.data?.snapshot?.type) {
        return EDITED_CLASS;
      }
      return undefined;
    },
  },
  {
    field: "data.date",
    headerName: "Date",
    valueFormatter: (
      params: ValueFormatterParams<RecordRow, Date | undefined>
    ): string => (params.value ? format(params.value, DATE_FORMAT) : ""),
    editable: (params: EditableCallbackParams<Row>) =>
      params.data?.rowType === RowType.RECORD,
    cellEditor: RecordWriteTableDateCell,
    cellClass: (params: CellClassParams<RecordRow, Date>) => {
      if (params.value !== params?.data?.snapshot?.date) {
        return EDITED_CLASS;
      }
      return undefined;
    },
  },
  {
    field: "data.journal",
    headerName: "Journal",
    editable: (params: EditableCallbackParams<Row>) =>
      params.data?.rowType === RowType.RECORD,
    valueFormatter: (params: ValueFormatterParams<Row, Journal | undefined>) =>
      params.value?.name ?? "",
    valueSetter: (params: ValueSetterParams<RecordRow>) => {
      params.data.data.journal = params.newValue;
      if (params.node) {
        params.api.refreshCells({
          force: true,
          rowNodes: params.node.allLeafChildren,
        });
      }
      return true;
    },
    cellEditor: "agRichSelectCellEditor",
    cellEditorParams: () => {
      return {
        values: Object.values(journals.value).sort((a, b) =>
          a.name.localeCompare(b.name)
        ),
        formatValue: (journal: Journal) => journal.name,
      };
    },
    cellClass: (params: CellClassParams<RecordRow, Journal | undefined>) => {
      if (params.value?.id !== params.data?.snapshot?.journal?.id) {
        return EDITED_CLASS;
      }
      return undefined;
    },
  },
  {
    field: "data.amount",
    headerName: "Amount",
    editable: (params: EditableCallbackParams<Row>) =>
      params.data?.rowType === RowType.ITEM,
    cellClass: (params: CellClassParams<RecordItemRow, number>) => {
      if (params.value !== params.data?.snapshot?.amount) {
        return EDITED_CLASS;
      }
      return undefined;
    },
  },
  {
    field: "data.price",
    headerName: "Price",
    editable: (params: EditableCallbackParams<Row>) =>
      params.data?.rowType === RowType.ITEM,
    cellClass: (params: CellClassParams<RecordItemRow, number>) => {
      if (params.value !== params.data?.snapshot?.price) {
        return EDITED_CLASS;
      }
      return undefined;
    },
  },
  {
    field: "data.tags",
    headerName: "Tags",
    editable: (params: EditableCallbackParams<Row>) =>
      params.data?.rowType === RowType.RECORD,
    cellRenderer: RecordReadTableTagCell,
    cellEditor: RecordWriteTableTagCell,
    cellClass: (params: CellClassParams<RecordRow, Set<string>>) => {
      if (params.value !== params.data?.snapshot?.tags) {
        return EDITED_CLASS;
      }
      return undefined;
    },
  },
  {
    field: "data.description",
    headerName: "Description",
    editable: (params: EditableCallbackParams<Row>) =>
      params.data?.rowType === RowType.RECORD,
    cellEditorPopup: true,
    cellEditor: "agLargeTextCellEditor",
    cellClass: (params: CellClassParams<RecordRow, string>) => {
      if (params.value !== params.data?.snapshot?.description) {
        return EDITED_CLASS;
      }
      return undefined;
    },
  },
]);

const defaultColDef: ColDef = {
  flex: 1,
  minWidth: 100,
  resizable: true,
  suppressMovable: true,
};

const autoGroupColumnDef: ColDef<Row> = {
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
    if (params.data instanceof RecordRow) {
      params.data.data.name = params.newValue;
    } else if (params.data instanceof RecordItemRow) {
      params.data.hierarchy[1] = params.newValue.id;
      params.data.data.account = params.newValue;
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

  cellClass: (params: CellClassParams<Row>) => {
    const classes = [];

    let isEdited = false;
    if (params.data instanceof RecordRow) {
      isEdited = params.data.data.name !== params.data.snapshot.name;
    } else if (params.data instanceof RecordItemRow) {
      isEdited = params.data.data.account !== params.data.snapshot.account;
    }

    if (isEdited) {
      classes.push(EDITED_CLASS);
    }

    let accountJournalMismatch =
      params.data instanceof RecordItemRow &&
      params.data.data.account?.journalId !==
        (params.node.parent as unknown as RowNode<RecordRow>).data?.data
          ?.journal?.id;

    if (accountJournalMismatch) {
      classes.push(ERROR_CLASS);
    }

    return classes;
  },
};

const getDataPath = (data: Row) => data.hierarchy;

const onFirstDataRendered = (event: FirstDataRenderedEvent) => {
  event.columnApi.autoSizeAllColumns();
};
</script>

<style scoped lang="scss">
.ag-grid-vue :deep(.table-hint) {
  @apply border border-dashed;

  &.table-hint-edited {
    border-color: rgb(var(--v-theme-info));
  }

  &.table-hint-error {
    border-color: rgb(var(--v-theme-error));
  }
}
</style>
