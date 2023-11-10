<script setup lang="ts">
import { useRoute } from "vue-router";
import { computed, ref } from "vue";
import { useAccounts, useInject, useJournals } from "@core/composable";
import { JournalCard } from "@core/components/JournalCard";
import type { Value as JournalCardValue } from "@core/components/JournalCard";
import { AccountTable } from "@core/components/AccountTable";
import { JOURNAL_API_KEY } from "@core/services";
import type {
  AccountQuery,
  AccountSort,
  FindAllArgs,
  JournalApi,
  JournalCommandUpdate,
} from "@core/services";
import JournalDeleteDialog from "@core/components/JournalDeleteDialog.vue";

const route = useRoute();

const { models: journals, loading: journalsLoading, reload: journalsReload } = useJournals({});

const journalApi = useInject<JournalApi>(JOURNAL_API_KEY);

const journalId = computed(() => route.params["id"] as string);

const journal = computed(() => journals.value.find((model) => model.id === journalId.value));

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
const {
  models: accounts,
  loading: accountsLoading,
  reload: accountsReload,
} = useAccounts(accountArgs);

const doLoading = ref(false);

const loading = computed(() => journalsLoading.value || accountsLoading.value || doLoading.value);

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
      await journalsReload();
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
          <q-tab name="Accounts" label="Accounts" />
          <q-tab name="Entries" label="Entries" />
        </q-tabs>
        <q-separator />
        <q-tab-panels v-model="tab" animated>
          <q-tab-panel name="Accounts">
            <AccountTable
              :model-value="accounts"
              :journal="journal"
              @reload="accountsReload"
            ></AccountTable>
          </q-tab-panel>
          <q-tab-panel name="Entries">
            <div class="text-h6">Entries</div>
            Lorem ipsum dolor sit amet consectetur adipisicing elit.
          </q-tab-panel>
        </q-tab-panels>
      </q-card>
    </div>
  </div>
  <JournalDeleteDialog
    v-if="journal"
    v-model="showDeleteDialog"
    :journal="journal"
  ></JournalDeleteDialog>
</template>
