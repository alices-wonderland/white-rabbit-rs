<template>
  <v-slide-group class="h-full" show-arrows disabled>
    <div class="flex gap-x-1 items-center">
      <v-slide-group-item v-for="tag in tags" :key="tag">
        <v-chip @click="onTagClicked">{{ tag }}</v-chip>
      </v-slide-group-item>
    </div>
  </v-slide-group>
</template>

<script lang="ts" setup>
import { ICellRendererParams } from "@ag-grid-community/core";
import { computed } from "vue";

const props = defineProps<{
  params: ICellRendererParams;
}>();

const tags = computed(() => props.params.data.tags);

const onTagClicked = (params: any) => console.log("onTagClicked: ", params);
</script>
