import type { HierarchyReportQuery, HierarchyReportApi } from "@core/services";
import { AbstractReadApi } from "@/services/api";
import { HierarchyReport } from "@core/services";

class HierarchyReportApiImpl extends AbstractReadApi<HierarchyReport, HierarchyReportQuery> {
  protected override get modelType(): string {
    return "/hierarchy-reports";
  }

  protected override convert(input: Record<string, unknown>): HierarchyReport {
    return new HierarchyReport({
      id: input.id as string,
      journalId: input.journalId as string,
      prefix: input.prefix as string,
      unit: input.unit as string,
      values: input.values as Map<string, number>,
    });
  }
}

export const hierarchyReportApi: HierarchyReportApi = new HierarchyReportApiImpl();
