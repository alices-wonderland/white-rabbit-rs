import { ModuleRegistry } from "@ag-grid-community/core";
import { ClientSideRowModelModule } from "@ag-grid-community/client-side-row-model";
import { RangeSelectionModule } from "@ag-grid-enterprise/range-selection";
import { ClipboardModule } from "@ag-grid-enterprise/clipboard";
import { InfiniteRowModelModule } from "@ag-grid-community/infinite-row-model";

export default function (): void {
  ModuleRegistry.registerModules([
    ClientSideRowModelModule,
    InfiniteRowModelModule,
    ClipboardModule,
    RangeSelectionModule,
  ]);
}
