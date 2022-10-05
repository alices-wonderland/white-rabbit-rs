<template>
  <div v-if="isRecord">{{ name }}</div>
  <a v-else :href="`/accounts/${props.params.data.hierarchy[1]}`">{{ name }}</a>
</template>

<script lang="ts" setup>
import { ICellRendererParams } from "@ag-grid-community/core";
import { invoke } from "@tauri-apps/api/tauri";
import { computedAsync } from "@vueuse/core";
import { computed } from "vue";

const props = defineProps<{
  params: ICellRendererParams;
}>();

const isRecord = computed(() => props.params.data.hierarchy.length <= 1);
const accountId = computed(() => props.params.data.hierarchy[1]);

const name = computedAsync(
  async () => {
    if (isRecord.value) {
      return props.params.data.name;
    } else {
      const users = await invoke<any[]>("get_users", {
        input: {
          query: { role: "Owner" },
          sort: { field: "date", order: "Desc" },
        },
      });

      const account: any = await invoke("get_account_by_id", {
        operator: users[0].id,
        id: accountId.value,
      });

      return account.name;
    }
  },
  undefined,
  {
    onError: (e) => console.error("Error when get account name: ", e),
  }
);
</script>
