<script setup lang="ts">
import { useJournals } from "@core/composable";
import JournalCard from "@core/components/JournalCard.vue";
import { computed, ref } from "vue";
import { refDebounced } from "@vueuse/core";
import type { FindAllArgs, JournalQuery, JournalSort } from "@core/services";

const search = ref("");
const searchDebounced = refDebounced(search, 500);

const journalsArgs = computed<FindAllArgs<JournalQuery, JournalSort>>(() => ({
  query: {
    fullText: searchDebounced.value ? [searchDebounced.value, ["name"]] : undefined,
  },
  sort: "name",
}));

const { models: journals, loading: journalsLoading } = useJournals(journalsArgs);

const doLoading = ref(false);
const loading = computed(() => journalsLoading.value || doLoading.value);

const showAddDialog = ref(false);
</script>

<template>
  <div class="flex flex-col gap-4">
    <div class="flex gap-2">
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
      <q-btn icon="add" color="primary" label="Add" @click="showAddDialog = true"></q-btn>
      <q-btn icon="edit" flat label="Batch Update"></q-btn>
    </div>
    <div v-if="!loading" class="flex flex-col gap-2">
      <JournalCard v-for="journal in journals" :key="journal.id" :model-value="journal" readonly>
      </JournalCard>
    </div>
  </div>
  <q-dialog v-model="showAddDialog">
    <JournalCard class="w-3/5">
      <template #actions>
        <q-btn icon="save" color="primary">Save</q-btn>
        <q-btn flat icon="cancel" @click="showAddDialog = false">Cancel</q-btn>
      </template>
    </JournalCard>
  </q-dialog>
</template>
