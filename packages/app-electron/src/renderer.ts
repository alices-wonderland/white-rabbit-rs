import "@core/style.scss";
import { createApp } from "vue";
import App from "./App.vue";
import { agGrid, i18n, quasar } from "@core/plugins";
import {
  ACCOUNT_API_KEY,
  ENTRY_API_KEY,
  HIERARCHY_REPORT_API_KEY,
  JOURNAL_API_KEY,
} from "@core/services";
import { accountApi, entryApi, hierarchyReportApi, journalApi } from "@/services";
import router from "@core/router";
import { VueQueryPlugin } from "@tanstack/vue-query";

const intervalId = setInterval(async () => {
  const port = await window.api.getPort();
  console.log("Port Value: ", port);
  if (port && intervalId) {
    const resp = await fetch(`http://localhost:${port}/actuator/health`);
    const body = await resp.json();
    if (resp.ok && typeof body === "object" && body.status === "UP") {
      await fetch(`http://localhost:${port}/init`, {
        method: "POST",
      });

      const app = createApp(App);

      app.use(agGrid);
      app.use(i18n);
      app.use(quasar);
      app.use(router);
      app.use(VueQueryPlugin);

      app.provide(JOURNAL_API_KEY, journalApi);
      app.provide(ACCOUNT_API_KEY, accountApi);
      app.provide(ENTRY_API_KEY, entryApi);
      app.provide(HIERARCHY_REPORT_API_KEY, hierarchyReportApi);
      app.mount("#app");

      clearInterval(intervalId);
    }
  }
}, 500);
