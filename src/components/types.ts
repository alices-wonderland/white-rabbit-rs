export type FieldState<V = unknown> =
  | { readonly state: "NORMAL"; readonly value: V }
  | { readonly state: "UPDATED"; readonly value: V; readonly existing: V };

export const NULL_PLACEHOLDER = "(null)";
