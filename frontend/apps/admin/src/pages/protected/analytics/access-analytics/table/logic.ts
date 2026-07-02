import type { DataTableSortState } from "@workspace/app-kit"
import type { TFunction } from "i18next"
import type {
  AnalyticsAccountRowResponse,
  AnalyticsRangeRequest,
  AnalyticsSchoolTeamRowResponse,
  ListAnalyticsAccountsRequest,
  ListAnalyticsSchoolTeamsRequest,
} from "@/api"
import { resolveRouteDisplayInfo } from "../../route-labels"
import type {
  AnalyticsAccountRow,
  AnalyticsAccountTableQuery,
  AnalyticsSchoolTeamRow,
  AnalyticsSchoolTeamTableQuery,
} from "../types"

export function buildAccountListParams(
  page: number,
  pageSize: number,
  query: AnalyticsAccountTableQuery,
  sort: DataTableSortState | null,
  range: AnalyticsRangeRequest
): ListAnalyticsAccountsRequest {
  const params: ListAnalyticsAccountsRequest = {
    ...range,
    page,
    pageSize,
  }
  const keyword = query.keyword.trim()
  if (keyword) params.keyword = keyword
  if (query.schoolName.trim()) params.schoolName = query.schoolName.trim()
  if (query.teamName.trim()) params.teamName = query.teamName.trim()

  if (sort) {
    params.sortBy = mapAccountSort(sort.columnKey)
    params.sortOrder = sort.direction
  }
  return params
}

export function buildSchoolTeamListParams(
  page: number,
  pageSize: number,
  query: AnalyticsSchoolTeamTableQuery,
  sort: DataTableSortState | null,
  range: AnalyticsRangeRequest
): ListAnalyticsSchoolTeamsRequest {
  const params: ListAnalyticsSchoolTeamsRequest = {
    ...range,
    page,
    pageSize,
  }
  if (query.schoolName.trim()) params.schoolName = query.schoolName.trim()
  if (query.teamName.trim()) params.teamName = query.teamName.trim()

  if (sort) {
    params.sortBy = mapSchoolTeamSort(sort.columnKey)
    params.sortOrder = sort.direction
  }
  return params
}

export function mapAccountRow(
  row: AnalyticsAccountRowResponse,
  t: TFunction
): AnalyticsAccountRow {
  return {
    user_id: row.userId,
    display_id: row.displayId,
    display_name: row.displayName,
    school_name: row.schoolName,
    team_name: row.teamName,
    pv: row.pv,
    sessions: row.sessions,
    active_days: row.activeDays,
    agent_questions: row.agentQuestions,
    token_used: row.tokenUsed,
    lastSeenAt: row.lastSeenAt ?? null,
    top_pages: row.topPages
      .map((page) => {
        const route = resolveRouteDisplayInfo(page.routePath, t)
        return {
          description: route.description,
          label: route.label,
          path: route.rawPath,
          pv: page.pv,
        }
      }),
  }
}

export function mapSchoolTeamRow(
  row: AnalyticsSchoolTeamRowResponse
): AnalyticsSchoolTeamRow {
  return {
    id: `${row.schoolName}::${row.teamName}`,
    school_name: row.schoolName,
    team_name: row.teamName,
    account_count: row.accountCount,
    active_accounts: row.activeAccounts,
    pv: row.pv,
    uv: row.uv,
    sessions: row.sessions,
    agent_questions: row.agentQuestions,
    token_used: row.tokenUsed,
    lastSeenAt: row.lastSeenAt ?? null,
  }
}

function mapAccountSort(
  columnKey: string
): ListAnalyticsAccountsRequest["sortBy"] {
  switch (columnKey) {
    case "pv":
      return "pv"
    case "sessions":
      return "sessions"
    case "active_days":
      return "activeDays"
    case "agent_questions":
      return "agentQuestions"
    case "lastSeenAt":
      return "lastSeenAt"
    default:
      return undefined
  }
}

function mapSchoolTeamSort(
  columnKey: string
): ListAnalyticsSchoolTeamsRequest["sortBy"] {
  switch (columnKey) {
    case "active_accounts":
      return "activeAccounts"
    case "pv":
      return "pv"
    case "sessions":
      return "sessions"
    case "agent_questions":
      return "agentQuestions"
    case "lastSeenAt":
      return "lastSeenAt"
    default:
      return undefined
  }
}
