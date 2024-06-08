<template>
  <q-page class="row items-center justify-evenly">
    <example-component :title="configStore.apiUrlBase" active :todos="todos" :meta="meta" />
    <code>
      <pre>{{ JSON.stringify(journals, null, 2) }}</pre>
    </code>

    <div>
      <label for="journal-id">Journal Id</label>
      <input id="journal-id" v-model="id" />
    </div>

    <code>
      <pre>{{ JSON.stringify(journal, null, 2) }}</pre>
    </code>
  </q-page>
</template>

<script setup lang="ts">
import { ref, inject, onMounted, watch } from "vue";
import { Todo, Meta } from "components/models";
import ExampleComponent from "components/ExampleComponent.vue";
import { useConfigStore } from "stores/config-store";

defineOptions({
  name: "IndexPage",
});

const configStore = useConfigStore();

const todos = ref<Todo[]>([
  {
    id: 1,
    content: "ct1",
  },
  {
    id: 2,
    content: "ct2",
  },
  {
    id: 3,
    content: "ct3",
  },
  {
    id: 4,
    content: "ct4",
  },
  {
    id: 5,
    content: "ct5",
  },
]);

const meta = ref<Meta>({
  totalCount: 1200,
});

const journalFindById = inject<(id: string) => Promise<object | undefined>>("journalFindById");
const journalFindAll =
  inject<(query: object) => Promise<[object[], Map<string, object>]>>("journalFindAll");

const journals = ref<object[]>([]);
const included = ref<Map<string, object>>(new Map());

const journal = ref<object>();
const id = ref("");
watch(id, async (newId) => {
  if (newId) {
    journal.value = await journalFindById?.(newId);
  }
});

onMounted(async () => {
  const models = await journalFindAll?.({});
  if (models) {
    journals.value = models[0];
    included.value = models[1];
  }
});
</script>
