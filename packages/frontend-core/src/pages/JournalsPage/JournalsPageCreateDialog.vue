<script setup lang="ts">
import { JournalCard } from "@core/components/JournalCard";
import type { Value } from "@core/components/JournalCard";
import { computed, ref } from "vue";
import { JOURNAL_API_KEY } from "@core/services";
import type { JournalApi, JournalCommandCreate } from "@core/services";
import { useInject } from "@core/composable";

const props = defineProps<{
  readonly modelValue: boolean;
}>();

const emits = defineEmits<{
  "update:modelValue": [value: boolean];
  reload: [];
}>();

const journalApi = useInject<JournalApi>(JOURNAL_API_KEY);

const value = ref<Value>();
const loading = ref(false);

const show = computed({
  get: () => props.modelValue,
  set: (newShow) => {
    value.value = undefined;
    emits("update:modelValue", newShow);
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

const save = async () => {
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
    <JournalCard class="w-3/5" @update:value="value = $event" @submit="save">
      <template #title>
        <h6>Create New Journal</h6>
      </template>
      <template #actions>
        <q-btn
          icon="save"
          color="primary"
          :disable="!command"
          :loading="loading"
          label="Submit"
          type="submit"
        ></q-btn>
        <q-btn flat icon="cancel" label="Cancel" :loading="loading" @click="show = false"></q-btn>
      </template>
    </JournalCard>
  </q-dialog>
</template>
