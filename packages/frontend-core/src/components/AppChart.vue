<script setup lang="ts">
import { AgCharts } from "ag-charts-community";
import type { AgChartInstance, AgChartOptions } from "ag-charts-community";
import { useQuasar } from "quasar";
import { computed, onMounted, onUnmounted, ref, watch } from "vue";

const quasar = useQuasar();
const chartRef = ref<HTMLElement>();
const chartInst = ref<AgChartInstance>();
const created = ref(false);

const props = defineProps<{
  readonly options: AgChartOptions;
}>();
const emits = defineEmits<{
  chartReady: [inst: AgChartInstance];
}>();

const options = computed<AgChartOptions>(() => ({
  ...props.options,
  container: chartRef.value,
  theme: quasar.dark.isActive ? "ag-default-dark" : "ag-default",
}));

watch(
  options,
  (newOptions) => {
    if (chartInst.value && created.value) {
      AgCharts.update(chartInst.value, newOptions);
    }
  },
  {
    deep: true,
  },
);

onMounted(async () => {
  chartInst.value = AgCharts.create(options.value);
  created.value = true;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  await (chartInst.value as any).chart.waitForUpdate();
  emits("chartReady", chartInst.value);
});

onUnmounted(() => {
  chartInst.value?.destroy();
});
</script>

<template>
  <div ref="chartRef" class="app-chart"></div>
</template>

<style scoped lang="scss">
.app-chart {
  min-height: 30vh;
  max-height: 70vh;
  height: 60vh;
  overflow: hidden;
  resize: vertical;
  padding: 6px;
}
</style>
