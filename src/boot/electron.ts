import { boot } from "quasar/wrappers";

declare global {
  interface Window {
    electron: {
      journalFindById: (id: string) => object | undefined;
      journalFindAll: (query: object) => [object[], Map<string, object>];
      accountFindById: (id: string) => object | undefined;
      accountFindAll: (query: object) => [object[], Map<string, object>];
    };
  }
}

// "async" is optional;
// more info on params: https://v2.quasar.dev/quasar-cli/boot-files
export default boot(async ({ app }) => {
  console.log("[Boot Electron] Initializing...");

  const journalFindById = (id: string) => window.electron.journalFindById(id);
  const journalFindAll = (query: object) => {
    console.log("Journal Find All render side: ", query);
    return window.electron.journalFindAll(query);
  };

  const accountFindById = (id: string) => window.electron.accountFindById(id);
  const accountFindAll = (query: object) => window.electron.accountFindAll(query);

  app.provide("journalFindById", journalFindById);
  app.provide("journalFindAll", journalFindAll);
  app.provide("accountFindById", accountFindById);
  app.provide("accountFindAll", accountFindAll);
});
