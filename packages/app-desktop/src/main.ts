import "@core/style.scss";
import { createApp } from "vue";
import App from "./App.vue";
import { agGrid, vuetify, i18n } from "@core/plugins";
import { ACCOUNT_API_KEY, JOURNAL_API_KEY, ENTRY_API_KEY } from "@core/services";
import { accountApi, journalApi, entryApi } from "@/services";

const app = createApp(App);

app.use(agGrid);
app.use(vuetify);
app.use(i18n);
app.provide(JOURNAL_API_KEY, journalApi);
app.provide(ACCOUNT_API_KEY, accountApi);
app.provide(ENTRY_API_KEY, entryApi);
app.mount("#app");
