<template>
  <div class="flex gap-1 h-full items-center">
    <v-icon :icon="iconName"></v-icon>
    <span>{{ name }}</span>
  </div>
</template>

<script setup lang="ts">
import type { ICellRendererParams } from "@ag-grid-community/core";
import { computed } from "vue";
import { Child, type Row } from "./row";

const props = defineProps<{ readonly params: ICellRendererParams<Row> }>();

const iconName = computed(() => {
  const data = props.params.data;
  if (data instanceof Child) {
    return "checklist";
  } else if (data?.type === "Check") {
    return "verified";
  } else {
    return "payments";
  }
});

const name = computed(() =>
  props.params.data instanceof Child ? props.params.data.account.name : props.params.data?.name
);
</script>
