<template>
  <q-layout view="lHh lpr lff">
    <q-header elevated>
      <q-toolbar>
        <q-toolbar-title>Wonderland White Rabbit</q-toolbar-title>
        <q-btn flat round dense :icon="darkIcon" @click="toggleDark" />
        <q-btn flat round dense icon="translate">
          <q-menu auto-close>
            <q-list>
              <q-item
                v-for="[code, name] in localeItems"
                :key="code"
                clickable
                @click="changeLocale(code)"
              >
                <q-item-section>{{ name }}</q-item-section>
              </q-item>
            </q-list>
          </q-menu>
        </q-btn>
      </q-toolbar>

      <router-view name="toolbar"></router-view>
    </q-header>

    <q-page-container>
      <q-page padding class="flex justify-center">
        <router-view />
      </q-page>
    </q-page-container>
  </q-layout>
</template>

<script setup lang="ts">
import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";
import { useQuasar } from "quasar";
import langEnUS from "quasar/lang/en-US";
import langZhCH from "quasar/lang/zh-CN";

const { locale } = useI18n();
const quasar = useQuasar();

const toggleDark = () => {
  quasar.dark.toggle();
};

const darkIcon = computed(() => (quasar.dark.isActive ? "dark_mode" : "light_mode"));

const localeItems = ref([
  ["en", "English"],
  ["zh-Hans", "简体中文"],
]);

const changeLocale = (newLocale: string) => {
  locale.value = newLocale;
  if (newLocale === "en") {
    quasar.lang.set(langEnUS);
  } else if (newLocale === "zh-Hans") {
    quasar.lang.set(langZhCH);
  }
  document.querySelector("html")?.setAttribute("lang", newLocale);
};
</script>
