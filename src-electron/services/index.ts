import { Model } from "src/services/api";
import { ACCOUNT_TYPE_URL, fromProto as fromProtoAccount } from "./account";
import { JOURNAL_TYPE_URL, fromProto as fromProtoJournal } from "./journal";
import { Any } from "../proto/gen/google/protobuf/any";
import { Journal as JournalProto } from "../proto/gen/whiterabbit/journal/v1/journal";
import { Account as AccountProto } from "../proto/gen/whiterabbit/account/v1/account";

export function fromAny(model: Any): Model | undefined {
  switch (model.typeUrl) {
    case JOURNAL_TYPE_URL:
      return fromProtoJournal(JournalProto.decode(model.value));
    case ACCOUNT_TYPE_URL:
      return fromProtoAccount(AccountProto.decode(model.value));
    default:
      return undefined;
  }
}

export function fromIncluded(included: Record<string, Any>): Map<string, Model> {
  return new Map(
    Object.entries(included)
      .map(([id, proto]) => {
        const parsed = fromAny(proto);
        return parsed ? [id, parsed] : undefined;
      })
      .filter((item): item is [string, Model] => !!item),
  );
}
