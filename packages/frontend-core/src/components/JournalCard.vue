<script setup lang="ts">
import { Journal, JOURNAL_ICON } from "@core/services";
import { onMounted, ref, watch } from "vue";
import sortBy from "lodash/sortBy";
import { useRouter } from "vue-router";

const router = useRouter();

const props = defineProps<{
  readonly modelValue?: Journal;
  readonly readonly?: boolean;
}>();

watch(props, () => {
  reset();
});

onMounted(() => {
  reset();
});

const name = ref<string>("");
const description = ref<string>("");
const unit = ref<string>("");
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
</script>

<template>
  <q-card>
    <template v-if="modelValue">
      <q-card-section>
        <h6 class="flex items-center gap-2">
          <q-icon :name="JOURNAL_ICON"></q-icon>
          <span
            class="hover:underline hover:cursor-pointer"
            @click="router.push(`/journals/${modelValue.id}`)"
          >
            {{ modelValue.name }}
          </span>
        </h6>
      </q-card-section>
      <q-separator inset />
    </template>
    <q-card-section>
      <q-form>
        <q-input
          v-model="name"
          label="Name"
          :readonly="readonly"
          :bottom-slots="!readonly"
          clearable
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
          :bottom-slots="!readonly"
          clearable
        >
          <template #hint>
            <span> A brief description of this journal. </span>
          </template>
        </q-input>
        <q-input
          v-model="unit"
          label="Unit"
          :readonly="readonly"
          :bottom-slots="!readonly"
          clearable
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
          hide-dropdown-icon
          new-value-mode="add-unique"
          clearable
          :readonly="readonly"
          :bottom-slots="!readonly"
        ></q-select>
      </q-form>
    </q-card-section>
    <q-card-actions v-if="$slots['actions']">
      <slot name="actions"></slot>
    </q-card-actions>
  </q-card>
</template>
