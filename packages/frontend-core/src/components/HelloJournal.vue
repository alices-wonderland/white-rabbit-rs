<template>
  <div>
    <label>
      <strong>Name:</strong>
      <input v-model="queryName" type="text" />
    </label>
  </div>
  <button @click="loadNext">Load More</button>
  <div>
    <h2>Query:</h2>
    <code>
      <pre>{{ JSON.stringify(args, null, 2) }}</pre>
    </code>
  </div>
  <div>
    <h2>Journals:</h2>
    <div v-for="journal in journals" :key="journal.id">
      <div>
        <strong>Name:</strong>
        <span>{{ journal?.name }}</span>
      </div>
      <div>
        <strong>Description:</strong>
        <span>{{ journal.description }}</span>
      </div>
      <div>
        <strong>Permission:</strong>
        <span>{{ journal.permission }}</span>
      </div>
      <div>
        <strong>Unit:</strong>
        <span>{{ journal.unit }}</span>
      </div>
      <div>
        <strong>Admins:</strong>
        <span>{{ journal.admins }}</span>
      </div>
      <div>
        <strong>Members:</strong>
        <span>{{ journal.members }}</span>
      </div>
    </div>
  </div>
  <div>
    <h2>Included:</h2>
    <div v-for="inc in included.values()" :key="`${inc.modelType}:${inc.id}`">
      <div>
        <strong>Type:</strong>
        <span>{{ inc.modelType }}</span>
      </div>
      <div>
        <strong>Id:</strong>
        <span>{{ inc.id }}</span>
      </div>
      <div>
        <strong>Data:</strong>
        <code>{{ JSON.stringify(inc) }}</code>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { usePage } from "@core/composable";
import { ref, watch } from "vue";
import type { JournalQuery, JournalApi, JournalSort } from "@core/services";
import { Journal, JOURNAL_API_KEY } from "@core/services";

const queryName = ref("J");
const [args, journals, included, loadNext] = usePage<
  Journal,
  JournalQuery,
  JournalSort,
  JournalApi
>(JOURNAL_API_KEY, {
  query: { name: ["J", true] },
  size: 2,
  sort: [["name", "Asc"]],
});

watch(queryName, (newName) => {
  args.value = { ...args.value, query: { ...args.value.query, name: [newName, true] } };
});
</script>
