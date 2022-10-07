<template>
  <v-text-field v-model="name" autofocus></v-text-field>
</template>

<script lang="ts">
import { ICellRendererParams } from "@ag-grid-community/core";
import { defineComponent, PropType } from "vue";

type Data = {
  name: string;
};

export default defineComponent({
  props: {
    params: {
      type: Object as PropType<ICellRendererParams>,
      required: true,
    },
  },
  data(): Data {
    return {
      name: this.params.data.name,
    };
  },
  // https://vuejs.org/api/sfc-script-setup.html#defineexpose
  // Composition API will remove all unused functions,
  //so for Ag Grid Cell, we must use the Options API
  methods: {
    getValue(): string {
      return this.name;
    },
  },
});
</script>
