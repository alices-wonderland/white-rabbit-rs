<template>
  <v-chip v-if="state">{{ state }}</v-chip>
</template>

<script setup lang="ts">
import type { ICellRendererParams } from "@ag-grid-community/core";
import type { Row } from "./row";
import { computed } from "vue";

const props = defineProps<{ params: ICellRendererParams<Row> }>();

const state = computed(() => props.params.data?.state && JSON.stringify(props.params.data.state));
</script>
