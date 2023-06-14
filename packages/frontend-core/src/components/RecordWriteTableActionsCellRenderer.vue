<template>
  <div class="flex gap-1 h-full items-center">
    <v-btn
      v-if="isDeletable"
      size="x-small"
      variant="text"
      color="error"
      :icon="deleted ? 'undo' : 'deleted'"
      @click="toggleDelete"
    ></v-btn>
    <v-btn size="x-small" variant="text" icon="content_copy"></v-btn>
  </div>
</template>

<script setup lang="ts">
import type { ICellRendererParams } from "@ag-grid-community/core";
import type { Row } from "./row";
import { computed, type ComputedRef } from "vue";

const props = defineProps<{ params: ICellRendererParams<Row> }>();

const deleted: ComputedRef<boolean | undefined> = computed(() => props.params.data?.deleted);

const isDeletable = computed(() => typeof deleted.value === "boolean");

const toggleDelete = () => {
  const data = props.params.data;
  if (data && isDeletable) {
    data.deleted = !deleted.value;
    const children = props.params.node.childrenAfterGroup;
    if (children && children.length > 0) {
      props.params.api.redrawRows({ rowNodes: children });
    }
  }
};
</script>
