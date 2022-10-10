<template>
  <v-select
    v-model="account"
    :items="items"
    item-title="name"
    item-value="id"
    density="compact"
    return-object
  >
  </v-select>
</template>

<script lang="ts">
import { ICellRendererParams, RowNode } from "@ag-grid-community/core";
import { useAccountApi } from "@shared/hooks";
import { Account } from "@shared/models";
import { computedAsync } from "@vueuse/core";
import { defineComponent, PropType, ref, toRefs } from "vue";
import { RecordItemRow, RecordRow } from "./types";

interface Params extends ICellRendererParams<RecordItemRow> {
  readonly userId: string;
}

export default defineComponent({
  props: {
    params: {
      type: Object as PropType<Params>,
      required: true,
    },
  },
  setup(props) {
    const { params } = toRefs(props);

    const account = ref<Account | undefined>(params.value.data?.data?.account);

    const accountApi = useAccountApi();

    const items = computedAsync<Account[]>(
      async () => {
        const accounts = await accountApi.findAll({
          query: {
            journal: (params.value.node.parent as unknown as RowNode<RecordRow>)
              .data?.data?.journal?.id,
          },
        });
        return accounts.sort((a, b) => a.name.localeCompare(b.name));
      },
      [],
      {
        onError: (e) => console.error("Error when loading account items: ", e),
      }
    );

    return { account, items };
  },
  // https://vuejs.org/api/sfc-script-setup.html#defineexpose
  // Composition API will remove all unused functions,
  //so for Ag Grid Cell, we must use the Options API
  methods: {
    getValue(): Account | undefined {
      return this.account;
    },
  },
});
</script>
