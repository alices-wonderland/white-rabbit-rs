import {
  createVuetify,
  type ThemeDefinition,
  type VuetifyOptions,
} from "vuetify";
import { aliases, mdi } from "vuetify/iconsets/mdi-svg";
// Waiting for https://github.com/vuetifyjs/vuetify/issues/14875
// eslint-disable-next-line @typescript-eslint/ban-ts-comment
// @ts-ignore
// import { createVueI18nAdapter } from "vuetify/locale/adapters/vue-i18n";

const light: ThemeDefinition = {
  dark: false,
  colors: {
    background: "#fffbff",
    surface: "#fffbff",
    outline: "#827568",
    surfaceVariant: "#f0e0d0",

    primary: "#855400",
    secondary: "#8b13de",
    tertiary: "#3e6919",
    error: "#ba1a1a",
    info: "#5936f4",
  },
};

const dark: ThemeDefinition = {
  dark: true,
  colors: {
    background: "#1f1b16",
    surface: "#1f1b16",
    outline: "#9c8e80",
    surfaceVariant: "#504539",

    primary: "#ffb95d",
    secondary: "#e0b7ff",
    tertiary: "#a3d578",
    error: "#ffb4ab",
    info: "#c7bfff",
  },
};

export const vuetifyOptions: VuetifyOptions = {
  icons: {
    defaultSet: "mdi",
    aliases,
    sets: {
      mdi,
    },
  },
  theme: {
    defaultTheme: "light",
    themes: {
      light,
      dark,
    },
  },
};

export default createVuetify(vuetifyOptions);
