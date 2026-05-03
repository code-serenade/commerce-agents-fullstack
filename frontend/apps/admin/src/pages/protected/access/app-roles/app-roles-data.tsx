import { useCallback } from "react"
import { useQuery, useQueryClient } from "@tanstack/react-query"
import type {
  DataTableFetchResult,
  DataTableSortState,
} from "@workspace/app-kit"
import { listAppRolesApi } from "@/api"
import { appRolesQueryKey } from "./constants"
import { filterRoles, paginateRoles, sortRoles } from "./table/logic"
import { compareRoleSortValues, getRoleSortValue } from "./table/sort"
import type { RoleRow, RoleTableQuery } from "./types"

export function useAppRolesData() {
  const queryClient = useQueryClient()

  const loadRoleRows = useCallback(async (): Promise<RoleRow[]> => {
    const roles = await listAppRolesApi()

    return roles.map((role) => ({
      id: role.id,
      name: role.name,
      code: role.code,
    }))
  }, [])

  useQuery({
    queryKey: appRolesQueryKey,
    queryFn: loadRoleRows,
  })

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
      query: RoleTableQuery
      signal: AbortSignal
      sort: DataTableSortState | null
    }): Promise<DataTableFetchResult<RoleRow>> => {
      void signal
      const rows = await queryClient.ensureQueryData({
        queryKey: appRolesQueryKey,
        queryFn: loadRoleRows,
      })

      const filteredRows = filterRoles(rows, query)
      const sortedRows = sortRoles(
        filteredRows,
        sort,
        getRoleSortValue,
        compareRoleSortValues
      )

      return paginateRoles(sortedRows, page, pageSize)
    },
    [loadRoleRows, queryClient]
  )

  const invalidateRoles = useCallback(
    () =>
      queryClient.invalidateQueries({
        queryKey: appRolesQueryKey,
      }),
    [queryClient]
  )

  return {
    fetchData,
    invalidateRoles,
  }
}
