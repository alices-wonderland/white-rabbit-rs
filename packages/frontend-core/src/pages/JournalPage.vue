<script setup lang="ts">
import { useRoute, useRouter } from "vue-router";
import { computed, ref } from "vue";
import { useAccounts, useEntries, useInject, useJournals } from "@core/composable";
import { JournalCard } from "@core/components/JournalCard";
import type { Value as JournalCardValue } from "@core/components/JournalCard";
import { AccountTable } from "@core/components/AccountTable";
import type {
  EntryQuery,
  EntrySort,
  AccountQuery,
  AccountSort,
  FindAllArgs,
  JournalApi,
  JournalCommandUpdate,
} from "@core/services";
import { JOURNAL_API_KEY } from "@core/services";
import JournalDeleteDialog from "@core/components/JournalDeleteDialog.vue";
import { EntryTable } from "@core/components/EntryTable";
import TestAgCharts from "@core/components/TestAgCharts.vue";
import { ROUTE_JOURNALS } from "@core/router";

const route = useRoute();
const router = useRouter();

const { data: journalsData, status: journalsStatus, refetch: journalsRefetch } = useJournals({});
const journals = computed(() => (journalsData.value ? journalsData.value[0] : []));

const journalApi = useInject<JournalApi>(JOURNAL_API_KEY);

const journalId = computed(() => route.params["id"] as string);

const journal = computed(() => journals.value.find((model) => model.id === journalId.value));

const accountArgs = computed<FindAllArgs<AccountQuery, AccountSort> | undefined>(() => {
  if (journalId.value) {
    return {
      query: {
        journalId: [journalId.value],
      },
      sort: "name",
    };
  }

  return undefined;
});

const { data: accountsData, status: accountsStatus } = useAccounts(accountArgs);
const accounts = computed(() => (accountsData.value ? accountsData.value[0] : []));

const entriesArgs = computed((): FindAllArgs<EntryQuery, EntrySort> | undefined => {
  if (journalId.value) {
    return {
      query: {
        journalId: [journalId.value],
      },
      sort: "date",
    };
  }

  return undefined;
});

const {
  data: entriesData,
  status: entriesStatus,
  refetch: entriesRefetch,
} = useEntries(entriesArgs);
const entries = computed(() => (entriesData.value ? entriesData.value[0] : []));

const doLoading = ref(false);

const loading = computed(
  () =>
    journalsStatus.value === "pending" ||
    accountsStatus.value === "pending" ||
    entriesStatus.value === "pending" ||
    doLoading.value,
);

type Tab = "Accounts" | "Entries";

const tab = ref<Tab>("Accounts");

const journalReadonly = ref(true);

const showDeleteDialog = ref(false);

const journalValue = ref<JournalCardValue>();

const journalCommand = computed((): JournalCommandUpdate | undefined => {
  if (journalValue.value) {
    return {
      commandType: "journals:update",
      id: journalId.value,
      name: journalValue.value.name.state === "UPDATED" ? journalValue.value.name.value : undefined,
      description:
        journalValue.value.description.state === "UPDATED"
          ? journalValue.value.description.value
          : undefined,
      unit: journalValue.value.unit.state === "UPDATED" ? journalValue.value.unit.value : undefined,
      tags: journalValue.value.tags.state === "UPDATED" ? journalValue.value.tags.value : undefined,
    };
  }

  return undefined;
});

const submit = async () => {
  if (journalCommand.value) {
    doLoading.value = true;
    try {
      await journalApi.handleCommand(journalCommand.value);
      await journalsRefetch();
      cancel();
    } finally {
      doLoading.value = false;
    }
  }
};

const cancel = () => {
  journalReadonly.value = true;
  journalValue.value = undefined;
};
</script>

<template>
  <q-linear-progress v-if="loading" indeterminate></q-linear-progress>
  <span v-else-if="!journal">Not Found</span>
  <div v-else class="flex gap-4 w-full">
    <div class="w-1/4">
      <JournalCard
        :model-value="journal"
        :readonly="journalReadonly"
        @update:value="journalValue = $event"
        @submit="submit"
      >
        <template #actions>
          <template v-if="journalReadonly">
            <q-btn
              icon="edit"
              color="primary"
              label="Edit"
              @click="journalReadonly = false"
            ></q-btn>
            <q-btn
              flat
              icon="delete"
              color="negative"
              label="Delete"
              @click="showDeleteDialog = true"
            ></q-btn>
          </template>
          <template v-else>
            <q-btn
              icon="save"
              color="primary"
              label="Save"
              type="submit"
              :disable="!journalCommand"
            ></q-btn>
            <q-btn flat icon="cancel" label="Cancel" @click="cancel"></q-btn>
          </template>
        </template>
      </JournalCard>
    </div>
    <div class="flex-1">
      <q-card>
        <q-tabs v-model="tab" align="justify" narrow-indicator>
          <q-tab name="Accounts" label="Accounts" icon="account_balance" />
          <q-tab name="Entries" label="Entries" icon="comment_bank" />
        </q-tabs>
        <q-separator />
        <q-tab-panels v-model="tab" animated>
          <q-tab-panel name="Accounts">
            <AccountTable :model-value="accounts" :journal="journal"></AccountTable>
            <TestAgCharts></TestAgCharts>
          </q-tab-panel>
          <q-tab-panel name="Entries">
            <EntryTable
              :model-value="entries"
              :journal="journal"
              :accounts="accounts"
              @reload="entriesRefetch"
            ></EntryTable>
          </q-tab-panel>
        </q-tab-panels>
      </q-card>
    </div>
  </div>
  <JournalDeleteDialog
    v-if="journal"
    v-model="showDeleteDialog"
    :journal="journal"
    @reload="router.push({ name: ROUTE_JOURNALS })"
  ></JournalDeleteDialog>
</template>
