import { createApp } from "vue";
import App from "./App.vue";
import { agGridPlugin, vuetifyPlugin } from "@shared/plugins";
import VCalendar from "v-calendar";

import "./style.scss";
import "v-calendar/dist/style.css";
import { CacheApiService } from "@shared/services";
import {
  AccountApiService,
  AUTH_SERVICE,
  GroupApiService,
  JournalApiService,
  RecordApiService,
  UserApiService,
} from "./services";
import {
  KEY_USER_API,
  KEY_GROUP_API,
  KEY_JOURNAL_API,
  KEY_ACCOUNT_API,
  KEY_RECORD_API,
} from "@shared/hooks";

const app = createApp(App);

app.use(agGridPlugin);
app.use(vuetifyPlugin);
app.use(VCalendar, {});

app.provide(
  KEY_USER_API,
  new CacheApiService(KEY_USER_API, new UserApiService(AUTH_SERVICE))
);
app.provide(
  KEY_GROUP_API,
  new CacheApiService(KEY_GROUP_API, new GroupApiService(AUTH_SERVICE))
);
app.provide(
  KEY_JOURNAL_API,
  new CacheApiService(KEY_JOURNAL_API, new JournalApiService(AUTH_SERVICE))
);
app.provide(
  KEY_ACCOUNT_API,
  new CacheApiService(KEY_ACCOUNT_API, new AccountApiService(AUTH_SERVICE))
);
app.provide(
  KEY_RECORD_API,
  new CacheApiService(KEY_RECORD_API, new RecordApiService(AUTH_SERVICE))
);

app.mount("#app");
