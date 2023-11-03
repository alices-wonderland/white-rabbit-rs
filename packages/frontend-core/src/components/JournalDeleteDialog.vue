<script setup lang="ts">
import { computed, ref } from "vue";
import { Journal, JOURNAL_API_KEY } from "@core/services";
import type { JournalApi, JournalCommandDelete } from "@core/services";
import { useInject } from "@core/composable";

const props = defineProps<{
  readonly modelValue: boolean;
  readonly journal: Journal;
}>();

const journalApi = useInject<JournalApi>(JOURNAL_API_KEY);

const emits = defineEmits<{
  "update:modelValue": [value: boolean];
  reload: [];
}>();

const value = ref<string>();
const loading = ref(false);

const show = computed({
  get: () => props.modelValue,
  set: (newShow) => {
    value.value = undefined;
    emits("update:modelValue", newShow);
  },
});

const command = computed<JournalCommandDelete | undefined>(() => {
  if (value.value === props.journal.name) {
    return {
      commandType: "journals:delete",
      id: [props.journal.id],
    };
  }

  return undefined;
});

const confirm = async () => {
  if (!command.value) {
    return;
  }

  try {
    loading.value = true;
    await journalApi.handleCommand(command.value);
    emits("reload");
    show.value = false;
  } finally {
    loading.value = false;
  }
};
</script>

<template>
  <q-dialog v-model="show">
    <q-card>
      <q-card-section>
        <h6>Delete Journal[{{ journal.name }}]</h6>
      </q-card-section>
      <q-card-section>
        <div>
          Journal[{{ journal.name }}] will be deleted. Please <strong>input the name</strong> to
          confirm this operation.
        </div>
        <div class="text-negative">
          This operation will <strong>remove all related Accounts and Entries</strong>!
        </div>
        <div class="text-negative">This operation <strong>is not recoverable</strong>!</div>
        <q-input v-model="value" label="Journal Name"> </q-input>
      </q-card-section>
      <q-card-actions>
        <q-btn
          icon="delete"
          flat
          color="negative"
          :disable="!command"
          :loading="loading"
          label="Delete"
          @click="confirm"
        >
        </q-btn>
        <q-btn icon="cancel" flat label="Cancel" :loading="loading" @click="show = false"></q-btn>
      </q-card-actions>
    </q-card>
  </q-dialog>
</template>
