<template>
  <AppScaffold>
    <q-select
      v-model="journalId"
      :options="journals"
      option-label="name"
      option-value="id"
      map-options
      emit-value
    ></q-select>
    <EntryWriteTable v-if="journal" :journal="journal"></EntryWriteTable>
    <TestAgCharts></TestAgCharts>
  </AppScaffold>
</template>

<script setup lang="ts">
import { EntryWriteTable, AppScaffold, TestAgCharts } from "@core/components";
import { useInject } from "@core/composable";
import type { JournalApi } from "@core/services";
import { JOURNAL_API_KEY } from "@core/services";
import { computedAsync } from "@vueuse/core";
import { computed, ref } from "vue";

const journalId = ref<string>();

const journalApi = useInject<JournalApi>(JOURNAL_API_KEY);
const journals = computedAsync(async () => {
  const [result, _included] = await journalApi.findAll({ query: {} });
  journalId.value = result[0].id;
  return result;
}, []);

const journal = computed(() => journals.value.find((j) => j.id === journalId.value));
</script>
