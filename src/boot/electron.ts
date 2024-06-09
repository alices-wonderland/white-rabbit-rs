import { boot } from "quasar/wrappers";

import { ACCOUNT_API_KEY, type AccountApi } from "src/services/account";
import { ENTRY_API_KEY, type EntryApi } from "src/services/entry";
import { HIERARCHY_REPORT_API_KEY, type HierarchyReportApi } from "src/services/hierarchy-report";
import { JOURNAL_API_KEY, type JournalApi } from "src/services/journal";

declare global {
  interface Window {
    electron: {
      journalApi: JournalApi;
      accountApi: AccountApi;
      entryApi: EntryApi;
      hierarchyReportApi: HierarchyReportApi;
    };
  }
}

// "async" is optional;
// more info on params: https://v2.quasar.dev/quasar-cli/boot-files
export default boot(async ({ app }) => {
  console.log("[Boot Electron] Initializing...");

  app.provide<JournalApi>(JOURNAL_API_KEY, {
    async findAll(args, loadIncluded) {
      return window.electron.journalApi.findAll(args, loadIncluded);
    },

    async findById(id, loadIncluded) {
      return window.electron.journalApi.findById(id, loadIncluded);
    },

    async handleCommand(command) {
      return window.electron.journalApi.handleCommand(command);
    },
  });

  app.provide<AccountApi>(ACCOUNT_API_KEY, {
    async findAll(args, loadIncluded) {
      return window.electron.accountApi.findAll(args, loadIncluded);
    },

    async findById(id, loadIncluded) {
      return window.electron.accountApi.findById(id, loadIncluded);
    },

    async handleCommand(command) {
      return window.electron.accountApi.handleCommand(command);
    },
  });

  app.provide<EntryApi>(ENTRY_API_KEY, {
    async findAll(args, loadIncluded) {
      return window.electron.entryApi.findAll(args, loadIncluded);
    },

    async findById(id, loadIncluded) {
      return window.electron.entryApi.findById(id, loadIncluded);
    },

    async handleCommand(command) {
      return window.electron.entryApi.handleCommand(command);
    },
  });

  app.provide<HierarchyReportApi>(HIERARCHY_REPORT_API_KEY, {
    async findAll(args, loadIncluded) {
      return window.electron.hierarchyReportApi.findAll(args, loadIncluded);
    },

    async findById(id, loadIncluded) {
      return window.electron.hierarchyReportApi.findById(id, loadIncluded);
    },
  });
});
