import { useCallback, useMemo } from "react"
import { useTranslation } from "react-i18next"
import { useQuery, useQueryClient } from "@tanstack/react-query"
import type {
  DataTableFetchResult,
  DataTableSortState,
} from "@workspace/app-kit"
import {
  getAnalyticsOverviewApi,
  getAnalyticsPagesApi,
  getAnalyticsSourcesApi,
  listAnalyticsAccountsApi,
  listAnalyticsSchoolTeamsApi,
  type AnalyticsRangeRequest,
} from "@/api"
import { toEndIso, toStartIso } from "./format"
import {
  buildMetricCards,
  buildPageChart,
  buildSourceKindChart,
  buildTrafficTrend,
} from "./metrics/analytics-metrics"
import {
  buildAccountListParams,
  buildSchoolTeamListParams,
  mapAccountRow,
  mapSchoolTeamRow,
} from "./table/logic"
import type {
  AnalyticsAccountRow,
  AnalyticsAccountTableQuery,
  AnalyticsDateRange,
  AnalyticsSchoolTeamRow,
  AnalyticsSchoolTeamTableQuery,
} from "./types"

export const analyticsQueryKey = ["admin", "analytics", "access"] as const

export function useAccessAnalyticsData(range: AnalyticsDateRange) {
  const { t } = useTranslation()
  const queryClient = useQueryClient()
  const rangeParams = useMemo(() => buildRangeParams(range), [range])

  const overviewQuery = useQuery({
    queryKey: [...analyticsQueryKey, "overview", rangeParams],
    queryFn: () => getAnalyticsOverviewApi(rangeParams),
  })
  const sourcesQuery = useQuery({
    queryKey: [...analyticsQueryKey, "sources", rangeParams],
    queryFn: () => getAnalyticsSourcesApi(rangeParams),
  })
  const pagesQuery = useQuery({
    queryKey: [...analyticsQueryKey, "pages", rangeParams],
    queryFn: () => getAnalyticsPagesApi(rangeParams),
  })

  const metricCards = useMemo(
    () => buildMetricCards(overviewQuery.data?.summary, t),
    [overviewQuery.data?.summary, t]
  )

  const trafficTrend = useMemo(
    () => buildTrafficTrend(overviewQuery.data),
    [overviewQuery.data]
  )

  const sourceKindChart = useMemo(
    () => buildSourceKindChart(sourcesQuery.data, t),
    [sourcesQuery.data, t]
  )

  const pageChart = useMemo(
    () => buildPageChart(pagesQuery.data, t),
    [pagesQuery.data, t]
  )

  const fetchAccounts = useCallback(
    async ({
      page,
      pageSize,
      query,
      signal,
      sort,
    }: {
      page: number
      pageSize: number
      query: AnalyticsAccountTableQuery
      signal: AbortSignal
      sort: DataTableSortState | null
    }): Promise<DataTableFetchResult<AnalyticsAccountRow>> => {
      void signal
      const response = await listAnalyticsAccountsApi(
        buildAccountListParams(page, pageSize, query, sort, rangeParams)
      )
      return {
        items: response.items.map((row) => mapAccountRow(row, t)),
        total: response.total,
      }
    },
    [rangeParams, t]
  )

  const fetchSchoolTeams = useCallback(
    async ({
      page,
      pageSize,
      query,
      signal,
      sort,
    }: {
      page: number
      pageSize: number
      query: AnalyticsSchoolTeamTableQuery
      signal: AbortSignal
      sort: DataTableSortState | null
    }): Promise<DataTableFetchResult<AnalyticsSchoolTeamRow>> => {
      void signal
      const response = await listAnalyticsSchoolTeamsApi(
        buildSchoolTeamListParams(page, pageSize, query, sort, rangeParams)
      )
      return {
        items: response.items.map(mapSchoolTeamRow),
        total: response.total,
      }
    },
    [rangeParams]
  )

  const refresh = useCallback(
    () =>
      queryClient.invalidateQueries({
        queryKey: analyticsQueryKey,
      }),
    [queryClient]
  )

  return {
    metricCards,
    trafficTrend,
    sourceKindChart,
    pageChart,
    pages: pagesQuery.data?.items ?? [],
    isLoading:
      overviewQuery.isLoading || sourcesQuery.isLoading || pagesQuery.isLoading,
    refresh,
    fetchAccounts,
    fetchSchoolTeams,
  }
}

function buildRangeParams(range: AnalyticsDateRange): AnalyticsRangeRequest {
  return {
    from: toStartIso(range.from),
    to: toEndIso(range.to),
    granularity: "day",
  }
}
