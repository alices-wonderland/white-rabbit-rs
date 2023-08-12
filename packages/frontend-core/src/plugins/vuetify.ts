import { createVuetify } from "vuetify";
import { aliases, mdi } from "vuetify/iconsets/mdi-svg";
import { useI18n } from "vue-i18n";
import { createVueI18nAdapter } from "vuetify/locale/adapters/vue-i18n";
import i18n from "./i18n";
import DateFnsAdapter from "@date-io/date-fns";
import { enUS, zhCN } from "date-fns/locale";

const vuetify = createVuetify({
  icons: {
    defaultSet: "mdi",
    aliases,
    sets: {
      mdi,
    },
  },
  date: {
    adapter: DateFnsAdapter,
    locale: {
      en: enUS,
      "zh-Hans": zhCN,
    },
  },
  locale: {
    adapter: createVueI18nAdapter({ i18n, useI18n }),
  },
  theme: {
    themes: {
      light: {
        dark: false,
        colors: {
          background: "#FFFBFE",
          surface: "#FFFBFE",
          primary: "#6750A4",
          secondary: "#625B71",
          tertiary: "#7e5260",
          error: "#B3261E",
        },
      },
      dark: {
        dark: true,
        colors: {
          background: "#1c1b1e",
          surface: "#1c1b1e",
          primary: "#cfbcff",
          secondary: "#cbc2db",
          tertiary: "#efb8c8",
          error: "#ffb4ab",
        },
      },
    },
  },
});

export default vuetify;
