<template>
  <v-slide-group class="h-full" show-arrows disabled>
    <div class="flex gap-x-1 items-center">
      <template v-if="isParent && isRecord">
        <template v-if="data.state.length <= 1">
          <v-chip color="info" size="small">Valid</v-chip>
          <v-chip color="primary" size="small"
            >Amount: {{ data.state[0] }}</v-chip
          >
        </template>
        <template v-else>
          <v-chip color="error" size="small">Invalid</v-chip>
          <v-chip color="primary" size="small"
            >Left: {{ data.state[0] }}</v-chip
          >
          <v-chip color="secondary" size="small"
            >Right: {{ data.state[1] }}</v-chip
          >
        </template>
      </template>
      <template v-else-if="isParent"></template>
      <template v-else-if="isRecord">
        <v-chip color="primary" size="small"
          >Amount: {{ data.state[0] }}</v-chip
        >
        <v-chip v-if="data.state[1]" color="secondary" size="small"
          >Price: {{ data.state[1] }}</v-chip
        >
      </template>
      <template v-else>
        <v-chip color="error" size="small">Invalid</v-chip>
        <v-chip color="primary" size="small"
          >Expected: {{ data.state[0] }}</v-chip
        >
        <v-chip color="secondary" size="small"
          >Actual: {{ data.state[1] }}</v-chip
        >
      </template>
    </div>
  </v-slide-group>
</template>

<script lang="ts" setup>
import { ICellRendererParams } from "@ag-grid-community/core";
import { computed } from "vue";
import { RecordType } from "@shared/models";

const props = defineProps<{
  params: ICellRendererParams;
}>();

const data = computed(() => props.params.data);
const isParent = computed(() => data.value.hierarchy.length <= 1);
const isRecord = computed(
  () =>
    data.value.recordType === RecordType.RECORD ||
    data.value.type === RecordType.RECORD
);
</script>
