import type {
  AccountQuery,
  JournalQuery,
  EntryApi,
  EntryCommand,
  EntryItem,
  EntryQuery,
  EntrySort,
  EntryState,
  EntryType,
  Model,
} from "@core/services";
import { Entry } from "@core/services";
import { AbstractWriteApi } from "./api";
import { journalApi } from "./journal";
import { toMap } from "@core/utils";
import { accountApi } from "./account";
import { parseISO } from "date-fns";

class EntryApiImpl extends AbstractWriteApi<Entry, EntryQuery, EntryCommand, EntrySort> {
  protected override get findAllKey(): string {
    return "entry_find_all";
  }

  protected override get findByIdKey(): string {
    return "entry_find_by_id";
  }

  protected override get handleCommandKey(): string {
    return "entry_handle_command";
  }

  protected override async loadIncluded(models: Entry[]): Promise<Map<string, Model>> {
    const journalIds = new Set(models.map((model) => model.journalId));
    const journals = await journalApi.findAll({ query: { id: [...journalIds] } as JournalQuery });

    const accountIds = new Set(models.flatMap((model) => model.items).map((item) => item.account));
    const accounts = await accountApi.findAll({ query: { id: [...accountIds] } as AccountQuery });

    return toMap([...journals[0], ...accounts[0]]);
  }

  protected override convert(input: Record<string, unknown>): Entry {
    const items: EntryItem[] = (
      input.items as Array<{ account: string; price?: string; amount: string }>
    ).map(({ account, amount, price }) => ({
      account: account,
      amount: parseFloat(amount),
      price: price ? parseFloat(price) : undefined,
    }));
    return new Entry({
      id: input.id as string,
      journalId: input.journalId as string,
      name: input.name as string,
      description: input.description as string,
      type: input.type as EntryType,
      date: parseISO(input.date as string),
      tags: input.tags as string[],
      items: items,
      state: input.state as EntryState,
    });
  }
}

export const entryApi: EntryApi = new EntryApiImpl();
