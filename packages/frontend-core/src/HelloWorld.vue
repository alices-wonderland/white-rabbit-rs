<script setup lang="ts">
const props = defineProps<{ value: object }>();

const externalMethod = (num: number) => num * 2;

defineExpose({ externalMethod });
</script>

<template>
  <div class="hello-tag">
    <div>Hello, the result is:</div>
    <code>
      <pre>{{ JSON.stringify(props.value, null, 2) }}</pre>
    </code>
  </div>
</template>

<style scoped lang="scss">
.hello-tag {
  background-color: aliceblue;
}
</style>
