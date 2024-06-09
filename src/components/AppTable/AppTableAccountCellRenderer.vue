<script setup lang="ts" generic="R">
import type { ICellRendererParams } from "@ag-grid-community/core";
import { computed } from "vue";

import { type FieldState, NULL_PLACEHOLDER } from "components/types";
import { Account } from "src/services/account";

import AppTableEditableCellRenderer from "./AppTableEditableCellRenderer.vue";

const props = defineProps<{
  readonly params: ICellRendererParams<R, string> & {
    readonly accounts: Map<string, Account>;
    readonly fieldState?: FieldState<string>;
  };
}>();

const state = computed((): FieldState<string> | undefined => {
  const accounts = props.params.accounts;
  const fieldState = props.params.fieldState;
  if (fieldState?.state === "NORMAL") {
    const current = accounts.get(fieldState.value);
    return {
      state: "NORMAL",
      value: current?.name ?? NULL_PLACEHOLDER,
    };
  } else if (fieldState?.state === "UPDATED") {
    const current = accounts.get(fieldState.value);
    const existing = accounts.get(fieldState.existing);
    return {
      state: "UPDATED",
      value: current?.name ?? NULL_PLACEHOLDER,
      existing: existing?.name ?? NULL_PLACEHOLDER,
    };
  }
  return undefined;
});
</script>

<template>
  <AppTableEditableCellRenderer v-if="state" :params="{ ...params, fieldState: state }">
    <q-chip :label="state.value" icon="account_balance"></q-chip>
  </AppTableEditableCellRenderer>
</template>
