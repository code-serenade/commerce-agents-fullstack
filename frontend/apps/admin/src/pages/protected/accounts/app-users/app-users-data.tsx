import { useCallback, useMemo } from "react"
import { useTranslation } from "react-i18next"
import { useQuery, useQueryClient } from "@tanstack/react-query"
import type {
  DataTableFetchResult,
  DataTableSortState,
} from "@workspace/app-kit"
import {
  listAppUserMetricsApi,
  listAppUsersApi,
  type ListAppUsersRequest,
} from "@/api"
import { buildAppUserMetricCards } from "./metrics"
import { mapStatusLabelToApiStatus } from "./table/status"
import { appUserAuditColumns, appUserMetricsQueryKey } from "./constants"
import type {
  AppUserRow,
  AppUserTableQuery,
  AppUserTimeField,
} from "./types"

function mapAppUserRow(
  appUser: Awaited<ReturnType<typeof listAppUsersApi>>["items"][number],
  fallbackDisplayName: string
): AppUserRow {
  return {
    user_id: appUser.userId,
    display_id: appUser.displayId?.trim() || appUser.userId,
    display_name: appUser.displayName?.trim() || fallbackDisplayName,
    remark: appUser.remark ?? null,
    status: appUser.status,
    createdAt: appUser.createdAt ?? null,
    updatedAt: appUser.updatedAt ?? null,
    roles: appUser.roles.map((role) => role.name),
  }
}

function toStartOfDayIso(value: Date) {
  const date = new Date(value)
  date.setHours(0, 0, 0, 0)
  return date.toISOString()
}

function toEndOfDayIso(value: Date) {
  const date = new Date(value)
  date.setHours(23, 59, 59, 999)
  return date.toISOString()
}

function applyTimeRangeParams(
  params: ListAppUsersRequest,
  field: AppUserTimeField,
  range: AppUserTableQuery["auditRange"]
) {
  if (!range?.from && !range?.to) return

  const from = range.from ? toStartOfDayIso(range.from) : undefined
  const to = range.to ? toEndOfDayIso(range.to) : undefined

  if (field === "createdAt") {
    params.createdAtFrom = from
    params.createdAtTo = to
    return
  }

  params.updatedAtFrom = from
  params.updatedAtTo = to
}

function applySortParams(
  params: ListAppUsersRequest,
  sort: DataTableSortState | null
) {
  if (sort?.columnKey !== "createdAt" && sort?.columnKey !== "updatedAt") {
    return
  }

  params.sortBy = sort.columnKey
  params.sortOrder = sort.direction
}

function buildAppUserListParams(
  page: number,
  pageSize: number,
  query: AppUserTableQuery,
  sort: DataTableSortState | null
): ListAppUsersRequest {
  const params: ListAppUsersRequest = {
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

  const auditField =
    appUserAuditColumns.length > 1 ? query.auditField : appUserAuditColumns[0]

  applySortParams(params, sort)

  if (auditField) {
    applyTimeRangeParams(params, auditField, query.auditRange)
  }

  return params
}

export function useAppUsersData() {
  const { t } = useTranslation()
  const queryClient = useQueryClient()

  const loadAppUserPage = useCallback(
    async (params: ListAppUsersRequest) => {
      const response = await listAppUsersApi(params)
      const fallbackDisplayName = t(
        "admin.accounts.appUsers.data.fallbackDisplayName"
      )

      return {
        items: response.items.map((item) =>
          mapAppUserRow(item, fallbackDisplayName)
        ),
        total: response.total,
      }
    },
    [t]
  )

  const appUserMetricsQuery = useQuery({
    queryKey: appUserMetricsQueryKey,
    queryFn: listAppUserMetricsApi,
  })

  const metricCards = useMemo(
    () => buildAppUserMetricCards(appUserMetricsQuery.data, t),
    [appUserMetricsQuery.data, t]
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
      query: AppUserTableQuery
      signal: AbortSignal
      sort: DataTableSortState | null
    }): Promise<DataTableFetchResult<AppUserRow>> => {
      void signal
      const serverPage = await loadAppUserPage(
        buildAppUserListParams(page, pageSize, query, sort)
      )

      return {
        items: serverPage.items,
        total: serverPage.total,
      }
    },
    [loadAppUserPage]
  )

  const invalidateAppUsers = useCallback(
    () =>
      queryClient.refetchQueries({
        queryKey: appUserMetricsQueryKey,
      }),
    [queryClient]
  )

  return {
    metricCards,
    fetchData,
    invalidateAppUsers,
  }
}
