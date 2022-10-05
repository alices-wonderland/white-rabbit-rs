<template>
  <RecordTable
    v-if="records"
    style="width: 100%; height: 500px"
    :records="records"
    :editable="false"
  ></RecordTable>
  <div v-else>Loading...</div>
</template>

<script setup lang="ts">
import { invoke } from "@tauri-apps/api/tauri";
import { computedAsync } from "@vueuse/core";
import { RecordTable } from "@shared/components";
import { Record_ } from "@shared/models";

const records = computedAsync<Record_[]>(
  async () => {
    const users = await invoke<any[]>("get_users", {
      input: {
        query: { role: "Owner" },
        sort: { field: "date", order: "Desc" },
      },
    });

    return invoke("get_records", {
      operator: users[0].id,
      input: {
        query: {},
        sort: { field: "date", order: "Desc" },
      },
    });
  },
  undefined,
  {
    onError(e) {
      console.error("Error when loading records: " + e);
    },
  }
);
</script>
