<script setup lang="ts" generic="R">
import type { ICellRendererParams } from "@ag-grid-community/core";
import { computed } from "vue";

import type { FieldState } from "components/types";

import AppTableEditableCellRenderer from "./AppTableEditableCellRenderer.vue";

const props = defineProps<{
  readonly params: ICellRendererParams<R, string[]> & {
    readonly fieldState?: FieldState<string[]>;
  };
}>();

const state = computed(() => props.params.fieldState);
</script>

<template>
  <AppTableEditableCellRenderer v-if="state" :params="params">
    <div class="flex gap-1">
      <q-chip v-for="tag in state.value" :key="tag" :label="tag"></q-chip>
    </div>
  </AppTableEditableCellRenderer>
</template>
