export { default as AppTable } from "./AppTable.vue";
export { default as AppTableEditableCellRenderer } from "./AppTableEditableCellRenderer.vue";

export type CellState<V = unknown> =
  | { readonly state: "NORMAL"; readonly value: V }
  | { readonly state: "UPDATED"; readonly value: V; readonly existing: V };

export type RowState<F extends string = string> =
  | { readonly state: "NEW" | "DELETED" | "NORMAL" }
  | { readonly state: "UPDATED"; readonly updatedFields: F[] };

export abstract class AbstractRow<M, F extends string = string> {
  readonly id: string;
  protected readonly _existing?: M;

  protected _deleted: boolean = false;

  protected constructor(id: string, existing?: M) {
    this.id = id;
    this._existing = existing;
  }

  abstract get updatableFields(): readonly F[];

  abstract reset(): void;

  abstract getCellState<V>(field: F): CellState<V>;

  get rowState(): RowState {
    if (!this._existing) {
      return {
        state: "NEW",
      };
    } else if (this._deleted) {
      return {
        state: "DELETED",
      };
    }

    const updatedFields = this.updatableFields
      .map<[F, CellState]>((field) => [field, this.getCellState(field)])
      .filter(([_field, state]) => state.state === "UPDATED")
      .map(([field]): F => field);

    if (updatedFields.length > 0) {
      return {
        state: "UPDATED",
        updatedFields: updatedFields,
      };
    } else {
      return {
        state: "NORMAL",
      };
    }
  }

  get deleted() {
    return this._deleted;
  }

  set deleted(value: boolean) {
    this._deleted = value;
    if (this._deleted) {
      this.reset();
    }
  }
}
