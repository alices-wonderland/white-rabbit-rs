export { default as AppTable } from "./AppTable.vue";
export { default as AppTableEditableCellRenderer } from "./AppTableEditableCellRenderer.vue";

export type EditableValue<V> =
  | { readonly state: "NEW" | "NORMAL"; readonly value: V }
  | { readonly state: "UPDATED"; readonly value: V; readonly existing: V };
