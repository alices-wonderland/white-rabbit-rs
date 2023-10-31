import { ModuleRegistry } from "@ag-grid-community/core";
import { ClientSideRowModelModule } from "@ag-grid-community/client-side-row-model";
import { RangeSelectionModule } from "@ag-grid-enterprise/range-selection";
import { ClipboardModule } from "@ag-grid-enterprise/clipboard";
import { RowGroupingModule } from "@ag-grid-enterprise/row-grouping";
import { SetFilterModule } from "@ag-grid-enterprise/set-filter";
import { MultiFilterModule } from "@ag-grid-enterprise/multi-filter";
import { MenuModule } from "@ag-grid-enterprise/menu";
import { FiltersToolPanelModule } from "@ag-grid-enterprise/filter-tool-panel";
import { ColumnsToolPanelModule } from "@ag-grid-enterprise/column-tool-panel";

export default function () {
  ModuleRegistry.registerModules([
    ClipboardModule,
    ClientSideRowModelModule,
    ColumnsToolPanelModule,
    FiltersToolPanelModule,
    MenuModule,
    MultiFilterModule,
    RangeSelectionModule,
    RowGroupingModule,
    SetFilterModule,
  ]);
}
