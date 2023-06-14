import type { WriteApi } from "@core/services";
import { inject } from "vue";

export default function usePage<A extends WriteApi>(key: symbol): A {
  const api = inject<A>(key);
  if (!api) {
    throw new Error(`Api with ${String(key)} cannot be found`);
  }
  return api;
}
