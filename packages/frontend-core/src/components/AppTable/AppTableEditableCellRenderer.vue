<script setup lang="ts" generic="R, V">
import type { ICellRendererParams } from "@ag-grid-community/core";
import type { FieldState } from "@core/types";
import { computed } from "vue";

const props = defineProps<{
  readonly params: ICellRendererParams<R, V> & { readonly fieldState?: FieldState<V> };
}>();

const state = computed(() => props.params.fieldState);
</script>

<template>
  <div v-if="state" class="flex gap-1 w-full h-full items-center overflow-hidden">
    <slot>
      <span>{{ state.value }}</span>
    </slot>
    <q-badge
      v-if="state.state === 'UPDATED'"
      class="absolute top-0.5 right-0.5"
      align="top"
      rounded
      color="positive"
    >
      <q-tooltip>
        <slot name="tooltip">
          <div class="font-bold">Existing Value:</div>
          <div>{{ state.existing }}</div>
        </slot>
      </q-tooltip>
    </q-badge>
  </div>
</template>
