<template>
  <div class="flex gap-1 h-full items-center">
    <q-chip v-if="state">{{ state }}</q-chip>
  </div>
</template>

<script setup lang="ts">
import type { ICellRendererParams } from "@ag-grid-community/core";
import type { Row } from "./row";
import { computed } from "vue";

const props = defineProps<{ params: ICellRendererParams<Row> }>();

const state = computed(() => props.params.data?.state && JSON.stringify(props.params.data.state));
</script>
