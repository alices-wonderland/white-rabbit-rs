<script setup lang="ts">
import { AgChartsVue } from "ag-charts-vue3";
import type { AgChartOptions } from "ag-charts-community";
import { computed } from "vue";
import { useTheme } from "vuetify";

const theme = useTheme();

const chartTheme = computed(() => {
  return theme.global.name.value === "dark" ? "ag-default-dark" : "ag-default";
});

const options = computed<AgChartOptions>(() => ({
  theme: chartTheme.value,
  data: [
    {
      beverage: "Coffee",
      Q1: 700,
      Q2: 600,
      Q3: 560,
      Q4: 450,
    },
    {
      beverage: "Tea",
      Q1: 520,
      Q2: 450,
      Q3: 380,
      Q4: 270,
    },
    {
      beverage: "Milk",
      Q1: 200,
      Q2: 190,
      Q3: 170,
      Q4: 180,
    },
  ],
  title: {
    text: "Beverage Expenses",
  },
  subtitle: {
    text: "per quarter",
  },
  footnote: {
    text: "Based on a sample size of 200 respondents",
  },
  padding: {
    top: 40,
    right: 40,
    bottom: 40,
    left: 40,
  },
  series: [
    { type: "column", xKey: "beverage", yKey: "Q1", stacked: true },
    { type: "column", xKey: "beverage", yKey: "Q2", stacked: true },
    { type: "column", xKey: "beverage", yKey: "Q3", stacked: true },
    { type: "column", xKey: "beverage", yKey: "Q4", stacked: true },
  ],
  legend: { spacing: 40 },
}));
</script>

<template>
  <div class="wrapper">
    <ag-charts-vue :options="options"> </ag-charts-vue>
  </div>
</template>

<style scoped lang="scss">
.wrapper {
  min-height: 30vh;
  max-height: 70vh;
  height: 60vh;
  overflow: hidden;
  resize: vertical;
  padding: 6px;
}
</style>
