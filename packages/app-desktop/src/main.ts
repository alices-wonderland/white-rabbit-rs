import { createApp } from "vue";
import App from "./App.vue";
import { agGridPlugin, vuetifyPlugin } from "@white-rabbit/frontend-shared";

const app = createApp(App);

app.use(agGridPlugin);
app.use(vuetifyPlugin);

app.mount("#app");
