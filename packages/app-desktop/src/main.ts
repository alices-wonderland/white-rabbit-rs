import "@desktop/style.scss";

import { createApp } from "vue";
import App from "./App.vue";
import { agGrid } from "@core/plugins";

const app = createApp(App);

app.use(agGrid);

app.mount("#app");
