<script setup lang="ts">
import {
  AppTable,
  AppTableEditableCellRenderer,
  AppTableTagsCellRenderer,
  AppTableAccountCellRenderer,
} from "@core/components/AppTable";
import { Account, Entry } from "@core/services";
import { computed, onMounted, ref, shallowRef, triggerRef } from "vue";
import { ChildRow, createAll, ParentRow } from "./row";
import type { Row } from "./row";
import type { ColDef, ICellRendererParams } from "@ag-grid-community/core";

import EntryTableDateCellEditor from "./EntryTableDateCellEditor.vue";
import EntryTableStateCellRenderer from "./EntryTableStateCellRenderer.vue";

const props = defineProps<{
  readonly modelValue: Entry[];
  readonly accounts: Account[];
}>();

const readonly = ref(true);
const loading = ref(false);

const rows = shallowRef<Row[]>([]);

onMounted(() => {
  rows.value = createAll(props.modelValue);
});

const accountMap = computed(() => new Map(props.accounts.map((account) => [account.id, account])));

// eslint-disable-next-line sonarjs/cognitive-complexity
const columnDefs = computed((): ColDef<Row>[] => {
  const results: ColDef<Row>[] = [
    {
      headerName: "Description",
      width: 200,
      valueGetter: (params) => {
        if (params.data instanceof ParentRow) {
          return params.data.description;
        }
      },
      valueSetter: (params) => {
        if (params.data instanceof ParentRow && params.newValue) {
          params.data.description = params.newValue;
          triggerRef(rows);
          return true;
        }
        return false;
      },
      editable: (params) => params.data instanceof ParentRow && params.data.editable("description"),
      cellRendererSelector: (params: ICellRendererParams<Row>) => {
        if (params.data instanceof ParentRow) {
          return {
            component: AppTableEditableCellRenderer,
            params: {
              fieldState: params.data.getFieldState("description"),
            },
          };
        }
      },
    },
    {
      headerName: "Type",
      valueGetter: (params) => {
        if (params.data instanceof ParentRow) {
          return params.data.type;
        }
      },
      valueSetter: (params) => {
        if (params.data instanceof ParentRow && params.newValue) {
          params.data.type = params.newValue;
          triggerRef(rows);
          return true;
        }
        return false;
      },
      editable: (params) => params.data instanceof ParentRow && params.data.editable("type"),
      cellRendererSelector: (params: ICellRendererParams<Row>) => {
        if (params.data instanceof ParentRow) {
          return {
            component: AppTableEditableCellRenderer,
            params: {
              fieldState: params.data.getFieldState("type"),
            },
          };
        }
      },
    },
    {
      headerName: "Date",
      valueGetter: (params) => {
        if (params.data instanceof ParentRow) {
          return params.data.date;
        }
      },
      valueSetter: (params) => {
        if (params.data instanceof ParentRow && params.newValue) {
          params.data.date = params.newValue;
          triggerRef(rows);
          return true;
        }
        return false;
      },
      editable: (params) => params.data instanceof ParentRow && params.data.editable("date"),
      cellRendererSelector: (params: ICellRendererParams<Row>) => {
        if (params.data instanceof ParentRow) {
          return {
            component: AppTableEditableCellRenderer,
            params: {
              fieldState: params.data.getFieldState("date"),
            },
          };
        }
      },
      cellEditor: EntryTableDateCellEditor,
      filter: "agDateColumnFilter",
    },
    {
      headerName: "Tags",
      width: 200,
      valueGetter: (params) => {
        if (params.data instanceof ParentRow) {
          return params.data.tags;
        }
      },
      valueSetter: (params) => {
        if (params.newValue && params.data instanceof ParentRow) {
          params.data.tags = params.newValue;
          triggerRef(rows);
          return true;
        }
        return false;
      },
      editable: (params) => params.data instanceof ParentRow && params.data.editable("tags"),
      cellRendererSelector: (params: ICellRendererParams<Row>) => {
        if (params.data instanceof ParentRow) {
          return {
            component: AppTableTagsCellRenderer,
            params: {
              fieldState: params.data?.getFieldState("tags"),
            },
          };
        }
      },
    },
    {
      headerName: "Amount",
      valueGetter: (params) => {
        if (params.data instanceof ChildRow) {
          return params.data.amount;
        }
      },
      valueSetter: (params) => {
        if (params.newValue && params.data instanceof ChildRow) {
          params.data.amount = params.newValue;
          triggerRef(rows);
          return true;
        }
        return false;
      },
      editable: (params) => params.data instanceof ChildRow && params.data.editable("amount"),
      cellRendererSelector: (params: ICellRendererParams<Row>) => {
        if (params.data instanceof ChildRow) {
          return {
            component: AppTableEditableCellRenderer,
            params: {
              fieldState: params.data.getFieldState("amount"),
            },
          };
        }
      },
    },
    {
      headerName: "Price",
      valueGetter: (params) => {
        if (params.data instanceof ChildRow) {
          return params.data.price;
        }
      },
      valueSetter: (params) => {
        if (params.newValue && params.data instanceof ChildRow) {
          params.data.price = params.newValue;
          triggerRef(rows);
          return true;
        }
        return false;
      },
      editable: (params) => params.data instanceof ChildRow && params.data.editable("price"),
      cellRendererSelector: (params: ICellRendererParams<Row>) => {
        if (params.data instanceof ChildRow) {
          return {
            component: AppTableEditableCellRenderer,
            params: {
              fieldState: params.data.getFieldState("price"),
            },
          };
        }
      },
    },
  ];

  if (readonly.value) {
    return [
      ...results,
      {
        headerName: "State",
        valueGetter: (params) => params.data?.entryState,
        cellRendererSelector: (params) => {
          if (params.value) {
            return {
              component: EntryTableStateCellRenderer,
            };
          }
        },
      },
    ];
  }

  return results;
});

const getDataPath = (data: Row) => {
  if (data instanceof ParentRow) {
    return [data.id];
  } else {
    return data.id.split(":");
  }
};

const autoGroupColumnGroup = computed(
  (): ColDef<Row> => ({
    valueGetter: (params) => {
      if (params.data instanceof ParentRow) {
        return params.data.name;
      } else {
        const account = params.data && accountMap.value.get(params.data.accountId);
        return account?.name;
      }
    },
    valueSetter: (params) => {
      if (params.data instanceof ParentRow) {
        params.data.name = params.newValue;
        return true;
      } else if (params.data instanceof ChildRow) {
        const account = params.newValue && accountMap.value.get(params.newValue);
        params.data.accountId = account.id;
        return true;
      }

      return false;
    },
    editable: (params) => {
      if (params.data instanceof ParentRow) {
        return params.data.editable("name");
      } else if (params.data instanceof ChildRow) {
        return params.data.editable("account");
      }
      return false;
    },
    cellRendererParams: (params: ICellRendererParams<Row>) => {
      if (params.data instanceof ParentRow) {
        return {
          suppressDoubleClickExpand: true,
          innerRenderer: AppTableEditableCellRenderer,
          innerRendererParams: {
            fieldState: params.data.getFieldState("name"),
          },
        };
      } else if (params.data instanceof ChildRow) {
        return {
          suppressDoubleClickExpand: true,
          innerRenderer: AppTableAccountCellRenderer,
          innerRendererParams: {
            accounts: accountMap.value,
            fieldState: params.data.getFieldState("account"),
          },
        };
      }
    },
  }),
);
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
        <q-btn flat color="secondary" icon="add" label="Add" :loading="loading"></q-btn>
        <q-btn
          flat
          icon="cancel"
          label="Cancel"
          :loading="loading"
          @click="readonly = true"
        ></q-btn>
      </template>
    </div>
    <div class="flex gap-1 items-center"></div>
    <AppTable
      :row-data="rows"
      :column-defs="columnDefs"
      tree-data
      :get-data-path="getDataPath"
      :auto-group-column-def="autoGroupColumnGroup"
    ></AppTable>
  </div>
</template>
