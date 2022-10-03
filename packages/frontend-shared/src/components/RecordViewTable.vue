<template>
  <ag-grid-vue
    style="width: 500px; height: 200px"
    class="ag-theme-alpine"
    :columnDefs="columnDefs"
    :rowData="rowData"
  >
  </ag-grid-vue>
</template>

<script lang="ts" setup>
import { ref } from "vue";
import { AgGridVue } from "@ag-grid-community/vue3";

const columnDefs = ref([
  { headerName: "Make", field: "make" },
  { headerName: "Model", field: "model" },
  { headerName: "Price", field: "price" },
]);

const rowData = ref([
  { make: "Toyota", model: "Celica", price: 35000 },
  { make: "Ford", model: "Mondeo", price: 32000 },
  { make: "Porsche", model: "Boxster", price: 72000 },
]);
</script>

<style lang="scss">
@use "@ag-grid-community/styles" as ag;
@include ag.grid-styles(
  (
    theme: alpine,
  )
);
</style>
