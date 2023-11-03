<script setup lang="ts">
import { useRoute, useRouter } from "vue-router";
import { useJournals } from "@core/composable";
import { computed, ref, watch } from "vue";
import { JOURNAL_ICON } from "@core/services";

const route = useRoute();
const router = useRouter();

const journalId = computed(() => route.params["id"] as string);

const {
  models: journals,
  loading,
  reload,
} = useJournals({
  sort: "name",
});

const keyword = ref<string>("");

const filtered = computed(() =>
  journals.value.filter((journal) =>
    journal.name.toLowerCase().includes(keyword.value.trim().toLowerCase()),
  ),
);

const current = computed(() => journals.value.find((journal) => journal.id === journalId.value));

watch(journalId, async () => {
  keyword.value = "";
  await reload();
});
</script>

<template>
  <q-toolbar inset>
    <q-circular-progress v-if="loading" indeterminate></q-circular-progress>
    <q-breadcrumbs v-else active-color="on-primary">
      <q-breadcrumbs-el label="Journals" to="/journals" :icon="JOURNAL_ICON"></q-breadcrumbs-el>
      <q-breadcrumbs-el v-if="current" :to="`/journals/${current.id}`">
        <span class="flat flat-col gap-1 items-center">
          {{ current.name }}
          <q-btn icon="arrow_drop_down" round flat size="xs">
            <q-menu>
              <q-list>
                <q-item-label header>
                  <div class="flex gap-1 items-center">
                    <q-icon name="flag"></q-icon>
                    Navigate to
                  </div>
                  <q-input v-model="keyword" dense label="Search" clearable></q-input>
                </q-item-label>
                <q-separator></q-separator>
                <q-item
                  v-for="journal in filtered"
                  :key="journal.id"
                  :disable="current.id === journal.id"
                  clickable
                  @click="router.push(`/journals/${journal.id}`)"
                >
                  {{ journal.name }}
                </q-item>
              </q-list>
            </q-menu>
          </q-btn>
        </span>
      </q-breadcrumbs-el>
    </q-breadcrumbs>
  </q-toolbar>
</template>
