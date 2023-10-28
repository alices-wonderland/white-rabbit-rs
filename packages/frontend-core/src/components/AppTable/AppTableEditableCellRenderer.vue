<script setup lang="ts" generic="R">
import type { ICellRendererParams } from "@ag-grid-community/core";
import type { CellState } from "./index";
import { computed } from "vue";

const props = defineProps<{
  readonly params: ICellRendererParams<R> & { readonly cellState?: CellState<string> };
}>();

const state = computed(() => props.params.cellState);
</script>

<template>
  <div v-if="state" class="flex gap-1 w-full h-full items-center overflow-hidden">
    <span>{{ state.value }}</span>
    <q-badge
      v-if="state.state === 'UPDATED'"
      class="absolute top-0.5 right-0.5"
      align="top"
      rounded
      color="positive"
    >
      <q-tooltip>
        <slot v-if="$slots['tooltip']" name="tooltip"></slot>
        <template v-else>
          <div class="font-bold">Existing Value:</div>
          <div>{{ state.existing }}</div>
        </template>
      </q-tooltip>
    </q-badge>
  </div>
</template>
