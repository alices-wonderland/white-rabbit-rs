<template>
  <div class="flex gap-1 h-full items-center">
    <q-btn
      v-if="isDeletable"
      size="sm"
      flat
      color="error"
      round
      :icon="deleted ? 'undo' : 'delete'"
      @click="toggleDelete"
    ></q-btn>
    <q-btn size="sm" flat icon="content_copy" round @click="props.params.clone"></q-btn>
    <q-btn
      v-if="data instanceof Parent"
      size="sm"
      flat
      icon="add"
      round
      @click="props.params.addChild"
    ></q-btn>
  </div>
</template>

<script setup lang="ts">
import type { ICellRendererParams } from "@ag-grid-community/core";
import { Parent } from "./row";
import type { Row } from "./row";
import { computed } from "vue";
import type { ComputedRef } from "vue";

const props = defineProps<{
  params: ICellRendererParams<Row> & { addChild: () => void; clone: () => void };
}>();

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
