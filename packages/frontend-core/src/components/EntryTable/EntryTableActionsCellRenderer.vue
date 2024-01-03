<script setup lang="ts">
import type { ICellRendererParams } from "@ag-grid-community/core";
import { ChildRow, ParentRow, type Row } from "./row";
import { computed } from "vue";
import uniq from "lodash/uniq";

type Params = ICellRendererParams<Row> & {
  toggleDeleted(): void;
  toggleAdded(): void;
};

const props = defineProps<{
  readonly params: Params;
}>();

const data = computed((): ParentRow | [ParentRow, ChildRow] | undefined => {
  if (props.params.data instanceof ParentRow) {
    return props.params.data;
  } else if (
    props.params.data instanceof ChildRow &&
    props.params.node.parent?.data instanceof ParentRow
  ) {
    return [props.params.node.parent.data, props.params.data];
  }
  return undefined;
});

const disableDelete = computed((): boolean => {
  const parentNode = props.params.node.parent;
  if (props.params.data instanceof ChildRow && parentNode?.data instanceof ParentRow) {
    if (props.params.data.deleted) {
      return false;
    }

    const parentData = parentNode.data;
    const existingItems = uniq(
      parentNode.allLeafChildren
        .map((child) => child.data)
        .filter(
          (data): data is ChildRow =>
            data instanceof ChildRow && data.parentId === parentData.id && !data.deleted,
        )
        .map((data) => data.accountId),
    );
    return existingItems.length <= 2;
  }
  return false;
});
</script>

<template>
  <div v-if="data" class="flex gap-1 items-center h-full">
    <template v-if="data instanceof ParentRow">
      <q-btn
        flat
        round
        size="sm"
        :icon="data.deleted ? 'undo' : 'delete'"
        color="negative"
        :disable="disableDelete"
        @click="params.toggleDeleted"
      >
        <q-tooltip>
          <template v-if="data.deleted"> Undo the deletion. </template>
          <template v-else> Delete this Entry and <strong>all related items</strong>. </template>
        </q-tooltip>
      </q-btn>
      <q-btn v-if="!data.deleted" flat round size="sm" icon="add" @click="params.toggleAdded">
        <q-tooltip>Add an empty item.</q-tooltip>
      </q-btn>
    </template>
    <template v-else>
      <q-btn
        v-if="!data[0].deleted"
        flat
        round
        size="sm"
        :icon="data[1].deleted ? 'undo' : 'delete'"
        color="negative"
        :disable="disableDelete"
        @click="params.toggleDeleted"
      >
        <q-tooltip>
          <template v-if="data[1].deleted"> Undo the deletion. </template>
          <template v-else-if="disableDelete">An Entry should keep at least two items.</template>
          <template v-else>Delete this Entry item.</template>
        </q-tooltip>
      </q-btn>
    </template>
  </div>
</template>
