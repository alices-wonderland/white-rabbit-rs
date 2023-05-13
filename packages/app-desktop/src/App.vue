<template>
  <label>
    <span>A:</span>
    <input v-model="a" type="number" />
  </label>
  <label>
    <span>B:</span>
    <input v-model="b" type="number" />
  </label>
  <HelloWorld :value="result"></HelloWorld>
</template>

<script setup lang="ts">
import HelloWorld from "@core/HelloWorld.vue";
import { invoke } from "@tauri-apps/api/tauri";
import { computedAsync } from "@vueuse/core";
import { ref } from "vue";

const a = ref(0);
const b = ref(0);

const result = computedAsync(() => invoke<number>("add", { a: a.value, b: b.value }), 0);
</script>
