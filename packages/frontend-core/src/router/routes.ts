import { AppLayout } from "@core/layouts";
import { JournalsPage, JournalPage } from "@core/pages";
import type { RouterOptions } from "vue-router";
import JournalBreadcrumb from "@core/components/JournalBreadcrumb.vue";

export const ROUTE_JOURNALS = Symbol("ROUTE_JOURNALS");
export const ROUTE_JOURNAL = Symbol("ROUTE_JOURNAL");

export default [
  {
    path: "/journals",
    component: AppLayout,
    children: [
      {
        path: "",
        name: ROUTE_JOURNALS,
        components: {
          default: JournalsPage,
          toolbar: JournalBreadcrumb,
        },
      },
      {
        path: ":id",
        name: ROUTE_JOURNAL,
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
