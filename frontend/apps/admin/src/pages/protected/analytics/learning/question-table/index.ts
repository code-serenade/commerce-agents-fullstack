import type {
  DataTableFetchResult,
  DataTableSortState,
} from "@workspace/app-kit"
import { useTranslation } from "react-i18next"
import {
  learningPageSizeOptions,
  learningQuestionInitialQuery,
} from "../constants"
import type {
  LearningQuestionMetricRow,
  LearningQuestionTableQuery,
} from "../types"
import { buildLearningQuestionColumns } from "./columns"
import { buildLearningQuestionQueryFields } from "./query-fields"

export function useLearningQuestionTable(
  fetchData: (params: {
    page: number
    pageSize: number
    query: LearningQuestionTableQuery
    signal: AbortSignal
    sort: DataTableSortState | null
  }) => Promise<DataTableFetchResult<LearningQuestionMetricRow>>
) {
  const { t } = useTranslation()

  return {
    columns: buildLearningQuestionColumns(t),
    fetchData,
    getRowId: (row: LearningQuestionMetricRow) =>
      `${row.question_id}:${row.question_version}`,
    initialPageSize: 15,
    initialSort: {
      columnKey: "questionId",
      direction: "asc",
    } as const,
    initialQuery: learningQuestionInitialQuery,
    query: {
      builtInFields: buildLearningQuestionQueryFields(t),
      audit: false,
    },
    pageSizeOptions: learningPageSizeOptions,
    bulkDelete: false,
    bulkUpdate: false,
    compactColumns: true,
    compactRows: true,
    fillWidth: false,
  } as const
}

export { LearningQuestionTableView } from "./components/question-table-view"
