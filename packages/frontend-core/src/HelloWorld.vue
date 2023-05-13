<script setup lang="ts">
const props = defineProps<{ value: number }>();

const externalMethod = (num: number) => num * 2;

defineExpose({ externalMethod });
</script>

<template>
  <div class="hello-tag">Hello, the result is {{ props.value }}</div>
</template>

<style scoped lang="scss">
.hello-tag {
  background-color: aliceblue;
}
</style>
