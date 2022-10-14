import { Account, Journal } from "./models";

export abstract class SharedError {
  abstract get type(): string;

  protected constructor(readonly message: string) {}
}

export class AccountNotInJournalError extends SharedError {
  readonly journalId?: string;
  readonly accountId?: string;

  constructor(journal?: Journal, account?: Account) {
    super(
      `Account[${account?.id}, name=${account?.name}] is not in Journal[${journal?.id}, name=${journal?.name}]`
    );
  }

  override get type(): string {
    return "AccountNotInJournalError";
  }
}
