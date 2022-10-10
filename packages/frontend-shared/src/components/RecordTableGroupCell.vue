<template>
  <div v-if="isRecord || props.params.plainText">{{ name }}</div>
  <a v-else :href="`/accounts/${props.params.data.hierarchy[1]}`">{{ name }}</a>
</template>

<script lang="ts" setup>
import { ICellRendererParams } from "@ag-grid-community/core";
import { useAccountApi } from "@shared/hooks";
import { computedAsync } from "@vueuse/core";
import { computed } from "vue";

interface CellParams extends ICellRendererParams {
  readonly plainText?: boolean;
}

const props = defineProps<{
  params: CellParams;
}>();

const isRecord = computed(() => props.params.data.hierarchy.length <= 1);
const accountId = computed(() => props.params.data.hierarchy[1]);
const accountApi = useAccountApi();

const name = computedAsync(
  async () => {
    if (isRecord.value) {
      return props.params.data.name ?? props.params.data.data.name;
    } else {
      const account = await accountApi.findById(accountId.value);
      return account?.name;
    }
  },
  undefined,
  {
    onError: (e) => console.error("Error when get account name: ", e),
  }
);
</script>
