import { createApp } from "vue";
import App from "./App.vue";
import { agGridPlugin, vuetifyPlugin } from "@shared/plugins";
import VCalendar from "v-calendar";

import "./style.scss";
import "v-calendar/dist/style.css";

const app = createApp(App);

app.use(agGridPlugin);
app.use(vuetifyPlugin);
app.use(VCalendar, {});

app.mount("#app");
