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
  EntryStateItem,
} from "@core/services";
import { Entry } from "@core/services";
import { AbstractWriteApi } from "./api";
import { journalApi } from "./journal";
import { toMap } from "@core/utils";
import { accountApi } from "./account";
import { validate as uuidValidate } from "uuid";

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

  private parseStateItem(input: Record<string, unknown>): EntryStateItem | undefined {
    if ("type" in input && "value" in input) {
      if (input.type === "Valid" && typeof input.value === "string") {
        return {
          type: "Valid",
          value: parseFloat(input.value),
        };
      } else if (
        input.type === "Invalid" &&
        Array.isArray(input.value) &&
        input.value.length === 2
      ) {
        return {
          type: "Invalid",
          value: [parseFloat(input.value[0]), parseFloat(input.value[1])],
        };
      }
    }

    return undefined;
  }

  protected override convert(input: Record<string, unknown>): Entry {
    const items: EntryItem[] = (
      input.items as Array<{ account: string; price?: string; amount: string }>
    ).map(({ account, amount, price }) => ({
      account: account,
      amount: parseFloat(amount),
      price: price ? parseFloat(price) : undefined,
    }));

    let state: EntryState;
    if (input.type === "Record") {
      state = this.parseStateItem(input.state as Record<string, unknown>) ?? {
        type: "Valid",
        value: 0,
      };
    } else {
      state = Object.fromEntries(
        Object.entries(input.state as Record<string, unknown>)
          .filter(
            (entry): entry is [string, Record<string, unknown>] =>
              uuidValidate(entry[0]) && !!entry[1],
          )
          .map(([id, item]) => [id, this.parseStateItem(item)])
          .filter((entry) => entry[1]),
      );
    }

    return new Entry({
      id: input.id as string,
      journalId: input.journalId as string,
      name: input.name as string,
      description: input.description as string,
      type: input.type as EntryType,
      date: input.date as string,
      tags: input.tags as string[],
      items: items,
      state: state,
    });
  }
}

export const entryApi: EntryApi = new EntryApiImpl();
