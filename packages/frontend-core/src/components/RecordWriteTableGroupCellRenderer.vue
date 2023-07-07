<template>
  <div class="flex gap-1 h-full items-center w-fit">
    <v-icon :icon="iconName"></v-icon>
    <span>{{ name }}</span>
  </div>
</template>

<script setup lang="ts">
import type { ICellRendererParams } from "@ag-grid-community/core";
import { computed } from "vue";
import { Child, type Row } from "./row";
import { mdiAccountCashOutline, mdiCheckDecagramOutline, mdiCashMultiple } from "@mdi/js";

const props = defineProps<{ readonly params: ICellRendererParams<Row> }>();

const iconName = computed(() => {
  const data = props.params.data;
  if (data instanceof Child) {
    return mdiAccountCashOutline;
  } else if (data?.type === "Check") {
    return mdiCheckDecagramOutline;
  } else {
    return mdiCashMultiple;
  }
});

const name = computed(() =>
  props.params.data instanceof Child
    ? `${props.params.data.account?.name} :: ${props.params.data.account?.unit}`
    : `${props.params.data?.name} :: ${props.params.data?.journal?.unit}`,
);
</script>
