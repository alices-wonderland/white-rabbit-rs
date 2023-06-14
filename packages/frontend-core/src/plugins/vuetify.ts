import { createVuetify } from "vuetify";
import { aliases, md } from "vuetify/iconsets/md";

const vuetify = createVuetify({
  icons: {
    defaultSet: "md",
    aliases,
    sets: {
      md,
    },
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
