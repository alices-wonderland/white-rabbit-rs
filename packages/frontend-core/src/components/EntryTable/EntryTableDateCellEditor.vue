<script lang="ts">
import type { ICellEditorParams } from "@ag-grid-community/core";
import type { Row } from "./row";
import { ref } from "vue";
import { format } from "date-fns";

type Props = { readonly params: ICellEditorParams<Row, string> };

export default {
  setup(props: Props) {
    const value = ref<string>(props.params.value ?? format(new Date(), "yyyy-MM-dd"));

    const getValue = () => value.value;

    return {
      value,
      getValue,
    };
  },
};
</script>

<template>
  <q-input v-model="value" filled>
    <template #append>
      <q-icon name="event" class="cursor-pointer">
        <q-popup-proxy cover transition-show="scale" transition-hide="scale">
          <q-date v-model="value" mask="YYYY-MM-DD">
            <div class="row items-center justify-end">
              <q-btn v-close-popup label="Close" color="primary" flat />
            </div>
          </q-date>
        </q-popup-proxy>
      </q-icon>
    </template>
  </q-input>
</template>
