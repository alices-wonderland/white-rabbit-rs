import type { FieldState } from "@core/types";
import isEmpty from "lodash/isEmpty";

export { default as AppTable } from "./AppTable.vue";
export { default as AppTableEditableCellRenderer } from "./AppTableEditableCellRenderer.vue";
export { default as AppTableTagsCellRenderer } from "./AppTableTagsCellRenderer.vue";
export { default as AppTableTagsCellEditor } from "./AppTableTagsCellEditor.vue";
export { default as AppTableAccountCellRenderer } from "./AppTableAccountCellRenderer.vue";
export { default as AppTableAccountCellEditor } from "./AppTableAccountCellEditor.vue";

export type RowState<F extends string = string> =
  | { readonly state: "NEW" | "DELETED" | "NORMAL" }
  | { readonly state: "UPDATED"; readonly editedFields: F[] };

export abstract class AbstractRow<M, F extends string = string> {
  protected readonly _existing?: M;
  protected readonly _readonly?: boolean;

  protected _deleted: boolean = false;

  protected constructor(existing?: M, readonly?: boolean) {
    this._existing = existing;
    this._readonly = readonly;
  }

  abstract get id(): string;

  abstract get editableFields(): readonly F[];

  abstract reset(): void;

  abstract getFieldState<V>(field: F): FieldState<V>;

  get rowState(): RowState {
    if (!this._existing) {
      return {
        state: "NEW",
      };
    } else if (this.deleted) {
      return {
        state: "DELETED",
      };
    }

    const editedFields = this.editableFields
      .map<[F, FieldState]>((field) => [field, this.getFieldState(field)])
      .filter(([_field, state]) => state.state === "UPDATED")
      .map(([field]): F => field);

    if (editedFields.length > 0) {
      return {
        state: "UPDATED",
        editedFields: editedFields,
      };
    } else {
      return {
        state: "NORMAL",
      };
    }
  }

  editable(field?: F): boolean {
    return (
      !this._readonly &&
      !this._deleted &&
      (field && !isEmpty(field) ? this.editableFields.includes(field as F) : true)
    );
  }

  set deleted(value: boolean) {
    if (this._deleted !== value) {
      this._deleted = value;
      this.reset();
    }
  }

  get deleted() {
    return this._deleted;
  }
}
