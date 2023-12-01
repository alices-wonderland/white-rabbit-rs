import { createRouter, createWebHistory } from "vue-router";
import routes from "./routes";

const router = createRouter({
  history: createWebHistory(),
  routes: routes,
});

export default router;
export { ROUTE_JOURNALS, ROUTE_JOURNAL } from "./routes";
