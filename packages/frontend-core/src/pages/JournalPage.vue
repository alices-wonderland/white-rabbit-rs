<script setup lang="ts">
import { useAccounts, useJournals } from "@core/composable";
import { useRoute } from "vue-router";
import { computed, ref } from "vue";
import JournalCard from "@core/components/JournalCard.vue";
import type { AccountQuery, AccountSort, FindAllArgs } from "@core/services";
import { AccountTable } from "@core/components/AccountTable";

const route = useRoute();

const { models: journals, loading: journalsLoading } = useJournals({});
const journalMap = computed(() => new Map(journals.value.map((journal) => [journal.id, journal])));

const journal = computed(() => journals.value.find((model) => model.id === route.params["id"]));

const accountArgs = computed<FindAllArgs<AccountQuery, AccountSort> | undefined>(() => {
  if (journal.value) {
    return {
      query: {
        journalId: [journal.value.id],
      },
      sort: "name",
    };
  } else {
    return undefined;
  }
});
const { models: accounts, loading: accountsLoading } = useAccounts(accountArgs);

const loading = computed(() => journalsLoading.value || accountsLoading.value);

type Tab = "Accounts" | "Entries";

const tab = ref<Tab>("Accounts");
</script>

<template>
  <q-linear-progress v-if="loading" indeterminate></q-linear-progress>
  <span v-else-if="!journal">Not Found</span>
  <div v-else class="flex gap-4">
    <div class="w-1/4">
      <JournalCard :model-value="journal"></JournalCard>
    </div>
    <div class="flex-1">
      <q-card>
        <q-tabs v-model="tab" align="justify" narrow-indicator>
          <q-tab name="Accounts" label="Accounts" />
          <q-tab name="Entries" label="Entries" />
        </q-tabs>
        <q-separator />
        <q-tab-panels v-model="tab" animated>
          <q-tab-panel name="Accounts">
            <AccountTable :model-value="accounts" :journals="journalMap"></AccountTable>
          </q-tab-panel>
          <q-tab-panel name="Entries">
            <div class="text-h6">Entries</div>
            Lorem ipsum dolor sit amet consectetur adipisicing elit.
          </q-tab-panel>
        </q-tab-panels>
      </q-card>
    </div>
  </div>
</template>
