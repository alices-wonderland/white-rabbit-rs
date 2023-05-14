<template>
  <label>
    <span>A:</span>
    <input v-model="a" type="number" />
  </label>
  <label>
    <span>B:</span>
    <input v-model="b" type="number" />
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
const user = ref<object>({});

const reset = () => {
  a.value = 0;
  b.value = 0;
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
    try {
      users.value = await invoke<object[]>("user_find_all");
    } catch (e) {
      console.error("Errors when find all: ", e);
    }
  },
  { debounce: 500, maxWait: 1000 }
);
</script>
