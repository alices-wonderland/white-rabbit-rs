import { RouteRecordRaw } from "vue-router";
import AppLayout from "layouts/AppLayout.vue";
import JournalBreadcrumb from "components/JournalBreadcrumb.vue";

export const ROUTE_JOURNALS = Symbol("ROUTE_JOURNALS");
export const ROUTE_JOURNAL = Symbol("ROUTE_JOURNAL");

const routes: RouteRecordRaw[] = [
  {
    path: "/",
    component: AppLayout,
    redirect: "/journals",
    children: [
      {
        path: "/journals",
        name: ROUTE_JOURNALS,
        components: {
          default: () => import("pages/JournalsPage/JournalsPage.vue"),
          toolbar: JournalBreadcrumb,
        },
      },
      {
        path: ":id",
        name: ROUTE_JOURNAL,
        components: {
          default: () => import("pages/JournalPage.vue"),
          toolbar: JournalBreadcrumb,
        },
      },
    ],
  },

  {
    path: "/:pathMatch(.*)*",
    redirect: "/journals",
  },
];

export default routes;
