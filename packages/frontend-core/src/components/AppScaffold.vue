<template>
  <v-theme-provider :theme="theme.global.name.value">
    <v-app>
      <v-app-bar color="primary">
        <v-app-bar-title>White Rabbit</v-app-bar-title>
        <template #append>
          <v-btn :icon="darkIcon" @click="toggleDark()"></v-btn>
        </template>
      </v-app-bar>
      <v-main>
        <v-btn-group>
          <v-btn color="primary">Primary</v-btn>
          <v-btn color="secondary">Secondary</v-btn>
          <v-btn color="tertiary">Tertiary</v-btn>
          <v-btn color="error">Error</v-btn>
        </v-btn-group>
        <slot></slot>
      </v-main>
    </v-app>
  </v-theme-provider>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { useTheme } from "vuetify";

const theme = useTheme();
const toggleDark = () => {
  theme.global.name.value = theme.global.name.value === "dark" ? "light" : "dark";
};
const darkIcon = computed(() => (theme.global.name.value === "dark" ? "dark_mode" : "light_mode"));
</script>

<style scoped lang="scss">
.v-main {
  margin: 1rem;
}

.v-btn-group {
  display: flex;
  gap: 1rem;
}
</style>
