import { useCallback, useMemo } from "react"
import { useTranslation } from "react-i18next"
import { useQuery, useQueryClient } from "@tanstack/react-query"
import type {
  DataTableFetchResult,
  DataTableSortState,
} from "@workspace/app-kit"
import {
  getLearningStudentMetricsApi,
  listLearningStudentsApi,
} from "@/api"
import { learningMetricsQueryKey } from "./constants"
import {
  buildLearningMetricCards,
  buildLearningPilotMetricCards,
  buildLearningQuestionGovernanceMetricCards,
} from "./metrics"
import { buildLearningListParams, mapLearningStudentRow } from "./table/logic"
import {
  mapLearningQuestionMetricRow,
  queryLearningQuestionRows,
} from "./question-table/logic"
import type {
  LearningQuestionMetricRow,
  LearningQuestionTableQuery,
  LearningStudentRow,
  LearningStudentTableQuery,
} from "./types"

export function useLearningData() {
  const { t } = useTranslation()
  const queryClient = useQueryClient()

  const metricsQuery = useQuery({
    queryKey: learningMetricsQueryKey,
    queryFn: getLearningStudentMetricsApi,
  })

  const metricCards = useMemo(
    () => buildLearningMetricCards(metricsQuery.data, t),
    [metricsQuery.data, t]
  )
  const pilotMetricCards = useMemo(
    () => buildLearningPilotMetricCards(metricsQuery.data, t),
    [metricsQuery.data, t]
  )
  const questionGovernanceMetricCards = useMemo(
    () => buildLearningQuestionGovernanceMetricCards(metricsQuery.data, t),
    [metricsQuery.data, t]
  )
  const questionRows = useMemo(
    () =>
      (metricsQuery.data?.probePilot?.questionMetrics ?? []).map(
        mapLearningQuestionMetricRow
      ),
    [metricsQuery.data?.probePilot?.questionMetrics]
  )

  const fetchData = useCallback(
    async ({
      page,
      pageSize,
      query,
      signal,
      sort,
    }: {
      page: number
      pageSize: number
      query: LearningStudentTableQuery
      signal: AbortSignal
      sort: DataTableSortState | null
    }): Promise<DataTableFetchResult<LearningStudentRow>> => {
      void signal
      const response = await listLearningStudentsApi(
        buildLearningListParams(page, pageSize, query, sort)
      )

      return {
        items: response.items.map(mapLearningStudentRow),
        total: response.total,
      }
    },
    []
  )

  const invalidate = useCallback(
    () =>
      queryClient.invalidateQueries({
        queryKey: learningMetricsQueryKey,
      }),
    [queryClient]
  )

  const fetchQuestionData = useCallback(
    async ({
      page,
      pageSize,
      query,
      signal,
      sort,
    }: {
      page: number
      pageSize: number
      query: LearningQuestionTableQuery
      signal: AbortSignal
      sort: DataTableSortState | null
    }): Promise<DataTableFetchResult<LearningQuestionMetricRow>> => {
      if (signal.aborted) return { items: [], total: 0 }
      return queryLearningQuestionRows(questionRows, page, pageSize, query, sort)
    },
    [questionRows]
  )

  return {
    metricCards,
    pilotMetricCards,
    questionGovernanceMetricCards,
    fetchData,
    fetchQuestionData,
    invalidate,
  }
}
