<template>
  <AppScaffold>
    <v-select v-model="journal" :items="journals" item-title="name" item-value="id"></v-select>
    <RecordWriteTable v-if="journal" :journal="journal"></RecordWriteTable>
  </AppScaffold>
</template>

<script setup lang="ts">
import { RecordWriteTable, AppScaffold } from "@core/components";
import { useInject } from "@core/composable";
import type { JournalApi } from "@core/services";
import { JOURNAL_API_KEY } from "@core/services";
import { computedAsync } from "@vueuse/core";
import { ref } from "vue";

const journal = ref<string>();

const journalApi = useInject<JournalApi>(JOURNAL_API_KEY);
const journals = computedAsync(async () => {
  const [result, _included] = await journalApi.findAll({ query: {} }, false);
  journal.value = result[0].id;
  return result;
}, []);
</script>
