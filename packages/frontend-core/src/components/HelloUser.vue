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
    <h2>Users:</h2>
    <div v-for="user in users" :key="user.id">
      <div>
        <strong>Name:</strong>
        <span>{{ user?.name }}</span>
      </div>
      <div>
        <strong>Role:</strong>
        <span>{{ user.role }}</span>
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
import type { UserApi, UserQuery, UserSort } from "@core/services";
import { User, USER_API_KEY } from "@core/services";

const queryName = ref("U");
const [args, users, included, loadNext] = usePage<User, UserQuery, UserSort, UserApi>(
  USER_API_KEY,
  {
    query: { name: ["U", true] },
    size: 2,
    sort: [["name", "Asc"]],
  }
);

watch(queryName, (newName) => {
  args.value = { ...args.value, query: { ...args.value.query, name: [newName, true] } };
});
</script>
