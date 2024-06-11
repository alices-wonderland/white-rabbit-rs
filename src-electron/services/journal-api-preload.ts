import pick from "lodash/pick";
import { ipcRenderer } from "electron";

import { Model, FindAllArgs } from "src/services/api";
import {
  JOURNAL_FIELDS,
  Journal,
  JournalApi,
  JournalCommand,
  JournalField,
  JournalQuery,
  JournalSort,
} from "src/services/journal";

class JournalApiPreloadImpl implements JournalApi {
  async handleCommand(command: JournalCommand): Promise<Journal[]> {
    const results = await ipcRenderer.invoke("journalApi.handleCommand", { command });
    return JournalApiPreloadImpl.fromAll(results);
  }

  async findById(
    id: string,
    loadIncluded?: boolean,
  ): Promise<[Journal, Map<string, Model<string>>] | undefined> {
    const resp = await ipcRenderer.invoke("journalApi.findById", { id, loadIncluded });
    if (resp.value) {
      const parsed = JournalApiPreloadImpl.from(resp.value);
      if (parsed) {
        return [parsed, new Map()];
      }
    }

    return undefined;
  }

  async findAll(
    args: FindAllArgs<JournalQuery, JournalSort>,
    loadIncluded?: boolean,
  ): Promise<[Journal[], Map<string, Model<string>>]> {
    const resp = await ipcRenderer.invoke("journalApi.findAll", { args, loadIncluded });
    if (Array.isArray(resp.values)) {
      return [JournalApiPreloadImpl.fromAll(resp.values), new Map()];
    }

    return [[], new Map()];
  }

  static from(item: object): Journal | undefined {
    for (const key of JOURNAL_FIELDS) {
      if (!(key in item)) {
        return undefined;
      }
    }

    return new Journal(pick(item, JOURNAL_FIELDS) as Pick<Journal, JournalField>);
  }

  static fromAll(items: object[]): Journal[] {
    return items
      .map((item) => JournalApiPreloadImpl.from(item))
      .filter((item): item is Journal => !!item);
  }
}

export const journalApiPreload = new JournalApiPreloadImpl();
