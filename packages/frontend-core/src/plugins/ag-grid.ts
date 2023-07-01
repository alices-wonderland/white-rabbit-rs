import { ModuleRegistry } from "@ag-grid-community/core";
import { ClientSideRowModelModule } from "@ag-grid-community/client-side-row-model";
import { RangeSelectionModule } from "@ag-grid-enterprise/range-selection";
import { ClipboardModule } from "@ag-grid-enterprise/clipboard";
import { RowGroupingModule } from "@ag-grid-enterprise/row-grouping";
import { SetFilterModule } from "@ag-grid-enterprise/set-filter";
import { MultiFilterModule } from "@ag-grid-enterprise/multi-filter";
import { MenuModule } from "@ag-grid-enterprise/menu";
import { SideBarModule } from "@ag-grid-enterprise/side-bar";
import { FiltersToolPanelModule } from "@ag-grid-enterprise/filter-tool-panel";
import { ColumnsToolPanelModule } from "@ag-grid-enterprise/column-tool-panel";
import { RichSelectModule } from "@ag-grid-enterprise/rich-select";
import { StatusBarModule } from "@ag-grid-enterprise/status-bar";

export default function () {
  ModuleRegistry.registerModules([
    ClientSideRowModelModule,
    RangeSelectionModule,
    ClipboardModule,
    RowGroupingModule,
    SetFilterModule,
    MultiFilterModule,
    MenuModule,
    SideBarModule,
    FiltersToolPanelModule,
    ColumnsToolPanelModule,
    RichSelectModule,
    StatusBarModule,
  ]);
}
