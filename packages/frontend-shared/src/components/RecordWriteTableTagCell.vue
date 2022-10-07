<template>
  <v-combobox
    v-model="tags"
    v-model:search-input="search"
    :items="items"
    chips
    closable-chips
    multiple
    density="compact"
    variant="plain"
  >
    <template #chip="{ index, props: chipProps }">
      <v-chip v-if="index < maxTags" v-bind="chipProps" />
      <small v-else-if="index === maxTags" class="italic self-center">
        (+{{ tags.length - maxTags }} others)
      </small>
    </template>
  </v-combobox>
</template>

<script lang="ts">
import { ICellRendererParams } from "@ag-grid-community/core";
import { defineComponent, PropType } from "vue";

type Data = {
  search?: string;
  tags: string[];
  items: string[];
  readonly maxTags: number;
};

export default defineComponent({
  // type inference enabled
  props: {
    params: {
      type: Object as PropType<ICellRendererParams>,
      required: true,
    },
  },
  data(): Data {
    return {
      search: undefined,
      tags: this.params.data.tags,
      items: this.params.data.tags,
      maxTags: 3,
    };
  },
  watch: {
    search(val, prev) {
      if (val) {
        this.items = [val].concat(this.items.filter((item) => item !== prev));
      }
    },
  },
  methods: {
    getValue() {
      return this.tags;
    },
  },
});
</script>
