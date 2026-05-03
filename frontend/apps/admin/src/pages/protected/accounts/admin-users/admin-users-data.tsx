import { useCallback, useMemo } from "react"
import { useTranslation } from "react-i18next"
import { useQuery, useQueryClient } from "@tanstack/react-query"
import type {
  DataTableFetchResult,
  DataTableSortState,
} from "@workspace/app-kit"
import { listAdminUsersApi } from "@/api"
import { buildAdminUserMetricCards } from "./metrics"
import { compareSortValues, getAdminUserSortValue } from "./table/sort"
import {
  filterAdminUsers,
  paginateAdminUsers,
  sortAdminUsers,
} from "./table/logic"
import { adminUsersQueryKey } from "./constants"
import type { AdminUserRow, AdminUserTableQuery } from "./types"

export function useAdminUsersData() {
  const { t } = useTranslation()
  const queryClient = useQueryClient()

  const loadAdminUserRows = useCallback(async (): Promise<AdminUserRow[]> => {
    const adminUsers = await listAdminUsersApi()

    return adminUsers.map((adminUser) => {
      const roles = adminUser.adminRoles ?? adminUser.roles ?? []

      return {
        user_id: adminUser.userId,
        display_id: adminUser.displayId?.trim() || adminUser.userId,
        display_name:
          adminUser.displayName?.trim() ||
          t("admin.accounts.adminUsers.data.fallbackDisplayName"),
        remark: adminUser.remark ?? null,
        status: adminUser.status,
        roles: roles.map((role) => role.name),
      }
    })
  }, [t])

  const adminUsersQuery = useQuery({
    queryKey: adminUsersQueryKey,
    queryFn: loadAdminUserRows,
  })

  const metricCards = useMemo(
    () => buildAdminUserMetricCards(adminUsersQuery.data ?? [], t),
    [adminUsersQuery.data, t]
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
      query: AdminUserTableQuery
      signal: AbortSignal
      sort: DataTableSortState | null
    }): Promise<DataTableFetchResult<AdminUserRow>> => {
      void signal
      const rows = await queryClient.ensureQueryData({
        queryKey: adminUsersQueryKey,
        queryFn: loadAdminUserRows,
      })

      const filteredRows = filterAdminUsers(rows, query)
      const sortedRows = sortAdminUsers(
        filteredRows,
        sort,
        getAdminUserSortValue,
        compareSortValues
      )

      return paginateAdminUsers(sortedRows, page, pageSize)
    },
    [loadAdminUserRows, queryClient]
  )

  const invalidateAdminUsers = useCallback(
    () =>
      queryClient.refetchQueries({
        queryKey: adminUsersQueryKey,
      }),
    [queryClient]
  )

  return {
    metricCards,
    fetchData,
    invalidateAdminUsers,
  }
}
