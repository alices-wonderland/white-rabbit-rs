import type { Model, Query, ReadApi } from "@core/services/api";

export const HIERARCHY_REPORT_API_KEY = Symbol("HIERARCHY_REPORT_API_KEY");

export const HIERARCHY_REPORT_TYPE = "hierarchyReports";

export class HierarchyReport implements Model<typeof HIERARCHY_REPORT_TYPE> {
  id: string;
  journalId: string;
  prefix: string;
  unit: string;
  values: Map<string, number>;

  constructor({ id, journalId, prefix, unit, values }: Omit<HierarchyReport, "modelType">) {
    this.id = id;
    this.journalId = journalId;
    this.prefix = prefix;
    this.unit = unit;
    this.values = values;
  }

  get modelType(): typeof HIERARCHY_REPORT_TYPE {
    return HIERARCHY_REPORT_TYPE;
  }
}

export interface HierarchyReportQuery extends Query {
  readonly id?: string[];
  readonly journalId?: string[];
  readonly start?: string;
  readonly end?: string;
}

export type HierarchyReportApi = ReadApi<HierarchyReport, HierarchyReportQuery>;
