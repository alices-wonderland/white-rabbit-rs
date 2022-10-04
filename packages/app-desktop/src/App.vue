<template>
  <RecordViewTable></RecordViewTable>
  <code v-if="isReady">
    <pre>{{ JSON.stringify(state, null, "  ") }}</pre>
  </code>
  <div v-else>Loading...</div>
</template>

<script setup lang="ts">
import { invoke } from "@tauri-apps/api/tauri";
import { useAsyncState } from "@vueuse/core";
import { RecordViewTable } from "@white-rabbit/frontend-shared";

const { state, isReady } = useAsyncState(invoke("get_users"), []);
</script>
