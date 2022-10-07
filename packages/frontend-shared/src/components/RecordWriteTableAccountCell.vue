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
import { ICellRendererParams } from "@ag-grid-community/core";
import { invoke } from "@tauri-apps/api/tauri";
import { computedAsync } from "@vueuse/core";
import { defineComponent, PropType, ref, toRefs } from "vue";

interface ICellParams extends ICellRendererParams {
  readonly userId: string;
}

export default defineComponent({
  props: {
    params: {
      type: Object as PropType<ICellParams>,
      required: true,
    },
  },
  setup(props) {
    const { params } = toRefs(props);
    const journal: string | undefined =
      params.value.node?.parent?.data?.journal;

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const items = computedAsync<any[]>(
      async () =>
        await invoke("get_accounts", {
          operator: params.value.userId,
          input: { query: { journal } },
        }),
      [],
      {
        onError: (e) => console.error("Error when loading account items: ", e),
      }
    );

    const account = ref(
      items.value.find((item) => item.id === params.value.data.hierarchy[1])
    );

    return { account, items };
  },
  // https://vuejs.org/api/sfc-script-setup.html#defineexpose
  // Composition API will remove all unused functions,
  //so for Ag Grid Cell, we must use the Options API
  methods: {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    getValue(): any {
      return this.account;
    },
  },
});
</script>
