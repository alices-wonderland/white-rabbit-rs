<template>
  <label>
    <span>A:</span>
    <input v-model="a" type="number" />
  </label>
  <label>
    <span>B:</span>
    <input v-model="b" type="number" />
  </label>
  <label>
    <span>Id:</span>
    <input v-model="id" type="text" />
  </label>
  <label>
    <span>Name:</span>
    <input v-model="name" type="text" />
  </label>
  <label>
    <span>Role:</span>
    <select v-model="role">
      <option value="">-</option>
      <option value="User">User</option>
      <option value="Admin">Admin</option>
    </select>
  </label>
  <label>
    <span>Field:</span>
    <select v-model="field">
      <option value="id">ID</option>
      <option value="name">Name</option>
      <option value="role">Role</option>
    </select>
  </label>

  <label>
    <span>Order:</span>
    <select v-model="order">
      <option value="Asc">ASC</option>
      <option value="Desc">DSEC</option>
    </select>
  </label>

  <button @click="reset">Reset</button>
  <h2>Handle</h2>
  <HelloWorld :value="user"></HelloWorld>

  <h2>Handle</h2>
  <div v-for="u in users" :key="JSON.stringify(u)">
    <pre><code>{{ JSON.stringify(u, null, 2) }}</code></pre>
  </div>
</template>

<script setup lang="ts">
import { HelloWorld } from "@core/index";
import { invoke } from "@tauri-apps/api/tauri";
import { watchDebounced } from "@vueuse/core";
import { ref } from "vue";

const a = ref(0);
const b = ref(0);
const id = ref<string>("");
const name = ref<string>("");
const role = ref<string>("");
const field = ref<string>("id");
const order = ref<string>("Asc");
const user = ref<object>({});

const reset = () => {
  a.value = 0;
  b.value = 0;
  role.value = "";
  id.value = "";
};

const users = ref<object[]>([]);

watchDebounced(
  () => ({ a: a.value, b: b.value }),
  async ({ a, b }) => {
    user.value = await invoke<object>("user_create", {
      command: {
        name: `${a} + ${b}`,
        role: a + b <= 0 ? "Admin" : "User",
      },
    });
  },
  { debounce: 500, maxWait: 1000 }
);

watchDebounced(
  () => ({ id: id.value, name: name.value, role: role.value, field: field.value, order: order.value }),
  async ({ id, name, role, field, order }) => {
    try {
      users.value = await invoke<object[]>("user_find_all", {
        id: id
          .split(",")
          .map((s) => s.trim())
          .filter((s) => s.length > 0),
        name: name.trim().toLowerCase(),
        role: role === "" ? null : role,
        sort: [[field, order]],
      });
    } catch (e) {
      console.error("Errors when find all: ", e);
    }
  },
  { debounce: 500, maxWait: 1000 }
);
</script>
