<script setup lang="ts">
import type { ICellRendererParams } from "@ag-grid-community/core";
import type { Row } from "./row";

type Params = ICellRendererParams<Row> & {
  toggleDeleted: () => void;
};

defineProps<{ readonly params: Params }>();
</script>

<template>
  <div class="flex gap-1 items-center h-full">
    <q-btn
      flat
      round
      size="sm"
      color="negative"
      :icon="params.data?.rowState?.state === 'DELETED' ? 'undo' : 'delete'"
      @click="params.toggleDeleted"
    ></q-btn>
  </div>
</template>
