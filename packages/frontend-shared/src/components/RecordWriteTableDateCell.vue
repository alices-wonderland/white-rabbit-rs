<template>
  <v-menu>
    <template #activator="{ props: menuProps }">
      <v-text-field
        v-bind="menuProps"
        v-model="formatted"
        hide-details
      ></v-text-field>
    </template>
    <v-date-picker v-model="date" mode="date"> </v-date-picker>
  </v-menu>
</template>

<script lang="ts">
import { ICellRendererParams } from "@ag-grid-community/core";
import { computed, defineComponent, PropType, ref, toRefs } from "vue";
import format from "date-fns/format";
import isMatch from "date-fns/isMatch";

const FORMAT = "yyyy-MM-dd";

export default defineComponent({
  props: {
    params: {
      type: Object as PropType<ICellRendererParams>,
      required: true,
    },
  },
  setup(props) {
    const { params } = toRefs(props);
    const formatted = ref(params.value.data.date);
    const date = computed({
      get(): Date {
        return new Date(formatted.value);
      },
      set(val: Date) {
        formatted.value = format(val, FORMAT);
      },
    });
    return {
      formatted,
      date,
    };
  },
  // https://vuejs.org/api/sfc-script-setup.html#defineexpose
  // Composition API will remove all unused functions,
  //so for Ag Grid Cell, we must use the Options API
  methods: {
    getValue(): string | undefined {
      if (isMatch(this.formatted, FORMAT)) {
        return this.formatted;
      } else {
        return undefined;
      }
    },
  },
});
</script>
