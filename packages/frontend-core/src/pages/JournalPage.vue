<script setup lang="ts">
import { useJournals } from "@core/composable";
import { useRoute } from "vue-router";
import { computed } from "vue";
import JournalCard from "@core/components/JournalCard.vue";

const { models: journals, loading } = useJournals({});

const route = useRoute();

const journal = computed(() => journals.value.find((model) => model.id === route.params["id"]));
</script>

<template>
  <q-linear-progress v-if="loading" indeterminate></q-linear-progress>
  <span v-else-if="!journal">Not Found</span>
  <div v-else class="flex flex-col gap-2">
    <JournalCard :model-value="journal"></JournalCard>
  </div>
</template>
