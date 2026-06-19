import { useCallback, useMemo, useState } from "react"
import { useTranslation } from "react-i18next"
import type {
  DataTableFetchResult,
  DataTableSortState,
} from "@workspace/app-kit"
import {
  listTeamUsagesApi,
  type ListTeamUsagesRequest,
  type TeamUsageResponse,
} from "@/api"
import { buildTeamUsageMetricCards } from "./metrics"
import { mapStatusLabelToApiStatus } from "./table/status"
import type { TeamUsageRow, TeamUsageTableQuery } from "./types"

function mapTeamUsageRow(row: TeamUsageResponse): TeamUsageRow {
  return {
    id: row.id,
    user_id: row.userId,
    display_id: row.displayId,
    display_name: row.displayName,
    school_name: row.schoolName,
    team_name: row.teamName,
    question_limit: row.questionLimit,
    question_used: row.questionUsed,
    question_remaining: row.questionRemaining,
    token_used: row.tokenUsed,
    status: row.status,
    remark: row.remark ?? null,
    lastUsedAt: row.lastUsedAt ?? null,
    createdAt: row.createdAt ?? null,
    updatedAt: row.updatedAt ?? null,
  }
}

function mapSortColumn(columnKey: string): ListTeamUsagesRequest["sortBy"] {
  switch (columnKey) {
    case "display_id":
      return "displayId"
    case "question_used":
      return "questionUsed"
    case "question_remaining":
      return "questionRemaining"
    case "lastUsedAt":
      return "lastUsedAt"
    case "updatedAt":
      return "updatedAt"
    default:
      return undefined
  }
}

function buildTeamUsageListParams(
  page: number,
  pageSize: number,
  query: TeamUsageTableQuery,
  sort: DataTableSortState | null
): ListTeamUsagesRequest {
  const params: ListTeamUsagesRequest = {
    page,
    pageSize,
  }
  const keyword = query.keyword.trim()

  if (keyword) {
    params.keyword = keyword
  }

  if (query.status) {
    params.status = mapStatusLabelToApiStatus(query.status)
  }

  if (sort) {
    const sortBy = mapSortColumn(sort.columnKey)
    if (sortBy) {
      params.sortBy = sortBy
      params.sortOrder = sort.direction
    }
  }

  return params
}

export function useTeamUsageData() {
  const { t } = useTranslation()
  const [snapshot, setSnapshot] = useState<TeamUsageRow[]>([])
  const [total, setTotal] = useState(0)

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
      query: TeamUsageTableQuery
      signal: AbortSignal
      sort: DataTableSortState | null
    }): Promise<DataTableFetchResult<TeamUsageRow>> => {
      void signal
      const response = await listTeamUsagesApi(
        buildTeamUsageListParams(page, pageSize, query, sort)
      )
      const items = response.items.map(mapTeamUsageRow)

      setSnapshot(items)
      setTotal(response.total)

      return {
        items,
        total: response.total,
      }
    },
    []
  )

  const metricCards = useMemo(
    () => buildTeamUsageMetricCards(snapshot, total, t),
    [snapshot, total, t]
  )

  const invalidateTeamUsage = useCallback(async () => {
    setSnapshot([])
  }, [])

  return {
    metricCards,
    fetchData,
    invalidateTeamUsage,
  }
}
