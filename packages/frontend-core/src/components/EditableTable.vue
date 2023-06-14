<template>
  <ag-grid-vue
    class="ag-theme-alpine"
    :column-defs="columnDefs"
    :row-data="rowData"
    :default-col-def="defaultColDef"
  ></ag-grid-vue>
</template>

<script setup lang="ts">
import type { ColDef } from "@ag-grid-community/core";
import { AgGridVue } from "@ag-grid-community/vue3";
import { ref } from "vue";

const columnDefs = ref<ColDef[]>([
  { headerName: "Make", field: "make" },
  { headerName: "Model", field: "model" },
  { headerName: "Price", field: "price" },
]);

const defaultColDef: ColDef = {
  flex: 1,
  minWidth: 100,
  resizable: true,
  suppressMovable: true,
};

const rowData = ref([
  { make: "Toyota", model: "Celica", price: 35000 },
  { make: "Ford", model: "Mondeo", price: 32000 },
  { make: "Porsche", model: "Boxster", price: 72000 },
]);
</script>

<style scoped lang="scss">
.ag-theme-alpine {
  height: 40vh;
  width: 100%;
  resize: both;
  overflow: auto;
  padding: 10px;
}
</style>
