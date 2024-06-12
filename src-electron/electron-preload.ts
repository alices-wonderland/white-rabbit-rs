/**
 * This file is used specifically for security reasons.
 * Here you can access Nodejs stuff and inject functionality into
 * the renderer thread (accessible there through the "window" object)
 *
 * WARNING!
 * If you import anything from node_modules, then make sure that the package is specified
 * in package.json > dependencies and NOT in devDependencies
 *
 * Example (injects window.myAPI.doAThing() into renderer thread):
 *
 *   import { contextBridge } from 'electron'
 *
 *   contextBridge.exposeInMainWorld('myAPI', {
 *     doAThing: () => {}
 *   })
 *
 * WARNING!
 * If accessing Node functionality (like importing @electron/remote) then in your
 * electron-main.ts you will need to set the following when you instantiate BrowserWindow:
 *
 * mainWindow = new BrowserWindow({
 *   // ...
 *   webPreferences: {
 *     // ...
 *     sandbox: false // <-- to be able to import @electron/remote in preload script
 *   }
 * }
 */

import { contextBridge, ipcRenderer } from "electron";
import { AccountApi } from "src/services/account";
import { EntryApi } from "src/services/entry";
import { HierarchyReportApi } from "src/services/hierarchy-report";
import { JournalApi } from "src/services/journal";

contextBridge.exposeInMainWorld("electron", {
  journalApi: {
    findAll(args, loadIncluded) {
      return ipcRenderer.invoke("journalApi.findAll", args, loadIncluded);
    },

    findById(id, loadIncluded) {
      return ipcRenderer.invoke("journalApi.findById", id, loadIncluded);
    },

    handleCommand(command) {
      return ipcRenderer.invoke("journalApi.handleCommand", command);
    },
  } satisfies JournalApi,

  accountApi: {
    findAll(args, loadIncluded) {
      return ipcRenderer.invoke("accountApi.findAll", args, loadIncluded);
    },

    findById(id, loadIncluded) {
      return ipcRenderer.invoke("accountApi.findById", id, loadIncluded);
    },

    handleCommand(command) {
      return ipcRenderer.invoke("accountApi.handleCommand", command);
    },
  } satisfies AccountApi,

  entryApi: {
    findAll(args, loadIncluded) {
      return ipcRenderer.invoke("entryApi.findAll", args, loadIncluded);
    },

    findById(id, loadIncluded) {
      return ipcRenderer.invoke("entryApi.findById", id, loadIncluded);
    },

    handleCommand(command) {
      return ipcRenderer.invoke("entryApi.handleCommand", command);
    },
  } satisfies EntryApi,

  hierarchyReportApi: {
    findAll(args, loadIncluded) {
      return ipcRenderer.invoke("hierarchyReportApi.findAll", args, loadIncluded);
    },

    findById(id, loadIncluded) {
      return ipcRenderer.invoke("hierarchyReportApi.findById", id, loadIncluded);
    },
  } satisfies HierarchyReportApi,
});
