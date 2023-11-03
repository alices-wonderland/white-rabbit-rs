import "@quasar/extras/roboto-font/roboto-font.css";
import "@quasar/extras/material-icons/material-icons.css";
import iconSet from "quasar/icon-set/material-icons";

import "quasar/src/css/index.sass";

import type { App } from "vue";
import { Quasar, Notify } from "quasar";
import langEnUS from "quasar/lang/en-US";

export default function (app: App) {
  app.use(Quasar, {
    plugins: {
      Notify,
    },
    lang: langEnUS,
    iconSet: iconSet,
    config: {
      brand: {
        primary: "#1976D2",
        secondary: "#26A69A",
        accent: "#9C27B0",
        dark: "#1D1D1D",
        positive: "#21BA45",
        negative: "#C10015",
        info: "#31CCEC",
        warning: "#F2C037",
      },
    },
  });
}
