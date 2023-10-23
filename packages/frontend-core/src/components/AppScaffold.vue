<template>
  <q-layout view="lHh lpr lff">
    <q-header elevated>
      <q-toolbar>
        <q-toolbar-title>Header {{ t("test") }} {{ quasar.lang.label.close }}</q-toolbar-title>
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

      <q-toolbar inset>
        <q-breadcrumbs active-color="white">
          <q-breadcrumbs-el label="Home" icon="home" />
          <q-breadcrumbs-el label="Components" icon="widgets" />
          <q-breadcrumbs-el label="Toolbar" />
        </q-breadcrumbs>
      </q-toolbar>
    </q-header>

    <q-footer>
      <q-toolbar>
        <q-toolbar-title>Footer</q-toolbar-title>
      </q-toolbar>
    </q-footer>

    <q-page-container>
      <q-page padding>
        <q-btn color="primary" label="Primary"></q-btn>
        <q-btn color="secondary" label="Secondary" />
        <q-btn color="accent" label="Accent" />
        <q-btn color="dark" label="Dark" />
        <q-btn color="positive" label="Positive" />
        <q-btn color="negative" label="Negative" />
        <q-btn color="info" label="Info" />
        <q-btn color="warning" label="Warning" />
        <slot></slot>
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

const { locale, t } = useI18n();
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
