<template>
  <div v-if="records" class="flex flex-col gap-4">
    <RecordReadTable
      style="width: 100%; height: 500px"
      :records="records"
      :editable="false"
    >
    </RecordReadTable>
    <RecordWriteTable
      style="width: 100%; height: 500px"
      :records="records"
      :editable="false"
    >
    </RecordWriteTable>
  </div>
  <div v-else>Loading...</div>
</template>

<script setup lang="ts">
import { computedAsync } from "@vueuse/core";
import { RecordReadTable, RecordWriteTable } from "@shared/components";
import { Record_ } from "@shared/models";
import { useRecordApi } from "@shared/hooks";
import { Order } from "@shared/services";

const recordApi = useRecordApi();

const records = computedAsync<Record_[]>(
  () => recordApi.findAll({ sort: { field: "date", order: Order.DESC } }),
  undefined,
  {
    onError(e) {
      console.error("Error when loading records: " + e);
    },
  }
);
</script>
