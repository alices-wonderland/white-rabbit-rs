import "@core/style.scss";
import { createApp } from "vue";
import App from "./App.vue";
import { agGrid, vuetify, i18n } from "@core/plugins";
import { ACCOUNT_API_KEY, JOURNAL_API_KEY, RECORD_API_KEY, USER_API_KEY } from "@core/services";
import { accountApi, journalApi, recordApi, userApi } from "@desktop/services";

const app = createApp(App);

app.use(agGrid);
app.use(vuetify);
app.use(i18n);
app.provide(USER_API_KEY, userApi);
app.provide(JOURNAL_API_KEY, journalApi);
app.provide(ACCOUNT_API_KEY, accountApi);
app.provide(RECORD_API_KEY, recordApi);
app.mount("#app");
