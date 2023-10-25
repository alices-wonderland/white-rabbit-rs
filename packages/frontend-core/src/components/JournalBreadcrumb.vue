<script setup lang="ts">
import { useRoute } from "vue-router";
import { useJournal } from "@core/composable";
import { computed } from "vue";
import { JOURNAL_ICON } from "@core/services";

const route = useRoute();

const journalId = computed(() => route.params["id"] as string);

const { model, loading } = useJournal(journalId);
</script>

<template>
  <q-toolbar inset>
    <q-linear-progress v-if="loading" indeterminate></q-linear-progress>
    <q-breadcrumbs v-else active-color="on-primary">
      <q-breadcrumbs-el label="Journals" to="/journals" :icon="JOURNAL_ICON"></q-breadcrumbs-el>
      <q-breadcrumbs-el
        v-if="model"
        :label="model.name"
        :to="`/journals/${model.id}`"
      ></q-breadcrumbs-el>
    </q-breadcrumbs>
  </q-toolbar>
</template>
