<script setup lang="ts">
import type { RouterLinkProps } from "vue-router";

const props = defineProps<RouterLinkProps>();
</script>

<template>
  <router-link v-slot="{ href, navigate }" v-bind="props" custom>
    <a :href="href" class="no-underline hover:underline hover:cursor-pointer" @click="navigate">
      <slot></slot>
    </a>
  </router-link>
</template>
