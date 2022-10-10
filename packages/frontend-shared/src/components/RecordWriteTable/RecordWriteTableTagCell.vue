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
import { defineComponent, PropType, ref, toRefs, watch } from "vue";
import { RecordRow } from "./types";

export default defineComponent({
  // type inference enabled
  props: {
    params: {
      type: Object as PropType<ICellRendererParams<RecordRow>>,
      required: true,
    },
  },
  setup(props) {
    const { params } = toRefs(props);
    const tags = ref([...(params.value.data?.data?.tags ?? [])]);
    const search = ref<string>();
    const items = ref([...(params.value.data?.data?.tags ?? [])]);
    const maxTags = ref(3);

    watch(search, (val, prev) => {
      if (val) {
        items.value = [val].concat(items.value.filter((item) => item !== prev));
      }
    });

    return {
      search,
      tags,
      items,
      maxTags,
    };
  },
  methods: {
    getValue() {
      return this.tags;
    },
  },
});
</script>
