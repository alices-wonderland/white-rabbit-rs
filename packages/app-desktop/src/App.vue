<template>
  <RecordViewTable></RecordViewTable>
  <div>
    <h3>User List</h3>
    <label>
      Input
      <input v-model="allInput" />
    </label>
    <code v-if="getAllState">
      <pre>{{ JSON.stringify(getAllState, null, "  ") }}</pre>
    </code>
  </div>

  <div>
    <h3>User Page</h3>
    <label>
      Input
      <input v-model="pageInput" />
    </label>
    <code v-if="getUserPageState">
      <pre>{{ JSON.stringify(getUserPageState, null, "  ") }}</pre>
    </code>
  </div>
</template>

<script setup lang="ts">
import { invoke } from "@tauri-apps/api/tauri";
import { computedAsync } from "@vueuse/core";
import { RecordViewTable } from "@white-rabbit/frontend-shared";
import { ref } from "vue";

const allInput = ref<string>("");
const getAllState = computedAsync<unknown[]>(
  () =>
    invoke("get_users", {
      operator: "a0152c9d-11b0-d91b-f6cf-fed7925a6622",
      input: {
        query: { name: { value: allInput.value, __fullText: true } },
        sort: { field: "name", order: "Desc" },
      },
    }),
  []
);

const pageInput = ref<string>("");
const getUserPageState = computedAsync<unknown[]>(
  () =>
    invoke("get_user_page", {
      operator: "a0152c9d-11b0-d91b-f6cf-fed7925a6622",
      input: {
        query: { name: { value: pageInput.value, __fullText: true } },
        sort: { field: "name", order: "Asc" },
      },
    }),
  []
);
</script>
