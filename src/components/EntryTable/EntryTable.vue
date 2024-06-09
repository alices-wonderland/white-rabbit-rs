<script setup lang="ts">
import { computed, ref, shallowRef, triggerRef, watch } from "vue";
import type { CellValueChangedEvent, ColDef, ICellRendererParams } from "@ag-grid-community/core";
import { format, isMatch } from "date-fns";
import { useQueryClient } from "@tanstack/vue-query";

import {
  AppTable,
  AppTableEditableCellRenderer,
  AppTableTagsCellRenderer,
  AppTableAccountCellRenderer,
  AppTableAccountCellEditor,
  AppTableTagsCellEditor,
} from "components/AppTable";
import { Journal } from "src/services/journal";
import { Account } from "src/services/account";
import {
  Entry,
  ENTRY_TYPES,
  type EntryCommandBatch,
  type EntryCommandCreate,
  type EntryCommandUpdate,
  type EntryItem,
} from "src/services/entry";
import { useEntryCommand } from "src/composable/useCommand";

import { ChildRow, createAll, ParentRow } from "./row";
import type { Row } from "./row";
import EntryTableDateCellEditor from "./EntryTableDateCellEditor.vue";
import EntryTableStateCellRenderer from "./EntryTableStateCellRenderer.vue";
import EntryTableActionsCellRenderer from "./EntryTableActionsCellRenderer.vue";

const props = defineProps<{
  readonly modelValue: Entry[];
  readonly journal: Journal;
  readonly accounts: Account[];
}>();

const queryClient = useQueryClient();

const readonly = ref(true);

const rows = shallowRef<Row[]>([]);
watch(
  (): [Entry[], boolean] => [props.modelValue, readonly.value],
  ([newValues, newReadonly]) => {
    rows.value = createAll(newValues, newReadonly);
  },
  { immediate: true },
);

const accountMap = computed(() => new Map(props.accounts.map((account) => [account.id, account])));

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
          return true;
        }
        return false;
      },
      filter: "agTextColumnFilter",
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
      filterValueGetter: (params) => {
        if (params.data instanceof ParentRow) {
          return params.data.type;
        } else if (params.data) {
          const parent = parentAndChildRows.value[0].get(params.data.parentId);
          return parent?.type;
        }
        return undefined;
      },
      valueSetter: (params) => {
        if (params.data instanceof ParentRow && params.newValue) {
          params.data.type = params.newValue;
          return true;
        }
        return false;
      },
      editable: (params) => params.data instanceof ParentRow && params.data.editable("type"),
      cellEditor: "agSelectCellEditor",
      cellEditorParams: {
        values: ENTRY_TYPES,
      },
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
        if (params.data instanceof ParentRow && isMatch(params.newValue, "yyyy-MM-dd")) {
          params.data.date = params.newValue;
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
      filterParams: () => {
        return {
          comparator: (filterValue: Date, cellValue: string) => {
            const formattedFilter = format(filterValue, "yyyy-MM-dd");
            return cellValue.localeCompare(formattedFilter);
          },
        };
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
          return true;
        }
        return false;
      },
      filter: "agNumberColumnFilter",
      cellEditor: "agNumberCellEditor",
      cellEditorParams: {
        min: 0,
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
        if (params.newValue > 0 && params.data instanceof ChildRow) {
          params.data.price = params.newValue;
          return true;
        }
        return false;
      },
      filter: "agNumberColumnFilter",
      cellEditor: "agNumberCellEditor",
      cellEditorParams: {
        min: 0,
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
    {
      headerName: "Tags",
      width: 200,
      valueGetter: (params) => {
        if (params.data instanceof ParentRow) {
          return params.data.tags;
        }
      },
      filterValueGetter: (params) => {
        if (params.data instanceof ParentRow) {
          return params.data.tags;
        } else if (params.data) {
          const parent = parentAndChildRows.value[0].get(params.data.parentId);
          return parent?.tags;
        }
        return undefined;
      },
      valueSetter: (params) => {
        if (params.newValue && params.data instanceof ParentRow) {
          params.data.tags = params.newValue;
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
      cellEditor: AppTableTagsCellEditor,
      useValueParserForImport: true,
      valueFormatter: (params) => params.value?.join(","),
      valueParser: (params) => params.newValue.split(","),
    },
  ];

  if (readonly.value) {
    return [
      ...results,
      {
        headerName: "State",
        filter: false,
        valueGetter: (params) => params.data?.entryState,
        valueFormatter: () => "",
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

  return [
    {
      headerName: "Actions",
      filter: false,
      width: 120,
      lockPosition: "left",
      cellRenderer: EntryTableActionsCellRenderer,
      cellRendererParams: (params: ICellRendererParams<Row>) => {
        return {
          toggleDeleted: () => {
            if (params.node && params.data) {
              params.data.deleted = !params.data.deleted;

              let parent = params.node;
              if (params.data instanceof ChildRow && params.node.parent) {
                parent = params.node.parent;
              } else if (params.data instanceof ParentRow) {
                for (const child of params.node.allLeafChildren) {
                  if (child.data instanceof ChildRow) {
                    child.data.deleted = params.data.deleted;
                  }
                }
              }

              params.api.redrawRows({ rowNodes: [parent, ...parent.allLeafChildren] });
              if (params.data instanceof ChildRow) {
                triggerRef(rows);
              } else if (params.data instanceof ParentRow) {
                const parentId = params.data.id;
                rows.value = rows.value.filter(
                  (row) =>
                    !(
                      row instanceof ChildRow &&
                      row.parentId === parentId &&
                      row.rowState.state === "NEW"
                    ),
                );
              }
            }
          },
          toggleAdded: () => {
            if (params.data instanceof ParentRow && !params.data.deleted) {
              rows.value = [...rows.value, new ChildRow(params.data.id)];
            }
          },
        };
      },
    } as ColDef<Row>,
    ...results,
  ];
});

const onCellValueChanged = (event: CellValueChangedEvent<Row>) => {
  console.log("On Cell Value Changed:", event);
  event.api.redrawRows({ rowNodes: [event.node] });
  triggerRef(rows);
};

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
        return account?.id;
      }
    },
    filterValueGetter: (params) => {
      if (params.data instanceof ParentRow) {
        const children = parentAndChildRows.value[1].get(params.data.id);
        const names =
          children
            ?.map((item) => accountMap.value.get(item.accountId)?.name)
            .filter((name): name is string => !!name) ?? [];
        return [params.data.name, ...names];
      } else {
        const account = params.data && accountMap.value.get(params.data.accountId);
        return account?.name;
      }
    },
    valueSetter: (params) => {
      if (params.data instanceof ParentRow) {
        params.data.name = params.newValue;
        return true;
      } else if (params.data instanceof ChildRow && accountMap.value.has(params.newValue)) {
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
    cellEditorSelector: (params) => {
      if (params.data instanceof ParentRow) {
        return {
          component: "agTextCellEditor",
        };
      } else if (params.data instanceof ChildRow) {
        return {
          component: AppTableAccountCellEditor,
          params: {
            accounts: props.accounts,
          },
        };
      }
    },
  }),
);

const parentAndChildRows = computed((): [Map<string, ParentRow>, Map<string, ChildRow[]>] => {
  const parentRows = new Map<string, ParentRow>();
  const childRowsByParent = new Map<string, ChildRow[]>();
  for (const row of rows.value) {
    if (row instanceof ParentRow) {
      parentRows.set(row.id, row);
    } else if (accountMap.value.has(row.accountId)) {
      const existing = childRowsByParent.get(row.parentId) ?? [];
      childRowsByParent.set(row.parentId, [...existing, row]);
    }
  }
  return [parentRows, childRowsByParent];
});

const deletedIds = computed(() => {
  return [...parentAndChildRows.value[0].values()]
    .filter((row) => row.deleted)
    .map((row) => row.id);
});

const createCommands = computed((): EntryCommandCreate[] => {
  const [parentRows, childRowsByParent] = parentAndChildRows.value;
  const commands: EntryCommandCreate[] = [];

  for (const parent of parentRows.values()) {
    const children = childRowsByParent.get(parent.id);
    if (parent.rowState.state === "NEW" && children && children.length >= 2) {
      const items = children.map(
        (child): EntryItem => ({
          account: child.accountId,
          amount: child.amount,
          price: child.price,
        }),
      );

      commands.push({
        commandType: "entries:create",
        id: parent.id,
        journalId: props.journal.id,
        name: parent.name,
        description: parent.description,
        type: parent.type,
        date: parent.date,
        tags: parent.tags,
        items: items,
      });
    }
  }

  return commands;
});

const updateCommands = computed(() => {
  const [parentRows, childRowsByParent] = parentAndChildRows.value;

  const commands: EntryCommandUpdate[] = [];
  for (const parent of parentRows.values()) {
    const parentState = parent.rowState;

    if (parentState.state === "DELETED") {
      continue;
    }

    const children = childRowsByParent.get(parent.id) ?? [];
    if (children.length < 2) {
      continue;
    }

    const childStates = new Set(children.map((child) => child.rowState.state));

    if (parentState.state === "NORMAL" && childStates.has("NORMAL") && childStates.size === 1) {
      continue;
    }

    const items = children
      .filter((child) => child.rowState.state !== "DELETED")
      .map(
        (child): EntryItem => ({
          account: child.accountId,
          amount: child.amount,
          price: child.price,
        }),
      );

    commands.push({
      commandType: "entries:update",
      id: parent.id,
      name: parent.name,
      description: parent.description,
      type: parent.type,
      date: parent.date,
      tags: parent.tags,
      items: items,
    });
  }

  return commands;
});

const batchCommand = computed((): EntryCommandBatch | undefined => {
  if (
    !readonly.value &&
    (createCommands.value.length > 0 ||
      updateCommands.value.length > 0 ||
      deletedIds.value.length > 0)
  ) {
    return {
      commandType: "entries:batch",
      create: createCommands.value,
      update: updateCommands.value,
      delete: deletedIds.value,
    };
  }

  return undefined;
});

const { mutateAsync: batchAsync, isPending: batchPending } = useEntryCommand({
  async onSuccess() {
    readonly.value = true;
    await queryClient.invalidateQueries({ queryKey: ["entries"] });
  },
});

const addParentRow = () => {
  rows.value = [new ParentRow(), ...rows.value];
};
</script>

<template>
  <div class="flex flex-col gap-2">
    <div class="flex gap-1 items-center">
      <template v-if="readonly">
        <q-btn
          color="primary"
          icon="edit"
          label="Edit"
          :loading="batchPending"
          @click="readonly = false"
        ></q-btn>
      </template>
      <template v-else>
        <q-btn
          color="primary"
          icon="save"
          label="Save"
          :disable="!batchCommand"
          @click="batchCommand && batchAsync(batchCommand)"
        ></q-btn>
        <q-btn
          flat
          color="secondary"
          icon="add"
          label="Add"
          :loading="batchPending"
          @click="addParentRow"
        ></q-btn>
        <q-btn
          flat
          icon="cancel"
          label="Cancel"
          :loading="batchPending"
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
      @cell-value-changed="onCellValueChanged"
    ></AppTable>
  </div>
</template>
