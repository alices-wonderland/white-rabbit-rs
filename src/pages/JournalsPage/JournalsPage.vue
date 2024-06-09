<script setup lang="ts">
import { computed, ref } from "vue";
import { refDebounced } from "@vueuse/core";

import { useJournals } from "src/composable/useAll";
import { JournalCard } from "components/JournalCard";
import { Journal, type JournalQuery, type JournalSort } from "src/services/journal";
import type { FindAllArgs } from "src/services/api";
import JournalDeleteDialog from "components/JournalDeleteDialog.vue";

import JournalsPageCreateDialog from "./JournalsPageCreateDialog.vue";

const search = ref("");
const searchDebounced = refDebounced(search, 500);

const journalsArgs = computed<FindAllArgs<JournalQuery, JournalSort>>(() => ({
  query: {
    fullText: searchDebounced.value ? [searchDebounced.value, ["name"]] : undefined,
  },
  sort: "name",
}));

const { data: journalsData, status: journalsStatus } = useJournals(journalsArgs);
const journals = computed(() => (journalsData.value ? journalsData.value[0] : []));

const loading = computed(() => journalsStatus.value === "pending");

const showCreateDialog = ref(false);
const deletingJournal = ref<Journal>();
</script>

<template>
  <div class="flex flex-col gap-4 w-3/5">
    <div class="flex gap-2">
      <q-btn icon="add" color="primary" label="Add" @click="showCreateDialog = true"></q-btn>
      <q-btn icon="edit" flat label="Batch Update"></q-btn>
      <q-space></q-space>

      <q-input
        v-model="search"
        label="Search"
        clearable
        autofocus
        :loading="loading"
        class="w-96"
        dense
      >
        <template #prepend>
          <q-icon name="search"></q-icon>
        </template>
      </q-input>
    </div>
    <div v-if="!loading" class="flex flex-col gap-2">
      <JournalCard v-for="journal in journals" :key="journal.id" :model-value="journal" readonly>
        <template #actions>
          <q-btn
            flat
            icon="delete"
            color="negative"
            label="Delete"
            :loading="loading"
            @click="deletingJournal = journal"
          >
          </q-btn>
        </template>
      </JournalCard>
    </div>
  </div>
  <JournalsPageCreateDialog v-model="showCreateDialog"></JournalsPageCreateDialog>
  <JournalDeleteDialog
    v-if="deletingJournal"
    :journal="deletingJournal"
    :model-value="!!deletingJournal"
    @update:model-value="deletingJournal = undefined"
  ></JournalDeleteDialog>
</template>
