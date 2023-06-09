import "@desktop/style.scss";

import { createApp } from "vue";
import App from "./App.vue";
import { agGrid } from "@core/plugins";
import { USER_API_KEY } from "@core/services";
import { userApi } from "@desktop/services";

const app = createApp(App);

app.use(agGrid);
app.provide(USER_API_KEY, userApi);
app.mount("#app");
