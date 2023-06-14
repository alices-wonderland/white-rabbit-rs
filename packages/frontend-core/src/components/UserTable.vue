<template>
  <button @click="onAddUser">Add User</button>
  <ag-grid-vue
    class="ag-theme-alpine"
    :column-defs="columnDefs"
    :default-col-def="defaultColDef"
    :row-data="rows"
  ></ag-grid-vue>
</template>

<script setup lang="ts">
import { AgGridVue } from "@ag-grid-community/vue3";
import type { ColDef } from "@ag-grid-community/core";
import { useInject } from "@core/composable";
import type { Permission, Role, UserApi } from "@core/services";
import { USER_API_KEY } from "@core/services";
import { onMounted, ref, watch } from "vue";

const columnDefs = ref<ColDef[]>([
  { headerName: "ID", field: "id" },
  { headerName: "Name", field: "name" },
  { headerName: "Role", field: "role" },
  { headerName: "Permission", field: "permission" },
]);

const defaultColDef: ColDef = {
  flex: 1,
  minWidth: 100,
  resizable: true,
  suppressMovable: true,
};

const rows = ref<Array<{ id?: string; permission?: Permission; name?: string; role?: Role }>>([]);

onMounted(async () => {
  const userApi = useInject<UserApi>(USER_API_KEY);
  const users = await userApi.findAll({ query: {} });
  const newRows = users[0].map((user) => ({
    id: user.id,
    permission: user.permission,
    name: user.name,
    role: user.role,
  }));
  rows.value = newRows;
});

const onAddUser = () =>
  (rows.value = [{ permission: "ReadWrite", name: "New User", role: "User" }, ...rows.value]);

watch(rows, (newRows) => console.log("New Rows: ", newRows));
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
