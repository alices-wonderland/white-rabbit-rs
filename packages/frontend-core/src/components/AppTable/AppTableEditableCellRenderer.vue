<script setup lang="ts" generic="R">
import type { ICellRendererParams } from "@ag-grid-community/core";
import type { EditableValue } from "./index";
import { computed } from "vue";

const props = defineProps<{
  readonly params: ICellRendererParams<R> & { readonly editableValue?: EditableValue<string> };
}>();

const value = computed(() => props.params.editableValue);
</script>

<template>
  <div v-if="value" class="flex gap-1 w-full h-full items-center overflow-hidden">
    <template v-if="value.state === 'NORMAL'">
      <slot v-if="$slots['default']" :value="value.value"></slot>
      <span v-else>{{ value.value }}</span>
    </template>
    <template v-else-if="value.state === 'NEW'">
      <q-icon name="new_releases"></q-icon>
      <slot v-if="$slots['default']" :value="value.value"></slot>
      <span v-else>{{ value.value }}</span>
    </template>
    <template v-else-if="value.state === 'UPDATED'">
      <slot v-if="$slots['existing']" name="existing" :value="value.existing"></slot>
      <q-badge v-else color="warning" :label="value.existing">
        <q-tooltip>
          <span>
            <strong>Existing:</strong>
            <span>{{ value.existing }}</span>
          </span>
        </q-tooltip>
      </q-badge>
      <q-icon name="arrow_right"></q-icon>
      <slot v-if="$slots['current']" name="current" :value="value.value"></slot>
      <q-badge v-else color="positive" :label="value.value">
        <q-tooltip>
          <span>
            <strong>Current:</strong>
            <span>{{ value.value }}</span>
          </span>
        </q-tooltip>
      </q-badge>
    </template>
    <q-badge
      v-if="value.state !== 'NORMAL'"
      class="absolute top-0.5 right-0.5"
      align="top"
      rounded
      color="positive"
    ></q-badge>
  </div>
</template>
