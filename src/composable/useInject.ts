import { inject } from "vue";
import { ReadApi } from "src/services/api";

export default function useInject<A extends ReadApi>(key: symbol): A {
  const api = inject<A>(key);
  if (!api) {
    throw new Error(`Api with ${String(key)} cannot be found`);
  }
  return api;
}
