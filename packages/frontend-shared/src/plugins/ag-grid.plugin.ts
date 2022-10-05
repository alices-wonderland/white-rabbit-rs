import { ModuleRegistry } from "@ag-grid-community/core";
import { ClientSideRowModelModule } from "@ag-grid-community/client-side-row-model";
import { RangeSelectionModule } from "@ag-grid-enterprise/range-selection";
import { ClipboardModule } from "@ag-grid-enterprise/clipboard";
import { InfiniteRowModelModule } from "@ag-grid-community/infinite-row-model";
import { RowGroupingModule } from "@ag-grid-enterprise/row-grouping";
import { MenuModule } from "@ag-grid-enterprise/menu";
import { ColumnsToolPanelModule } from "@ag-grid-enterprise/column-tool-panel";

export default function (): void {
  ModuleRegistry.registerModules([
    ClientSideRowModelModule,
    InfiniteRowModelModule,
    ClipboardModule,
    RangeSelectionModule,
    RowGroupingModule,
    MenuModule,
    ColumnsToolPanelModule,
  ]);
}
