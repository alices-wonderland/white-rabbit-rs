<template>
  <div class="flex flex-col gap-2">
    <div>
      <v-btn
        v-if="Object.keys(errors).length > 0"
        icon
        color="error"
        size="x-small"
        variant="text"
      >
        <v-icon class="animate-pulse" :icon="mdiAlertCircleOutline"></v-icon>
        <v-tooltip activator="parent" location="end">
          <code>
            <pre>{{ JSON.stringify(errors, null, 2) }}</pre>
          </code>
        </v-tooltip>
      </v-btn>
      <v-btn :prepend-icon="mdiDisc" :disabled="Object.keys(errors).length > 0">
        Save
      </v-btn>
    </div>

    <ag-grid-vue
      class="flex-1 ag-grid-vue ag-theme-alpine"
      :column-defs="columnDefs"
      :default-col-def="defaultColDef"
      :row-data="rows"
      :tree-data="true"
      :row-class-rules="roleClassRules"
      :get-data-path="getDataPath"
      :auto-group-column-def="autoGroupColumnDef"
      undo-redo-cell-editing
      :undo-redo-cell-editing-limit="20"
      @first-data-rendered="onFirstDataRendered"
      @cell-value-changed="onCellValueChanged"
    >
    </ag-grid-vue>
  </div>
</template>

<script lang="ts" setup>
import { reactive, ref } from "vue";
import { AgGridVue } from "@ag-grid-community/vue3";
import { Record_, RecordType, Journal, Account } from "@shared/models";
import {
  CellValueChangedEvent,
  ColDef,
  EditableCallbackParams,
  FirstDataRenderedEvent,
  GridApi,
  ICellEditorParams,
  IRowNode,
  RowClassParams,
  RowClassRules,
  ValueFormatterParams,
  ValueGetterParams,
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
import RecordWriteTableActionsCell from "./RecordWriteTableActionsCell.vue";
import { SharedError } from "@shared/error";
import { mdiDisc, mdiAlertCircleOutline } from "@mdi/js";

const EDITED_CLASS = "table-hint table-hint-edited";
const ERROR_CLASS = "table-hint table-hint-error";
const NOT_EDITABLE_CLASS = "table-hint table-hint-not-editable";

const props = defineProps<{
  records: Record_[];
}>();

const records = ref<Record_[]>(props.records);

const accountApi = useAccountApi();
const journalApi = useJournalApi();
const gridApi = ref<GridApi>();

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
    headerName: "Actions",
    cellRenderer: RecordWriteTableActionsCell,
    pinned: "left",
    lockPinned: true,
  },
  {
    field: "data.type",
    headerName: "Type",
    editable: (params: EditableCallbackParams<Row>) =>
      (params.data?.isEditable ?? false) &&
      params.data?.rowType === RowType.RECORD,
    cellEditor: "agSelectCellEditor",
    cellEditorParams: (): { values: RecordType[] } => {
      return {
        values: [RecordType.RECORD, RecordType.CHECK],
      };
    },
    cellClassRules: {
      [EDITED_CLASS]: (params) => params.value !== params.data?.snapshot?.type,
    },
  },
  {
    field: "data.date",
    headerName: "Date",
    valueFormatter: (
      params: ValueFormatterParams<RecordRow, Date | undefined>
    ): string => (params.value ? format(params.value, DATE_FORMAT) : ""),
    editable: (params: EditableCallbackParams<Row>) =>
      (params.data?.isEditable ?? false) &&
      params.data?.rowType === RowType.RECORD,
    cellEditor: RecordWriteTableDateCell,
    cellClassRules: {
      [EDITED_CLASS]: (params) => params.value !== params?.data?.snapshot?.date,
    },
  },
  {
    field: "data.journal",
    headerName: "Journal",
    editable: (params: EditableCallbackParams<Row>) =>
      (params.data?.isEditable ?? false) &&
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
    cellClassRules: {
      [EDITED_CLASS]: (params) =>
        params.value?.id !== params.data?.snapshot?.journal?.id,
    },
  },
  {
    field: "data.amount",
    headerName: "Amount",
    editable: (params: EditableCallbackParams<Row>) =>
      (params.data?.isEditable ?? false) &&
      params.data?.rowType === RowType.ITEM,
    cellClassRules: {
      [EDITED_CLASS]: (params) =>
        params.value !== params.data?.snapshot?.amount,
    },
  },
  {
    field: "data.price",
    headerName: "Price",
    editable: (params: EditableCallbackParams<Row>) =>
      (params.data?.isEditable ?? false) &&
      params.data?.rowType === RowType.ITEM,
    cellClassRules: {
      [EDITED_CLASS]: (params) => params.value !== params.data?.snapshot?.price,
    },
  },
  {
    field: "data.tags",
    headerName: "Tags",
    editable: (params: EditableCallbackParams<Row>) =>
      (params.data?.isEditable ?? false) &&
      params.data?.rowType === RowType.RECORD,
    cellRenderer: RecordReadTableTagCell,
    cellEditor: RecordWriteTableTagCell,
    cellClassRules: {
      [EDITED_CLASS]: (params) => params.value !== params.data?.snapshot?.tags,
    },
  },
  {
    field: "data.description",
    headerName: "Description",
    editable: (params: EditableCallbackParams<Row>) =>
      (params.data?.isEditable ?? false) &&
      params.data?.rowType === RowType.RECORD,
    cellEditorPopup: true,
    cellEditor: "agLargeTextCellEditor",
    cellClassRules: {
      [EDITED_CLASS]: (params) =>
        params.value !== params.data?.snapshot?.description,
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

  valueGetter: (params: ValueGetterParams<Row>): string | undefined => {
    if (params.data instanceof RecordRow) {
      return params.data.data.name;
    } else {
      return params.data?.data.account?.id;
    }
  },
  // valueSetter can handle the async result well.
  //
  // eslint-disable-next-line @typescript-eslint/ban-ts-comment
  // @ts-ignore:next-line
  valueSetter: async (params) => {
    let parent: IRowNode<Row> | undefined | null;
    if (params.data instanceof RecordRow) {
      params.data.data.name = params.newValue;
      parent = params.node;
    } else if (params.data instanceof RecordItemRow) {
      let account: Account = params.newValue;
      parent = params.node?.parent;
      if (typeof params.newValue === "string") {
        try {
          const result = await accountApi.findById(params.newValue);
          if (!result) {
            console.log("Account not found: ", params.newValue);
            return false;
          }
          account = result;
        } catch (e) {
          console.log("Account not found: ", e);
          return false;
        }
      }
      params.data.hierarchy[1] = account.id;
      params.data.data.account = account;
    }

    if (parent) {
      const children = parent?.childrenAfterGroup ?? [];
      params.api.refreshCells({ rowNodes: [parent, ...children] });
    }
    return true;
  },

  editable: (params) => params.data?.isEditable ?? false,
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

  cellClassRules: {
    [ERROR_CLASS]: (params) =>
      (params.data?.errors(params.node)?.get("account")?.length ?? 0) > 0,
    [EDITED_CLASS]: (params) =>
      (params.data instanceof RecordRow &&
        params.data.data.name !== params.data.snapshot.name) ||
      (params.data instanceof RecordItemRow &&
        params.data.data.account?.id !== params.data.snapshot.account?.id),
  },
};

const roleClassRules: RowClassRules = {
  [NOT_EDITABLE_CLASS]: (params: RowClassParams<Row>) =>
    !params.data?.isEditable,
};

const getDataPath = (data: Row) => data.hierarchy;

const onFirstDataRendered = (event: FirstDataRenderedEvent) => {
  event.columnApi.autoSizeAllColumns();
  gridApi.value = event.api;
};

const errors = reactive<Record<string, Record<string, SharedError[]>>>({});

const onCellValueChanged = (event: CellValueChangedEvent<Row>) => {
  const parent =
    event.data instanceof RecordRow ? event.node : event.node.parent;
  if (!parent) {
    return;
  }

  for (const node of [parent, ...(parent.childrenAfterGroup ?? [])]) {
    const error = node.data?.errors(node);
    if (error) {
      errors[node.id ?? ""] = Object.fromEntries([...error]);
    } else {
      delete errors[node.id ?? ""];
    }
  }
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

  &.table-hint-not-editable {
    .ag-group-value {
      font-style: italic;
      text-decoration: line-through;
    }
  }
}
</style>
