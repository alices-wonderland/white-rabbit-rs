import type { Ref } from "vue";
import { useQuasar } from "quasar";
import { useDark as useVueDark, useToggle } from "@vueuse/core";

export function useDark(): [Ref<boolean>, (value?: boolean) => boolean] {
  const quasar = useQuasar();

  const isDarkRef = useVueDark({
    onChanged: (isDark) => {
      quasar.dark.set(isDark);
    },
  });

  const toggleDark = useToggle(isDarkRef);

  return [isDarkRef, toggleDark];
}
