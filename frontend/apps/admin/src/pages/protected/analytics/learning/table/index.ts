import type {
  DataTableFetchResult,
  DataTableSortState,
} from "@workspace/app-kit"
import { useTranslation } from "react-i18next"
import {
  learningAuditColumns,
  learningInitialQuery,
  learningPageSizeOptions,
} from "../constants"
import { buildLearningColumns } from "./columns"
import { buildLearningBuiltInQueryFields } from "./query-fields"
import { useLearningRowActions } from "./row-actions"
import type { LearningStudentRow, LearningStudentTableQuery } from "../types"

export function useLearningStudentsTable(
  fetchData: (params: {
    page: number
    pageSize: number
    query: LearningStudentTableQuery
    signal: AbortSignal
    sort: DataTableSortState | null
  }) => Promise<DataTableFetchResult<LearningStudentRow>>
) {
  const { t } = useTranslation()
  const rowActions = useLearningRowActions()

  return {
    auditColumns: learningAuditColumns,
    columns: buildLearningColumns(t),
    fetchData,
    getRowId: (row: LearningStudentRow) => row.user_id,
    initialPageSize: 10,
    initialSort: {
      columnKey: "updatedAt",
      direction: "desc",
    } as const,
    initialQuery: learningInitialQuery,
    query: {
      builtInFields: buildLearningBuiltInQueryFields(t),
    },
    pageSizeOptions: learningPageSizeOptions,
    rowActions,
    bulkDelete: false,
    bulkUpdate: false,
    compactColumns: true,
    compactRows: true,
    fillWidth: false,
  } as const
}
