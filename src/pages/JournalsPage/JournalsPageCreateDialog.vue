<script setup lang="ts">
import { computed, ref } from "vue";
import { useQueryClient } from "@tanstack/vue-query";

import { JournalCard } from "components/JournalCard";
import type { Value } from "components/JournalCard";
import { type JournalCommandCreate } from "src/services/journal";
import { useJournalCommand } from "src/composable/useCommand";

const modelValue = defineModel<boolean>("modelValue", { required: true });

const queryClient = useQueryClient();

const value = ref<Value>();

const show = computed({
  get: () => modelValue.value,
  set: (newShow) => {
    value.value = undefined;
    modelValue.value = newShow;
  },
});

const command = computed<JournalCommandCreate | undefined>(() => {
  if (value.value) {
    return {
      commandType: "journals:create",
      name: value.value.name.value,
      description: value.value.description.value,
      unit: value.value.unit.value,
      tags: value.value.tags.value,
    };
  }

  return undefined;
});

const { mutateAsync: createAsync, isPending: createPending } = useJournalCommand({
  async onSuccess() {
    show.value = false;
    await queryClient.invalidateQueries({ queryKey: ["journals"] });
  },
});
</script>

<template>
  <q-dialog v-model="show">
    <JournalCard
      class="w-3/5"
      @update:value="value = $event"
      @submit="command && createAsync(command)"
    >
      <template #title>
        <h6>Create New Journal</h6>
      </template>
      <template #actions>
        <q-btn
          icon="save"
          color="primary"
          :disable="!command"
          :loading="createPending"
          label="Submit"
          type="submit"
        ></q-btn>
        <q-btn
          flat
          icon="cancel"
          label="Cancel"
          :loading="createPending"
          @click="show = false"
        ></q-btn>
      </template>
    </JournalCard>
  </q-dialog>
</template>
