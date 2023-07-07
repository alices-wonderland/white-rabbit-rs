<template>
  <div class="flex gap-1 h-full items-center">
    <v-btn
      v-if="isDeletable"
      size="x-small"
      variant="text"
      color="error"
      :icon="deleted ? mdiUndo : mdiDelete"
      @click="toggleDelete"
    ></v-btn>
    <v-btn size="x-small" variant="text" :icon="mdiContentCopy"></v-btn>
    <v-btn
      v-if="data instanceof Parent"
      size="x-small"
      variant="text"
      :icon="mdiPlus"
      @click="props.params.addChild"
    ></v-btn>
  </div>
</template>

<script setup lang="ts">
import type { ICellRendererParams } from "@ag-grid-community/core";
import { Parent, type Row } from "./row";
import { computed, type ComputedRef } from "vue";
import { mdiDelete, mdiUndo, mdiContentCopy, mdiPlus } from "@mdi/js";

const props = defineProps<{ params: ICellRendererParams<Row> & { addChild: () => void } }>();

const data = computed(() => props.params.data);

const deleted: ComputedRef<boolean | undefined> = computed(() => props.params.data?.isDeleted);

const isDeletable = computed(() => typeof deleted.value === "boolean");

const toggleDelete = () => {
  if (data.value && isDeletable) {
    data.value.isDeleted = !deleted.value;
    const children = props.params.node.childrenAfterGroup;
    if (children && children.length > 0) {
      props.params.api.redrawRows({ rowNodes: children });
    }
  }
};
</script>
