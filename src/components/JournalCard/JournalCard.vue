<script setup lang="ts">
import { computed, ref, watch } from "vue";
import type { ValidationRule } from "quasar";
import sortBy from "lodash/sortBy";
import sortedUniq from "lodash/sortedUniq";
import isEqual from "lodash/isEqual";
import { watchDebounced } from "@vueuse/core";

import type { FieldState } from "components/types";
import { Journal, JOURNAL_ICON } from "src/services/journal";
import AppLink from "components/AppLink.vue";

export type Value = {
  readonly name: FieldState<string>;
  readonly description: FieldState<string>;
  readonly unit: FieldState<string>;
  readonly tags: FieldState<string[]>;
};

const props = defineProps<{
  readonly modelValue?: Journal;
  readonly readonly?: boolean;
  readonly loading?: boolean;
}>();

const emits = defineEmits<{
  "update:value": [value: Value];
  submit: [];
}>();

const name = ref<string>("");
const nameRules: ValidationRule<string>[] = [
  (value) => {
    const trimmed = value.trim();
    if (trimmed.length < 6 || trimmed.length > 63) {
      return "The length of Field[name] should between 6 to 63";
    }
    return true;
  },
];

const description = ref<string>("");
const descriptionRules: ValidationRule<string>[] = [
  (value) => {
    const trimmed = value.trim();
    if (trimmed.length > 1023) {
      return "The length of Field[description] should smaller than 1023";
    }
    return true;
  },
];

const unit = ref<string>("");
const unitRules: ValidationRule<string>[] = [
  (value) => {
    const trimmed = value.trim();
    if (trimmed.length < 2 || trimmed.length > 15) {
      return "The length of Field[unit] should between 2 to 15";
    }

    return true;
  },
];

const tags = ref<string[]>([]);

const reset = () => {
  if (props.modelValue) {
    name.value = props.modelValue.name;
    description.value = props.modelValue.description;
    unit.value = props.modelValue.unit;
    tags.value = sortBy(props.modelValue.tags);
  } else {
    name.value = "";
    description.value = "";
    unit.value = "";
    tags.value = [];
  }
};

const value = computed((): Value => {
  const currentName = name.value.trim();
  const currentDescription = description.value.trim();
  const currentUnit = unit.value.trim();
  const currentTags = sortedUniq(sortBy(tags.value.map((tag) => tag.trim()).filter((s) => !!s)));

  if (props.modelValue) {
    const existingTags = sortedUniq(sortBy(props.modelValue.tags));
    return {
      name:
        currentName === props.modelValue.name
          ? {
              state: "NORMAL",
              value: currentName,
            }
          : {
              state: "UPDATED",
              value: currentName,
              existing: props.modelValue.name,
            },
      description:
        currentDescription === props.modelValue.description
          ? {
              state: "NORMAL",
              value: currentDescription,
            }
          : {
              state: "UPDATED",
              value: currentDescription,
              existing: props.modelValue.description,
            },
      unit:
        currentUnit === props.modelValue.unit
          ? {
              state: "NORMAL",
              value: currentUnit,
            }
          : {
              state: "UPDATED",
              value: currentUnit,
              existing: props.modelValue.unit,
            },
      tags: isEqual(currentTags, existingTags)
        ? {
            state: "NORMAL",
            value: currentTags,
          }
        : {
            state: "UPDATED",
            value: currentTags,
            existing: existingTags,
          },
    };
  }

  return {
    name: {
      state: "NORMAL",
      value: currentName,
    },
    description: {
      state: "NORMAL",
      value: currentDescription,
    },
    unit: {
      state: "NORMAL",
      value: currentUnit,
    },
    tags: {
      state: "NORMAL",
      value: currentTags,
    },
  };
});

watchDebounced(
  value,
  (newValue, oldValue) => {
    if (!isEqual(newValue, oldValue)) {
      emits("update:value", newValue);
    }
  },
  { debounce: 100 },
);

watch(
  () => [props.modelValue?.id, props.readonly],
  () => {
    reset();
  },
  { immediate: true },
);

const submit = () => {
  emits("submit");
};
</script>

<template>
  <q-card>
    <q-card-section>
      <slot name="title">
        <h6 v-if="modelValue" class="flex items-center gap-2">
          <q-icon :name="JOURNAL_ICON"></q-icon>
          <AppLink :to="`/journals/${modelValue.id}`">
            {{ modelValue.name }}
          </AppLink>
        </h6>
      </slot>
    </q-card-section>
    <q-separator inset />
    <q-card-section>
      <q-form @submit="submit" @reset="reset">
        <q-input
          v-model="name"
          label="Name"
          :readonly="readonly"
          :loading="loading"
          :rules="nameRules"
          clearable
          bottom-slots
        >
          <template #hint>
            <span>
              The name of this journal, <strong class="text-negative">required</strong>.
            </span>
          </template>
        </q-input>
        <q-input
          v-model="description"
          rows="3"
          label="Description"
          type="textarea"
          :autogrow="readonly"
          :readonly="readonly"
          :loading="loading"
          :rules="descriptionRules"
          clearable
          bottom-slots
        >
          <template #hint>
            <span> A brief description of this journal. </span>
          </template>
        </q-input>
        <q-input
          v-model="unit"
          label="Unit"
          :readonly="readonly"
          :loading="loading"
          :rules="unitRules"
          clearable
          bottom-slots
        >
          <template #hint>
            <span>
              The currency of this journal, <strong class="text-negative">required</strong>.
            </span>
          </template>
        </q-input>
        <q-select
          v-model="tags"
          label="Tags"
          use-chips
          multiple
          use-input
          new-value-mode="add-unique"
          clearable
          bottom-slots
          :readonly="readonly"
          :loading="loading"
        >
          <template #hint>
            <span> Tags of this journal, used for full-searching. </span>
          </template>
        </q-select>

        <q-card-actions>
          <slot name="actions"></slot>
        </q-card-actions>
      </q-form>
    </q-card-section>
  </q-card>
</template>
