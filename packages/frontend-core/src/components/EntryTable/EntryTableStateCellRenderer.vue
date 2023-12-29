<script setup lang="ts">
import type { ICellEditorParams } from "@ag-grid-community/core";
import { ParentRow, type Row } from "./row";
import type { EntryStateItem, EntryType } from "@core/services";
import { computed } from "vue";

type Props = { readonly params: ICellEditorParams<Row, EntryStateItem> };

const props = defineProps<Props>();

const typeAndState = computed((): [EntryType, EntryStateItem] | undefined => {
  if (props.params.value) {
    return props.params.data instanceof ParentRow
      ? ["Record", props.params.value]
      : ["Check", props.params.value];
  }

  return undefined;
});
</script>

<template>
  <q-chip
    v-if="typeAndState"
    :color="typeAndState[1].type === 'Valid' ? 'positive' : 'negative'"
    text-color="white"
    :label="typeAndState[1].type"
  >
    <q-tooltip>
      <span v-if="typeAndState[1].type === 'Valid'">
        <strong>Value:</strong> {{ typeAndState[1].value }}
      </span>
      <div v-else-if="typeAndState[0] === 'Record'">
        <span>
          <strong>Left Side:</strong> {{ typeAndState[1].value[0] }}, <strong>Right Side:</strong>
          {{ typeAndState[1].value[1] }}
        </span>
        <ul class="list-disc list-inside">
          <li>
            <strong>Left Side:</strong> The sum of entry items for Account with
            <strong>type Asset & Expense</strong>
          </li>
          <li>
            <strong>Right Side:</strong> The sum with
            <strong>type Income, Liability & Equity</strong>
          </li>
        </ul>
      </div>
      <div v-else>
        <span>
          <strong>Expected Value:</strong> {{ typeAndState[1].value[0] }},
          <strong>Actual Value:</strong>
          {{ typeAndState[1].value[1] }}
        </span>
        <ul class="list-disc list-inside">
          <li>
            <strong>Expected Value:</strong> The value expected in this Account, assigned by users
            in this Check Entry
          </li>
          <li>
            <strong>Actual Value:</strong> The sum of this Account, from the start of this Journal,
            to the date of this Check Entry
          </li>
        </ul>
      </div>
    </q-tooltip>
  </q-chip>
</template>
