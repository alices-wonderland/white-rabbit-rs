<template>
  <v-theme-provider :theme="theme.global.name.value">
    <v-app>
      <v-app-bar color="primary">
        <v-app-bar-title>White Rabbit {{ t("test") }}</v-app-bar-title>
        <template #append>
          <v-btn :icon="darkIcon" @click="toggleDark()"></v-btn>
          <v-menu>
            <template #activator="{ props: transProps }">
              <v-btn :icon="mdiTranslate" v-bind="transProps"></v-btn>
            </template>
            <v-list>
              <v-list-item v-for="[id, text] in localeOptions" :key="id" :value="id">
                <v-list-item-title @click="changeLocale(id)">{{ text }}</v-list-item-title>
              </v-list-item>
            </v-list>
          </v-menu>
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
import { computed, ref } from "vue";
import { useTheme } from "vuetify";
import { mdiWeatherNight, mdiWeatherSunny, mdiTranslate } from "@mdi/js";
import { useI18n } from "vue-i18n";

const { locale, t } = useI18n();
const theme = useTheme();

const toggleDark = () => {
  theme.global.name.value = theme.global.name.value === "dark" ? "light" : "dark";
};

const darkIcon = computed(() =>
  theme.global.name.value === "dark" ? mdiWeatherNight : mdiWeatherSunny
);

const localeOptions = ref([
  ["en", "English"],
  ["zh-Hans", "简体中文"],
]);

const changeLocale = (newLocale: string) => {
  locale.value = newLocale;
  document.querySelector("html")?.setAttribute("lang", newLocale);
};
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
