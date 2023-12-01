<script setup lang="ts">
import { computed, ref } from "vue";
import { Journal, type JournalCommandDelete } from "@core/services";
import { useJournalCommand } from "@core/composable";
import { useQueryClient } from "@tanstack/vue-query";

const queryClient = useQueryClient();

const props = defineProps<{
  readonly modelValue: boolean;
  readonly journal: Journal;
}>();

const emits = defineEmits<{
  "update:modelValue": [value: boolean];
  reload: [];
}>();

const value = ref<string>();

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

const { mutateAsync: deleteAsync, isPending: deletePending } = useJournalCommand({
  async onSuccess() {
    emits("reload");
    show.value = false;
    await queryClient.invalidateQueries({ queryKey: ["journals"] });
  },
});
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
          :loading="deletePending"
          label="Delete"
          @click="command && deleteAsync(command)"
        >
        </q-btn>
        <q-btn
          icon="cancel"
          flat
          label="Cancel"
          :loading="deletePending"
          @click="show = false"
        ></q-btn>
      </q-card-actions>
    </q-card>
  </q-dialog>
</template>
