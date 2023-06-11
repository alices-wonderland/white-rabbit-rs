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
    <h2>Accounts:</h2>
    <div v-for="account in accounts" :key="account.id">
      <div>
        <strong>Name:</strong>
        <span>{{ account?.name }}</span>
      </div>
      <div>
        <strong>Permission:</strong>
        <span>{{ account.permission }}</span>
      </div>
      <div>
        <strong>Description:</strong>
        <span>{{ account.description }}</span>
      </div>
      <div>
        <strong>Unit:</strong>
        <span>{{ account.unit }}</span>
      </div>
      <div>
        <strong>Type:</strong>
        <span>{{ account.type }}</span>
      </div>
      <div>
        <strong>Tags:</strong>
        <span>{{ account.tags }}</span>
      </div>
      <div>
        <strong>Journal:</strong>
        <span>{{ account.journal }}</span>
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
import type { AccountQuery, AccountSort, AccountApi } from "@core/services";
import { Account, ACCOUNT_API_KEY } from "@core/services";

const queryName = ref("A");
const [args, accounts, included, loadNext] = usePage<
  Account,
  AccountQuery,
  AccountSort,
  AccountApi
>(ACCOUNT_API_KEY, {
  query: { name: ["A", true] },
  size: 2,
  sort: [["name", "Asc"]],
});

watch(queryName, (newName) => {
  args.value = { ...args.value, query: { ...args.value.query, name: [newName, true] } };
});
</script>
