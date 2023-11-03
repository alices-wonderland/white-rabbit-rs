import { AppLayout } from "@core/layouts";
import { JournalsPage, JournalPage } from "@core/pages";
import type { RouterOptions } from "vue-router";
import JournalBreadcrumb from "@core/components/JournalBreadcrumb.vue";

export default [
  {
    path: "/journals",
    component: AppLayout,
    children: [
      {
        path: "",
        components: {
          default: JournalsPage,
          toolbar: JournalBreadcrumb,
        },
      },
      {
        path: ":id",
        components: {
          default: JournalPage,
          toolbar: JournalBreadcrumb,
        },
      },
    ],
  },

  {
    path: "/:pathMatch(.*)*",
    redirect: "/journals",
  },
] as RouterOptions["routes"];
