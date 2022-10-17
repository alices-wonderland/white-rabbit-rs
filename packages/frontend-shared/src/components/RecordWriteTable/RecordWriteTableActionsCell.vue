<template>
  <div class="h-full flex gap-1 items-center">
    <v-btn
      v-if="errors"
      title="Errors"
      icon
      color="error"
      size="x-small"
      variant="text"
    >
      <v-icon class="animate-pulse" :icon="mdiAlertCircleOutline"></v-icon>
      <v-tooltip activator="parent" location="end">
        {{ JSON.stringify([...errors], null, 2) }}
      </v-tooltip>
    </v-btn>
    <v-btn
      v-if="!isParentDeleted && data?.isDeleted"
      title="Revert deletion"
      color="primary"
      size="x-small"
      variant="text"
      :icon="mdiUndo"
      @click="toggleDeleted"
    >
    </v-btn>
    <v-btn
      v-else-if="!isParentDeleted"
      title="Delete"
      color="primary"
      size="x-small"
      variant="text"
      :icon="mdiDelete"
      @click="toggleDeleted"
    >
    </v-btn>
  </div>
</template>

<script setup lang="ts">
import { GridApi, ICellRendererParams, RowNode } from "@ag-grid-community/core";
import { mdiAlertCircleOutline, mdiUndo, mdiDelete } from "@mdi/js";
import { computed } from "vue";
import { RecordItemRow, Row } from "./types";

type Params = ICellRendererParams<Row> & {
  toggleDeleted(api: GridApi, row: RowNode<Row>): void;
};

const props = defineProps<{ params: Params }>();

const data = computed(() => props.params.data);

const errors = computed(() => {
  const errors = data.value?.errors(props.params.node);
  console.log("Errors: ", errors);
  return errors;
});

const isParentDeleted = computed(
  () => data.value instanceof RecordItemRow && data.value.isParentDeleted
);

const toggleDeleted = () => {
  const data = props.params.data;
  const isDeleted = !data?.isDeleted;

  if (data) {
    data.isDeleted = isDeleted;
    props.params.node.setData(data);
  }

  for (const child of props.params.node.childrenAfterGroup ?? []) {
    if (child.data instanceof RecordItemRow) {
      child.data.isParentDeleted = isDeleted;
      child.setData(child.data);
    }
  }
};
</script>
