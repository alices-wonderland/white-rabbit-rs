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
          "sub-toolbar": JournalBreadcrumb,
        },
      },
      {
        path: ":id",
        components: {
          default: JournalPage,
          "sub-toolbar": JournalBreadcrumb,
        },
      },
    ],
  },

  {
    path: "/:pathMatch(.*)*",
    redirect: "/journals",
  },
] as RouterOptions["routes"];
